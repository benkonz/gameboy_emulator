mod screen;
mod web_rtc;

use crate::screen::Screen;
use crate::web_rtc::WebRTC;
use gameboy_core::{Button, Cartridge, ControllerEvent, Gameboy, Rtc, StepResult};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AudioBuffer, AudioContext, CanvasRenderingContext2d, Document, HtmlCanvasElement, ImageData,
    KeyboardEvent, Window,
};

struct WebCtx {
    visible_canvas: HtmlCanvasElement,
    visible_ctx: CanvasRenderingContext2d,
    hidden_canvas: HtmlCanvasElement,
    hidden_ctx: CanvasRenderingContext2d,
    audio: AudioContext,
    audio_buffer_pool: Rc<RefCell<Vec<AudioBuffer>>>,
    play_timestamp: Rc<RefCell<f64>>,
}

struct EmulatorState {
    web: WebCtx,
    gameboy: Gameboy,
    screen: Screen,
    controller_receiver: mpsc::Receiver<ControllerEvent>,
    should_save_to_local: Rc<RefCell<bool>>,
    ram_str: Rc<RefCell<String>>,
    busy: bool,
    audio_underrun: Option<usize>,
}

impl EmulatorState {
    pub fn emulate_until_vblank_or_audio(&mut self) -> StepResult {
        let step_result = loop {
            let step_result = self.gameboy.emulate(&mut self.screen);
            match step_result {
                StepResult::VBlank | StepResult::AudioBufferFull => break step_result,
                _ => (),
            }
        };

        if step_result == StepResult::AudioBufferFull {
            self.play_audio();
        }

        loop {
            match self.controller_receiver.try_recv() {
                Ok(ControllerEvent::Pressed(button)) => self.gameboy.press_button(button),
                Ok(ControllerEvent::Released(button)) => self.gameboy.release_button(button),
                Err(_) => break,
            }
        }

        step_result
    }

    pub fn play_audio(&mut self) {
        let samples = self.gameboy.get_audio_buffer();
        let stereo_frames = samples.len() / 2;

        let sample_rate = 44100.0_f64;
        let latency = 0.032_f64;

        let audio_buffer = {
            let mut pool = self.web.audio_buffer_pool.borrow_mut();
            pool.pop()
        };

        let audio_buffer = audio_buffer.unwrap_or_else(|| {
            self.web
                .audio
                .create_buffer(2, stereo_frames as u32, sample_rate as f32)
                .expect("create_buffer")
        });

        let mut left = vec![0.0_f32; stereo_frames];
        let mut right = vec![0.0_f32; stereo_frames];
        for i in 0..stereo_frames {
            left[i] = samples[i * 2];
            right[i] = samples[i * 2 + 1];
        }

        audio_buffer
            .copy_to_channel(&left, 0)
            .expect("copy_to_channel L");
        audio_buffer
            .copy_to_channel(&right, 1)
            .expect("copy_to_channel R");

        let node = self
            .web
            .audio
            .create_buffer_source()
            .expect("buffer source");
        node.set_buffer(Some(&audio_buffer));
        node.connect_with_audio_node(&self.web.audio.destination())
            .expect("connect");

        let pool = self.web.audio_buffer_pool.clone();
        let ended = Closure::once(Box::new(move || {
            pool.borrow_mut().push(audio_buffer);
        }));
        node.add_event_listener_with_callback("ended", ended.as_ref().unchecked_ref())
            .expect("add ended listener");
        ended.forget();

        let current_time = self.web.audio.current_time();
        let mut ts = self.web.play_timestamp.borrow_mut();
        let buffered = *ts - (current_time + latency);
        let play_when = (current_time + latency).max(*ts);
        node.start_with_when(play_when).expect("start");
        *ts = play_when + (stereo_frames as f64) / sample_rate;

        // Keep the node alive until the `ended` event; dropping the wrapper early can glitch playback.
        std::mem::forget(node);

        if buffered < 0.000 {
            self.audio_underrun = Some(std::cmp::max(self.audio_underrun.unwrap_or(0), 3));
        } else if buffered < 0.010 {
            self.audio_underrun = Some(std::cmp::max(self.audio_underrun.unwrap_or(0), 2));
        } else if buffered < 0.020 {
            self.audio_underrun = Some(std::cmp::max(self.audio_underrun.unwrap_or(0), 1));
        }
    }

