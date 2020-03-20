// needed by the js! macro in the play_audio function
#![recursion_limit = "2048"]

#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;

mod screen;
mod web_rtc;

use crate::screen::Screen;
use crate::web_rtc::WebRTC;
use gameboy_core::{Button, Cartridge, ControllerEvent, Gameboy, Rtc, StepResult};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::event::{
    ConcreteEvent, KeyDownEvent, KeyUpEvent, MouseDownEvent, MouseUpEvent, TouchEnd, TouchStart,
};
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, window, Element, IEventTarget, TypedArray};
use stdweb::Value;

struct EmulatorState {
    gameboy: Gameboy,
    screen: Screen,
    controller_receiver: mpsc::Receiver<ControllerEvent>,
    should_save_to_local: Rc<RefCell<bool>>,
    ram_str: Rc<RefCell<String>>,
    js_ctx: Value,
    busy: bool,
    audio_underrun: Option<usize>,
}

impl EmulatorState {
    pub fn emulate_until_vblank_or_audio(&mut self) -> StepResult {
        let step_result = loop {
            let step_result = self.gameboy.emulate(&mut self.screen);
            match step_result {
                StepResult::VBlank | StepResult::AudioBufferFull => {
                    break step_result;
                }
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

    // most of this code has been copied from
    // https://github.com/koute/pinky
    pub fn play_audio(&mut self) {
        let audio_buffer = self.gameboy.get_audio_buffer();

        let audio_buffered: f64 = js! {
            let h = @{&self.js_ctx};
            var samples = @{TypedArray::<f32>::from(audio_buffer)};
            var sampleRate = 44100;
            var sampleCount = 4096;
            var latency = 0.032;

            var audioBuffer;
            if (h.emptyAudioBuffers.length === 0) {
                audioBuffer = h.audio.createBuffer(2, sampleCount, sampleRate * 2);
            } else {
                audioBuffer = h.emptyAudioBuffers.pop();
            }

            audioBuffer.getChannelData(0).set(samples);

            var node = h.audio.createBufferSource();
            node.connect(h.audio.destination);
            node.buffer = audioBuffer;
            node.onended = function() {
                h.emptyAudioBuffers.push(audioBuffer);
            };

            var buffered = h.playTimestamp - (h.audio.currentTime + latency);
            var playTimestamp = Math.max(h.audio.currentTime + latency, h.playTimestamp);
            node.start(playTimestamp);
            h.playTimestamp = playTimestamp + sampleCount / 2 / sampleRate;

            return buffered;
        }
        .try_into()
        .unwrap();

        if audio_buffered < 0.000 {
            self.audio_underrun = Some(std::cmp::max(self.audio_underrun.unwrap_or(0), 3));
        } else if audio_buffered < 0.010 {
            self.audio_underrun = Some(std::cmp::max(self.audio_underrun.unwrap_or(0), 2));
        } else if audio_buffered < 0.020 {
            self.audio_underrun = Some(std::cmp::max(self.audio_underrun.unwrap_or(0), 1));
        }
    }

    pub fn render(&self) {
        let frame_buffer: &[u8] = self.screen.get_frame_buffer();
        js! {
            var h = @{&self.js_ctx};
            var frame_buffer = @{TypedArray::<u8>::from(frame_buffer)};
            h.buffer.set(frame_buffer);
            createImageBitmap(h.img).then(function (bitmap) {
                h.ctx.drawImage(bitmap, 0, 0, 160, 144, 0, 0, h.canvas.width, h.canvas.height);
            });
        }
    }

    pub fn save_ram_data(&mut self) {
        if *self.should_save_to_local.borrow() && self.gameboy.get_cartridge().has_battery() {
            let name = self.gameboy.get_cartridge().get_name();
            window()
                .local_storage()
                .insert(&name, &self.ram_str.borrow())
                .unwrap();
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
            window()
                .local_storage()
                .insert(&name, &timestamp_data_str)
                .unwrap();
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
pub fn start(rom: Vec<u8>, dom_ids: DOMInfo) {
    let (sender, receiver) = mpsc::channel();
    let should_save_to_local = Rc::new(RefCell::new(false));

    let up_btn = document()
        .get_element_by_id(dom_ids.up_button_id.as_str())
        .unwrap();
    let down_btn = document()
        .get_element_by_id(dom_ids.down_button_id.as_str())
        .unwrap();
    let left_btn = document()
        .get_element_by_id(dom_ids.left_button_id.as_str())
        .unwrap();
    let right_btn = document()
        .get_element_by_id(dom_ids.right_button_id.as_str())
        .unwrap();
    let up_left_btn = document()
        .get_element_by_id(dom_ids.up_left_button_id.as_str())
        .unwrap();
    let up_right_btn = document()
        .get_element_by_id(dom_ids.up_right_button_id.as_str())
        .unwrap();
    let down_left_btn = document()
        .get_element_by_id(dom_ids.down_left_button_id.as_str())
        .unwrap();
    let down_right_btn = document()
        .get_element_by_id(dom_ids.down_right_button_id.as_str())
        .unwrap();
    let a_btn = document()
        .get_element_by_id(dom_ids.a_button_id.as_str())
        .unwrap();
    let b_btn = document()
        .get_element_by_id(dom_ids.b_button_id.as_str())
        .unwrap();
    let start_btn = document()
        .get_element_by_id(dom_ids.start_button_id.as_str())
        .unwrap();
    let select_btn = document()
        .get_element_by_id(dom_ids.select_button_id.as_str())
        .unwrap();

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

    {
        let sender = sender.clone();
        window().add_event_listener(move |event: KeyDownEvent| {
            let _send_result = match event.key().as_ref() {
                "ArrowUp" => Some(sender.send(ControllerEvent::Pressed(Button::Up))),
                "ArrowDown" => Some(sender.send(ControllerEvent::Pressed(Button::Down))),
                "ArrowLeft" => Some(sender.send(ControllerEvent::Pressed(Button::Left))),
                "ArrowRight" => Some(sender.send(ControllerEvent::Pressed(Button::Right))),
                "z" => Some(sender.send(ControllerEvent::Pressed(Button::A))),
                "x" => Some(sender.send(ControllerEvent::Pressed(Button::B))),
                "Enter" => Some(sender.send(ControllerEvent::Pressed(Button::Select))),
                " " => Some(sender.send(ControllerEvent::Pressed(Button::Start))),
                _ => None,
            };
        });
    }

    window().add_event_listener(move |event: KeyUpEvent| {
        let _send_result = match event.key().as_ref() {
            "ArrowUp" => Some(sender.send(ControllerEvent::Released(Button::Up))),
            "ArrowDown" => Some(sender.send(ControllerEvent::Released(Button::Down))),
            "ArrowLeft" => Some(sender.send(ControllerEvent::Released(Button::Left))),
            "ArrowRight" => Some(sender.send(ControllerEvent::Released(Button::Right))),
            "z" => Some(sender.send(ControllerEvent::Released(Button::A))),
            "x" => Some(sender.send(ControllerEvent::Released(Button::B))),
            "Enter" => Some(sender.send(ControllerEvent::Released(Button::Select))),
            " " => Some(sender.send(ControllerEvent::Released(Button::Start))),
            _ => None,
        };
    });

    let canvas: CanvasElement = document()
        .get_element_by_id(dom_ids.canvas_id.as_str())
        .unwrap()
        .try_into()
        .unwrap();

    let js_ctx = js! {
        var h = {};
        var canvas = @{canvas};
        h.canvas = canvas;
        h.ctx = canvas.getContext("2d");
        h.ctx.imageSmoothingEnabled = false;
        h.img = h.ctx.createImageData(160, 144);
        h.buffer = new Uint8Array(h.img.data.buffer);
        h.audio = new AudioContext();
        h.emptyAudioBuffers = [];
        h.playTimestamp = 0;
        return h;
    };
    let rtc = Box::new(WebRTC::new());
    let mut gameboy = Gameboy::from_rom(rom, rtc);
    load_ram_save_data(gameboy.get_cartridge_mut());
    load_timestamp_data(gameboy.get_cartridge_mut());
    let ram = gameboy.get_cartridge().get_ram().to_vec();
    let ram_str = Rc::new(RefCell::new(
        ram.iter().map(|byte| format!("{:02x}", byte)).collect(),
    ));
    let screen = Screen::new();

    let mut emulator_state = EmulatorState {
        //from opengl_web
        gameboy,
        screen,
        controller_receiver: receiver,
        should_save_to_local,
        ram_str,
        js_ctx,
        busy: false,
        audio_underrun: None,
    };

    emulator_state.set_ram_change_listener();
    main_loop(Rc::new(RefCell::new(emulator_state)));
}

fn add_button_event_listeners(
    element: &Element,
    button: Button,
    sender: mpsc::Sender<ControllerEvent>,
) {
    add_controller_event_listener::<MouseDownEvent>(
        element,
        ControllerEvent::Pressed(button),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        element,
        ControllerEvent::Released(button),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        element,
        ControllerEvent::Pressed(button),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(element, ControllerEvent::Released(button), sender);
}

fn add_controller_event_listener<T: ConcreteEvent>(
    element: &Element,
    controller_event: ControllerEvent,
    sender: mpsc::Sender<ControllerEvent>,
) {
    element.add_event_listener(move |_: T| {
        sender.send(controller_event).unwrap();
    });
}

fn add_multi_button_event_listeners(
    element: &Element,
    first_button: Button,
    second_button: Button,
    sender: mpsc::Sender<ControllerEvent>,
) {
    add_multi_controller_event_listener::<MouseDownEvent>(
        element,
        ControllerEvent::Pressed(first_button),
        ControllerEvent::Pressed(second_button),
        sender.clone(),
    );
    add_multi_controller_event_listener::<MouseUpEvent>(
        element,
        ControllerEvent::Released(first_button),
        ControllerEvent::Released(second_button),
        sender.clone(),
    );
    add_multi_controller_event_listener::<TouchStart>(
        element,
        ControllerEvent::Pressed(first_button),
        ControllerEvent::Pressed(second_button),
        sender.clone(),
    );
    add_multi_controller_event_listener::<TouchEnd>(
        element,
        ControllerEvent::Released(first_button),
        ControllerEvent::Released(second_button),
        sender,
    );
}

fn add_multi_controller_event_listener<T: ConcreteEvent>(
    element: &Element,
    first_controller_event: ControllerEvent,
    second_controller_event: ControllerEvent,
    sender: mpsc::Sender<ControllerEvent>,
) {
    element.add_event_listener(move |_: T| {
        sender.send(first_controller_event).unwrap();
        sender.send(second_controller_event).unwrap();
    });
}

fn load_ram_save_data(cartridge: &mut Cartridge) {
    if let Some(ram_str) = window().local_storage().get(cartridge.get_name()) {
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
    let key = format!("{}-timestamp", cartridge.get_name());
    if let Some(timestamp_str) = window().local_storage().get(&key) {
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

fn main_loop(emulator_state: Rc<RefCell<EmulatorState>>) {
    if !emulator_state.borrow().busy {
        emulate_a_single_frame(emulator_state.clone());
    }

    emulator_state.borrow_mut().render();
    emulator_state.borrow_mut().save_ram_data();
    emulator_state.borrow_mut().save_timestamp_data();
    window().request_animation_frame(move |_| {
        main_loop(emulator_state);
    });
}

fn emulate_a_single_frame(emulator_state: Rc<RefCell<EmulatorState>>) {
    emulator_state.borrow_mut().busy = true;

    stdweb::web::set_timeout(
        move || {
            let step_result = emulator_state.borrow_mut().emulate_until_vblank_or_audio();
            match step_result {
                StepResult::AudioBufferFull => {
                    stdweb::web::set_timeout(move || emulate_a_single_frame(emulator_state), 0);
                }
                StepResult::VBlank => {
                    let mut emulator_state = emulator_state.borrow_mut();
                    if let Some(count) = emulator_state.audio_underrun.take() {
                        for _ in 0..count {
                            emulator_state.emulate_until_vblank_or_audio();
                        }
                    }
                    emulator_state.busy = false;
                }
                StepResult::Nothing => {}
            };
        },
        0,
    );
}
