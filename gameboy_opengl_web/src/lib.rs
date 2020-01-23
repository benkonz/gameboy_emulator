#![recursion_limit = "2048"]

#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb_derive;
extern crate gameboy_core;

mod gl_utils;
mod save_utils;
mod screen;
mod web_rtc;
mod webgl_rendering_context;

use gameboy_core::{Button, Cartridge, Controller, ControllerEvent, Emulator, StepResult};
use screen::Screen;
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
use stdweb::{UnsafeTypedArray, Value};
use web_rtc::WebRTC;
use webgl_rendering_context::WebGLRenderingContext;
use webgl_rendering_context::*;

type Gl = WebGLRenderingContext;

struct EmulatorState {
    emulator: Emulator,
    controller: Controller,
    screen: Screen,
    controller_receiver: mpsc::Receiver<ControllerEvent>,
    should_save_to_local: Rc<RefCell<bool>>,
    ram_str: Rc<RefCell<String>>,
    gl: Gl,
    shader_program: WebGLProgram,
    texture: WebGLTexture,
    js_ctx: Value,
    busy: bool,
    audio_underrun: Option<usize>,
    audio_buffer: Vec<f32>,
}

impl EmulatorState {
    pub fn emulate_until_vblank_or_audio(&mut self) -> StepResult {
        let step_result = loop {
            let step_result = self
                .emulator
                .emulate(&mut self.screen, &mut self.controller);
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

        // save_utils::save_ram_data(
        //     &emulator,
        //     ram_str.clone(),
        //     should_save_to_local.clone(),
        // );
        // save_utils::save_timestamp_data(&emulator);
        loop {
            match self.controller_receiver.try_recv() {
                Ok(ControllerEvent::Pressed(button)) => self.controller.press(button),
                Ok(ControllerEvent::Released(button)) => self.controller.release(button),
                Err(_) => break,
            }
        }

        step_result
    }

    pub fn render(&self) {
        let gl = &self.gl;
        let frame_buffer: &[u8] = self.screen.get_frame_buffer();
        gl.bind_texture(Gl::TEXTURE_2D, Some(&self.texture));
        gl.tex_image2_d(
            Gl::TEXTURE_2D,
            0,
            Gl::RGB as i32,
            160,
            144,
            0,
            Gl::RGB,
            Gl::UNSIGNED_BYTE,
            Some(frame_buffer),
        );
        gl.active_texture(Gl::TEXTURE0);
        gl.use_program(Some(&self.shader_program));
        let screen_uniform = gl
            .get_uniform_location(&self.shader_program, "screen")
            .unwrap();
        gl.uniform1i(Some(&screen_uniform), 0);
        gl.draw_elements(Gl::TRIANGLES, 6, Gl::UNSIGNED_BYTE, 0);
    }

    pub fn play_audio(&mut self) {
        let audio_buffer = self.emulator.get_audio_buffer();
        self.audio_buffer = vec![0.0; audio_buffer.len()];
        self.audio_buffer.copy_from_slice(audio_buffer);

        let audio_buffered: f64 = js! {
            let h = @{&self.js_ctx};
            var samples = @{unsafe { UnsafeTypedArray::new(&self.audio_buffer) }};
            var sampleRate = 44100;
            var sampleCount = samples.length;
            var latency = 0.032;

            var audioBuffer;
            if (h.emptyAudioBuffers.length === 0) {
                audioBuffer = h.audio.createBuffer(2, sampleCount, sampleRate);
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

        self.audio_buffer.clear();
    }
}

pub fn start(rom: Vec<u8>) {
    let (sender, receiver) = mpsc::channel();
    let should_save_to_local = Rc::new(RefCell::new(false));

    let up_btn = document().get_element_by_id("up-btn").unwrap();
    let down_btn = document().get_element_by_id("down-btn").unwrap();
    let left_btn = document().get_element_by_id("left-btn").unwrap();
    let right_btn = document().get_element_by_id("right-btn").unwrap();
    let up_left_btn = document().get_element_by_id("up-left-btn").unwrap();
    let up_right_btn = document().get_element_by_id("up-right-btn").unwrap();
    let down_left_btn = document().get_element_by_id("down-left-btn").unwrap();
    let down_right_btn = document().get_element_by_id("down-right-btn").unwrap();
    let a_btn = document().get_element_by_id("a-btn").unwrap();
    let b_btn = document().get_element_by_id("b-btn").unwrap();
    let start_btn = document().get_element_by_id("start-btn").unwrap();
    let select_btn = document().get_element_by_id("select-btn").unwrap();

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
        .get_element_by_id("canvas")
        .unwrap()
        .try_into()
        .unwrap();

    let gl: Gl = canvas.get_context().unwrap();
    gl.clear_color(1.0, 0.0, 0.0, 1.0);
    gl.clear(Gl::COLOR_BUFFER_BIT);

    let verticies: [f32; 12] = [
        1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 1.0, 0.0,
    ];
    let vertex_array = TypedArray::<f32>::from(verticies.as_ref()).buffer();
    let vertex_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_1(Gl::ARRAY_BUFFER, Some(&vertex_array), Gl::STATIC_DRAW);

    let texture_coordinate: [f32; 8] = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
    let texture_array = TypedArray::<f32>::from(texture_coordinate.as_ref()).buffer();
    let texture_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&texture_buffer));
    gl.buffer_data_1(Gl::ARRAY_BUFFER, Some(&texture_array), Gl::STATIC_DRAW);

    let indicies: [u8; 6] = [0, 1, 3, 1, 2, 3];
    let indicies_array = TypedArray::<u8>::from(indicies.as_ref()).buffer();
    let index_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    gl.buffer_data_1(
        Gl::ELEMENT_ARRAY_BUFFER,
        Some(&indicies_array),
        Gl::STATIC_DRAW,
    );

    let vertex_source: &str = include_str!("shaders/vertex.glsl");
    let vert_shader = gl_utils::compile_shader(&gl, Gl::VERTEX_SHADER, vertex_source).unwrap();

    let fragment_source: &str = include_str!("shaders/fragment.glsl");
    let frag_shader = gl_utils::compile_shader(&gl, Gl::FRAGMENT_SHADER, fragment_source).unwrap();

    let shader_program = gl_utils::link_program(&gl, &vert_shader, &frag_shader);

    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));
    let pos_attr = gl.get_attrib_location(&shader_program, "aPos") as u32;
    gl.vertex_attrib_pointer(pos_attr, 3, Gl::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(pos_attr);

    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&texture_buffer));
    let tex_attr = gl.get_attrib_location(&shader_program, "aTexCoord") as u32;
    gl.vertex_attrib_pointer(tex_attr, 2, Gl::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(tex_attr);

    let texture = gl.create_texture().unwrap();
    gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));

    gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::NEAREST as i32);
    gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::NEAREST as i32);
    gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);

    let js_ctx = js! {
        var h = {};
        h.audio = new AudioContext();
        h.emptyAudioBuffers = [];
        h.playTimestamp = 0;
        return h;
    };

    let mut cartridge = Cartridge::from_rom(rom);
    save_utils::load_ram_save_data(&mut cartridge);
    save_utils::load_timestamp_data(&mut cartridge);

    let rtc = Box::new(WebRTC::new());
    let mut emulator = Emulator::from_cartridge(cartridge, rtc);
    let screen = Screen::new();
    let controller = Controller::new();

    let ram = emulator.get_cartridge().get_ram().to_vec();
    let ram_str = Rc::new(RefCell::new(
        ram.iter().map(|byte| format!("{:02x}", byte)).collect(),
    ));
    set_ram_change_listener(&mut emulator, ram_str.clone(), should_save_to_local.clone());

    let emulator_state = EmulatorState {
        emulator,
        controller,
        screen,
        controller_receiver: receiver,
        should_save_to_local,
        ram_str,
        gl,
        shader_program,
        texture,
        js_ctx,
        busy: false,
        audio_underrun: None,
        audio_buffer: Vec::with_capacity(44100),
    };

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

fn set_ram_change_listener(
    emulator: &mut Emulator,
    ram_str: Rc<RefCell<String>>,
    should_save_to_local: Rc<RefCell<bool>>,
) {
    let has_battery = emulator.get_cartridge().has_battery();
    emulator.set_ram_change_callback(Box::new(move |address, value| {
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

fn main_loop(emulator_state: Rc<RefCell<EmulatorState>>) {
    if !emulator_state.borrow().busy {
        emulate_a_single_frame(emulator_state.clone());
    }

    emulator_state.borrow_mut().render();
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