    pub fn render(&self) {
        let frame_buffer: &[u8] = self.screen.get_frame_buffer();
        let img = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(frame_buffer),
            160,
            144,
        )
        .expect("ImageData");

        self.web
            .hidden_ctx
            .put_image_data(&img, 0.0, 0.0)
            .expect("put_image_data");

        let vw = self.web.visible_canvas.width() as f64;
        let vh = self.web.visible_canvas.height() as f64;
        self.web
            .visible_ctx
            .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &self.web.hidden_canvas,
                0.0,
                0.0,
                160.0,
                144.0,
                0.0,
                0.0,
                vw,
                vh,
            )
            .expect("draw_image");
    }

    pub fn save_ram_data(&mut self) {
        if *self.should_save_to_local.borrow() && self.gameboy.get_cartridge().has_battery() {
            let name = self.gameboy.get_cartridge().get_name();
            let storage = window()
                .local_storage()
                .expect("local_storage")
                .expect("storage object");
            storage
                .set_item(name, &self.ram_str.borrow())
                .expect("set ram");
            *self.should_save_to_local.borrow_mut() = false;
        }
    }

    pub fn save_timestamp_data(&mut self) {
        if self.gameboy.get_cartridge().has_rtc() {
            let name = format!("{}-timestamp", self.gameboy.get_cartridge().get_name());
            let (rtc_data, last_timestamp) = self.gameboy.get_cartridge().get_last_timestamp();
            let mut rtc_bytes = rtc_data.to_bytes().to_vec();
            let mut last_timestamp_bytes = u64::to_ne_bytes(last_timestamp).to_vec();
            rtc_bytes.append(&mut last_timestamp_bytes);
            let timestamp_data_str: String = rtc_bytes
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect();
            let storage = window()
                .local_storage()
                .expect("local_storage")
                .expect("storage object");
            storage
                .set_item(&name, &timestamp_data_str)
                .expect("set timestamp");
        }
    }

    pub fn set_ram_change_listener(&mut self) {
        let ram_str = self.ram_str.clone();
        let should_save_to_local = self.should_save_to_local.clone();
        let has_battery = self.gameboy.get_cartridge().has_battery();
        self.gameboy
            .set_ram_change_callback(Box::new(move |address, value| {
                if has_battery {
                    let byte_chars: Vec<char> = format!("{:02x}", value).chars().collect();
                    let (first, second) = (byte_chars[0] as u8, byte_chars[1] as u8);
                    unsafe {
                        let mut ram_str_ref = ram_str.borrow_mut();
                        let bytes = ram_str_ref.as_bytes_mut();
                        bytes[address * 2] = first;
                        bytes[address * 2 + 1] = second;
                    }
                    *should_save_to_local.borrow_mut() = true;
                }
            }));
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DOMInfo {
    up_button_id: String,
    down_button_id: String,
    left_button_id: String,
    right_button_id: String,
    up_left_button_id: String,
    up_right_button_id: String,
    down_left_button_id: String,
    down_right_button_id: String,
    a_button_id: String,
    b_button_id: String,
    start_button_id: String,
    select_button_id: String,
    canvas_id: String,
}

pub fn start(rom: Vec<u8>, dom_ids: DOMInfo) -> Result<(), String> {
    let (sender, receiver) = mpsc::channel();
    let should_save_to_local = Rc::new(RefCell::new(false));

    let up_btn = get_element_by_id(dom_ids.up_button_id.as_str())?;
    let down_btn = get_element_by_id(dom_ids.down_button_id.as_str())?;
    let left_btn = get_element_by_id(dom_ids.left_button_id.as_str())?;
    let right_btn = get_element_by_id(dom_ids.right_button_id.as_str())?;
    let up_left_btn = get_element_by_id(dom_ids.up_left_button_id.as_str())?;
    let up_right_btn = get_element_by_id(dom_ids.up_right_button_id.as_str())?;
    let down_left_btn = get_element_by_id(dom_ids.down_left_button_id.as_str())?;
    let down_right_btn = get_element_by_id(dom_ids.down_right_button_id.as_str())?;
    let a_btn = get_element_by_id(dom_ids.a_button_id.as_str())?;
    let b_btn = get_element_by_id(dom_ids.b_button_id.as_str())?;
    let start_btn = get_element_by_id(dom_ids.start_button_id.as_str())?;
    let select_btn = get_element_by_id(dom_ids.select_button_id.as_str())?;

    add_button_event_listeners(&up_btn, Button::Up, sender.clone());
    add_button_event_listeners(&down_btn, Button::Down, sender.clone());
    add_button_event_listeners(&left_btn, Button::Left, sender.clone());
    add_button_event_listeners(&right_btn, Button::Right, sender.clone());
    add_button_event_listeners(&start_btn, Button::Start, sender.clone());
    add_button_event_listeners(&select_btn, Button::Select, sender.clone());
    add_button_event_listeners(&a_btn, Button::A, sender.clone());
    add_button_event_listeners(&b_btn, Button::B, sender.clone());

    add_multi_button_event_listeners(&up_left_btn, Button::Up, Button::Left, sender.clone());
    add_multi_button_event_listeners(&up_right_btn, Button::Up, Button::Right, sender.clone());
    add_multi_button_event_listeners(&down_left_btn, Button::Down, Button::Left, sender.clone());
    add_multi_button_event_listeners(&down_right_btn, Button::Down, Button::Right, sender.clone());

    add_keyboard_listeners(sender)?;

    let visible_canvas = get_canvas(dom_ids.canvas_id.as_str())?;

    visible_canvas.set_width(640);
    visible_canvas.set_height(576);

    let visible_ctx = visible_canvas
        .get_context("2d")
        .map_err(|e| format!("{:?}", e))?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|e| format!("{:?}", e))?;
    visible_ctx.set_image_smoothing_enabled(false);

    let doc = document();
    let hidden_canvas = doc
        .create_element("canvas")
        .map_err(|e| format!("{:?}", e))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "hidden canvas".to_string())?;
    hidden_canvas.set_width(160);
    hidden_canvas.set_height(144);
    hidden_canvas
        .set_attribute("style", "display:none")
        .map_err(|e| format!("{:?}", e))?;

    let hidden_ctx = hidden_canvas
        .get_context("2d")
        .map_err(|e| format!("{:?}", e))?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|e| format!("{:?}", e))?;
    hidden_ctx.set_image_smoothing_enabled(false);

    let audio = AudioContext::new().map_err(|e| format!("{:?}", e))?;

    let web = WebCtx {
        visible_canvas,
        visible_ctx,
        hidden_canvas,
        hidden_ctx,
        audio: audio.clone(),
        audio_buffer_pool: Rc::new(RefCell::new(Vec::new())),
        play_timestamp: Rc::new(RefCell::new(0.0)),
    };

    spawn_resume_audio(&audio);
    attach_extra_resume_handlers(&audio);

    let rtc = Box::new(WebRTC::new());
    let mut gameboy = Gameboy::from_rom(rom, rtc)?;
    load_ram_save_data(gameboy.get_cartridge_mut());
    load_timestamp_data(gameboy.get_cartridge_mut());
    let ram = gameboy.get_cartridge().get_ram().to_vec();
    let ram_str = Rc::new(RefCell::new(
        ram.iter().map(|byte| format!("{:02x}", byte)).collect(),
    ));
    let screen = Screen::new();

    let mut emulator_state = EmulatorState {
        web,
        gameboy,
        screen,
        controller_receiver: receiver,
        should_save_to_local,
        ram_str,
        busy: false,
        audio_underrun: None,
    };

    emulator_state.set_ram_change_listener();

    let emulator_state = Rc::new(RefCell::new(emulator_state));

    let esc = emulator_state.clone();
    let tick_ms = (1000.0 / 60.0) as i32;
    let interval_closure = Closure::wrap(Box::new(move || {
        if !esc.borrow().busy {
            emulate_a_single_frame(esc.clone());
        }
        let mut st = esc.borrow_mut();
        st.render();
        st.save_ram_data();
        st.save_timestamp_data();
    }) as Box<dyn FnMut()>);

    window()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            interval_closure.as_ref().unchecked_ref(),
            tick_ms,
        )
        .expect("set_interval");
    interval_closure.forget();

    Ok(())
}

fn spawn_resume_audio(ctx: &AudioContext) {
    let ctx = ctx.clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(p) = ctx.resume() {
            let _ = JsFuture::from(p).await;
        }
    });
}

fn attach_extra_resume_handlers(audio: &AudioContext) {
    let a = audio.clone();
    let c = Closure::wrap(Box::new(move || spawn_resume_audio(&a)) as Box<dyn FnMut()>);
    let _ = window().add_event_listener_with_callback("pointerdown", c.as_ref().unchecked_ref());
    c.forget();
}

fn emulate_a_single_frame(emulator_state: Rc<RefCell<EmulatorState>>) {
    emulator_state.borrow_mut().busy = true;

    let esc = emulator_state.clone();
    let closure = Closure::once(Box::new(move || {
        let step_result = esc.borrow_mut().emulate_until_vblank_or_audio();
        match step_result {
            StepResult::AudioBufferFull => {
                let esc2 = esc.clone();
                let c2 = Closure::once(Box::new(move || emulate_a_single_frame(esc2)));
                window()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        c2.as_ref().unchecked_ref(),
                        0,
                    )
                    .unwrap();
                c2.forget();
            }
            StepResult::VBlank => {
                let mut st = esc.borrow_mut();
                if let Some(count) = st.audio_underrun.take() {
                    for _ in 0..count {
                        st.emulate_until_vblank_or_audio();
                    }
                }
                st.busy = false;
            }
            StepResult::Nothing => {
                unreachable!("emulate_until_vblank_or_audio only returns VBlank or AudioBufferFull")
            }
        }
    }));

    window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), 0)
        .unwrap();
    closure.forget();
}

fn window() -> Window {
    web_sys::window().expect("window")
}

fn document() -> Document {
    window().document().expect("document")
}

fn get_element_by_id(id: &str) -> Result<web_sys::Element, String> {
    document()
        .get_element_by_id(id)
        .ok_or_else(|| format!("unable to find element with id: {}", id))
}

fn get_canvas(id: &str) -> Result<HtmlCanvasElement, String> {
    get_element_by_id(id)?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|e| format!("{:?}", e))
}

fn add_keyboard_listeners(sender: mpsc::Sender<ControllerEvent>) -> Result<(), String> {
    let sender_down = sender.clone();
    let kd = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        let _ = match e.key().as_str() {
            "ArrowUp" => sender_down.send(ControllerEvent::Pressed(Button::Up)),
            "ArrowDown" => sender_down.send(ControllerEvent::Pressed(Button::Down)),
            "ArrowLeft" => sender_down.send(ControllerEvent::Pressed(Button::Left)),
            "ArrowRight" => sender_down.send(ControllerEvent::Pressed(Button::Right)),
            "z" => sender_down.send(ControllerEvent::Pressed(Button::A)),
            "x" => sender_down.send(ControllerEvent::Pressed(Button::B)),
            "Enter" => sender_down.send(ControllerEvent::Pressed(Button::Select)),
            " " => sender_down.send(ControllerEvent::Pressed(Button::Start)),
            _ => Ok(()),
        };
    }) as Box<dyn FnMut(KeyboardEvent)>);

    window()
        .add_event_listener_with_callback("keydown", kd.as_ref().unchecked_ref())
        .map_err(|e| format!("{:?}", e))?;
    kd.forget();

    let ku = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        let _ = match e.key().as_str() {
            "ArrowUp" => sender.send(ControllerEvent::Released(Button::Up)),
            "ArrowDown" => sender.send(ControllerEvent::Released(Button::Down)),
            "ArrowLeft" => sender.send(ControllerEvent::Released(Button::Left)),
            "ArrowRight" => sender.send(ControllerEvent::Released(Button::Right)),
            "z" => sender.send(ControllerEvent::Released(Button::A)),
            "x" => sender.send(ControllerEvent::Released(Button::B)),
            "Enter" => sender.send(ControllerEvent::Released(Button::Select)),
            " " => sender.send(ControllerEvent::Released(Button::Start)),
            _ => Ok(()),
        };
    }) as Box<dyn FnMut(KeyboardEvent)>);

    window()
        .add_event_listener_with_callback("keyup", ku.as_ref().unchecked_ref())
        .map_err(|e| format!("{:?}", e))?;
    ku.forget();

    Ok(())
}

fn add_button_event_listeners(
    element: &web_sys::Element,
    button: Button,
    sender: mpsc::Sender<ControllerEvent>,
) {
    add_mouse_touch_listener(
        element,
        "mousedown",
        ControllerEvent::Pressed(button),
        sender.clone(),
    );
    add_mouse_touch_listener(
        element,
        "mouseup",
        ControllerEvent::Released(button),
        sender.clone(),
    );
    add_mouse_touch_listener(
        element,
        "touchstart",
        ControllerEvent::Pressed(button),
        sender.clone(),
    );
    add_mouse_touch_listener(
        element,
        "touchend",
        ControllerEvent::Released(button),
        sender,
    );
}

fn add_multi_button_event_listeners(
    element: &web_sys::Element,
    first: Button,
    second: Button,
    sender: mpsc::Sender<ControllerEvent>,
) {
    add_mouse_touch_multi_press(element, "mousedown", first, second, sender.clone());
    add_mouse_touch_multi_release(element, "mouseup", first, second, sender.clone());
    add_mouse_touch_multi_press(element, "touchstart", first, second, sender.clone());
    add_mouse_touch_multi_release(element, "touchend", first, second, sender);
}

fn add_mouse_touch_listener(
    element: &web_sys::Element,
    event_type: &'static str,
    ev: ControllerEvent,
    sender: mpsc::Sender<ControllerEvent>,
) {
    let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let _ = sender.send(ev);
    }) as Box<dyn FnMut(web_sys::Event)>);
    let _ = element.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref());
    closure.forget();
}

fn add_mouse_touch_multi_press(
    element: &web_sys::Element,
    event_type: &'static str,
    first: Button,
    second: Button,
    sender: mpsc::Sender<ControllerEvent>,
) {
    let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let _ = sender.send(ControllerEvent::Pressed(first));
        let _ = sender.send(ControllerEvent::Pressed(second));
    }) as Box<dyn FnMut(web_sys::Event)>);
    let _ = element.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref());
    closure.forget();
}

fn add_mouse_touch_multi_release(
    element: &web_sys::Element,
    event_type: &'static str,
    first: Button,
    second: Button,
    sender: mpsc::Sender<ControllerEvent>,
) {
    let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let _ = sender.send(ControllerEvent::Released(first));
        let _ = sender.send(ControllerEvent::Released(second));
    }) as Box<dyn FnMut(web_sys::Event)>);
    let _ = element.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref());
    closure.forget();
}

fn load_ram_save_data(cartridge: &mut Cartridge) {
    let storage = match window().local_storage().ok().flatten() {
        Some(s) => s,
        None => return,
    };
    if let Ok(Some(ram_str)) = storage.get_item(cartridge.get_name()) {
        let chars: Vec<char> = ram_str.chars().collect();
        let bytes: Vec<u8> = chars
            .chunks(2)
            .map(|chunk| {
                let byte: String = chunk.iter().collect();
                u8::from_str_radix(&byte, 16).unwrap()
            })
            .collect();
        cartridge.set_ram(bytes);
    }
}

fn load_timestamp_data(cartridge: &mut Cartridge) {
    let storage = match window().local_storage().ok().flatten() {
        Some(s) => s,
        None => return,
    };
    let key = format!("{}-timestamp", cartridge.get_name());
    if let Ok(Some(timestamp_str)) = storage.get_item(&key) {
        let chars: Vec<char> = timestamp_str.chars().collect();
        let bytes: Vec<u8> = chars
            .chunks(2)
            .map(|chunk| {
                let byte: String = chunk.iter().collect();
                u8::from_str_radix(&byte, 16).unwrap()
            })
            .collect();
        let rtc = Rtc::from_bytes(&bytes[..5]);
        let mut timestamp_data = [0; 8];
        timestamp_data.copy_from_slice(&bytes[5..]);
        let timestamp = u64::from_ne_bytes(timestamp_data);
        cartridge.set_last_timestamp(rtc, timestamp);
    }
}
