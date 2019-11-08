#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb_derive;
extern crate gameboy_core;

mod screen;
mod webgl_rendering_context;

use gameboy_core::{Button, Cartridge, Controller, Emulator};
use screen::Screen;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::event::{
    BeforeUnloadEvent, ConcreteEvent, KeyDownEvent, KeyUpEvent, MouseDownEvent, MouseUpEvent,
    TouchEnd, TouchStart,
};
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, window, Element, IEventTarget, TypedArray};
use webgl_rendering_context::*;

type Gl = WebGLRenderingContext;

const VERTEX_SOURCE: &str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SOURCE: &str = include_str!("shaders/fragment.glsl");
const VERTICIES: [f32; 12] = [
    1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 1.0, 0.0,
];
const TEXTURE_COORDINATE: [f32; 8] = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
const INDICIES: [u8; 6] = [0, 1, 3, 1, 2, 3];

#[derive(Copy, Clone)]
enum ControllerEvent {
    Pressed(Button),
    Released(Button),
}

pub fn start(rom: Vec<u8>) {
    let (sender, receiver) = mpsc::channel::<ControllerEvent>();
    let (end_sender, end_receiver) = mpsc::channel::<bool>();

    let up_btn = document().get_element_by_id("up-btn").unwrap();
    let down_btn = document().get_element_by_id("down-btn").unwrap();
    let left_btn = document().get_element_by_id("left-btn").unwrap();
    let right_btn = document().get_element_by_id("right-btn").unwrap();
    let a_btn = document().get_element_by_id("a-btn").unwrap();
    let b_btn = document().get_element_by_id("b-btn").unwrap();
    let start_btn = document().get_element_by_id("start-btn").unwrap();
    let select_btn = document().get_element_by_id("select-btn").unwrap();

    add_controller_event_listener::<MouseDownEvent>(
        &up_btn,
        ControllerEvent::Pressed(Button::Up),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        &up_btn,
        ControllerEvent::Released(Button::Up),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        &up_btn,
        ControllerEvent::Pressed(Button::Up),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(
        &up_btn,
        ControllerEvent::Released(Button::Up),
        sender.clone(),
    );

    add_controller_event_listener::<MouseDownEvent>(
        &down_btn,
        ControllerEvent::Pressed(Button::Down),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        &down_btn,
        ControllerEvent::Released(Button::Down),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        &down_btn,
        ControllerEvent::Pressed(Button::Down),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(
        &down_btn,
        ControllerEvent::Released(Button::Down),
        sender.clone(),
    );

    add_controller_event_listener::<MouseDownEvent>(
        &left_btn,
        ControllerEvent::Pressed(Button::Left),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        &left_btn,
        ControllerEvent::Released(Button::Left),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        &left_btn,
        ControllerEvent::Pressed(Button::Left),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(
        &left_btn,
        ControllerEvent::Released(Button::Left),
        sender.clone(),
    );

    add_controller_event_listener::<MouseDownEvent>(
        &right_btn,
        ControllerEvent::Pressed(Button::Right),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        &right_btn,
        ControllerEvent::Released(Button::Right),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        &right_btn,
        ControllerEvent::Pressed(Button::Right),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(
        &right_btn,
        ControllerEvent::Released(Button::Right),
        sender.clone(),
    );

    add_controller_event_listener::<MouseDownEvent>(
        &a_btn,
        ControllerEvent::Pressed(Button::A),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        &a_btn,
        ControllerEvent::Released(Button::A),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        &a_btn,
        ControllerEvent::Pressed(Button::A),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(
        &a_btn,
        ControllerEvent::Released(Button::A),
        sender.clone(),
    );

    add_controller_event_listener::<MouseDownEvent>(
        &b_btn,
        ControllerEvent::Pressed(Button::B),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        &b_btn,
        ControllerEvent::Released(Button::B),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        &b_btn,
        ControllerEvent::Pressed(Button::B),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(
        &b_btn,
        ControllerEvent::Released(Button::B),
        sender.clone(),
    );

    add_controller_event_listener::<MouseDownEvent>(
        &start_btn,
        ControllerEvent::Pressed(Button::Start),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        &start_btn,
        ControllerEvent::Released(Button::Start),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        &start_btn,
        ControllerEvent::Pressed(Button::Start),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(
        &start_btn,
        ControllerEvent::Released(Button::Start),
        sender.clone(),
    );

    add_controller_event_listener::<MouseDownEvent>(
        &select_btn,
        ControllerEvent::Pressed(Button::Select),
        sender.clone(),
    );
    add_controller_event_listener::<MouseUpEvent>(
        &select_btn,
        ControllerEvent::Released(Button::Select),
        sender.clone(),
    );
    add_controller_event_listener::<TouchStart>(
        &select_btn,
        ControllerEvent::Pressed(Button::Select),
        sender.clone(),
    );
    add_controller_event_listener::<TouchEnd>(
        &select_btn,
        ControllerEvent::Released(Button::Select),
        sender.clone(),
    );

    let canvas: CanvasElement = document()
        .get_element_by_id("canvas")
        .unwrap()
        .try_into()
        .unwrap();

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

    {
        let sender = sender.clone();
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
    }

    let gl: Gl = canvas.get_context().unwrap();

    gl.clear_color(1.0, 0.0, 0.0, 1.0);
    gl.clear(Gl::COLOR_BUFFER_BIT);

    let verticies = TypedArray::<f32>::from(VERTICIES.as_ref()).buffer();
    let vertex_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_1(Gl::ARRAY_BUFFER, Some(&verticies), Gl::STATIC_DRAW);

    let textures = TypedArray::<f32>::from(TEXTURE_COORDINATE.as_ref()).buffer();
    let texture_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&texture_buffer));
    gl.buffer_data_1(Gl::ARRAY_BUFFER, Some(&textures), Gl::STATIC_DRAW);

    let indicies = TypedArray::<u8>::from(INDICIES.as_ref()).buffer();
    let index_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    gl.buffer_data_1(Gl::ELEMENT_ARRAY_BUFFER, Some(&indicies), Gl::STATIC_DRAW);

    let vert_shader = match compile_shader(&gl, Gl::VERTEX_SHADER, VERTEX_SOURCE) {
        Ok(shader) => shader,
        Err(msg) => panic!(msg),
    };

    let frag_shader = match compile_shader(&gl, Gl::FRAGMENT_SHADER, FRAGMENT_SOURCE) {
        Ok(shader) => shader,
        Err(msg) => panic!(msg),
    };

    let shader_program = link_program(&gl, &vert_shader, &frag_shader);

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

    let mut cartridge = Cartridge::from_rom(rom);

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

    window().add_event_listener(move |_: BeforeUnloadEvent| {
        end_sender.send(true).unwrap();
    });

    let emulator = Emulator::from_cartridge(cartridge);
    let screen = Screen::new();
    let controller = Controller::new();

    main_loop(
        emulator,
        screen,
        controller,
        receiver,
        end_receiver,
        gl,
        shader_program,
        texture,
    );
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

fn compile_shader(gl: &Gl, shader_type: GLenum, source: &str) -> Result<WebGLShader, String> {
    let shader = gl.create_shader(shader_type).unwrap();
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    let compiled = gl.get_shader_parameter(&shader, Gl::COMPILE_STATUS);

    if compiled == stdweb::Value::Bool(false) {
        let error = gl.get_shader_info_log(&shader);
        Err(error.unwrap_or_else(|| "Unknown compilation error".to_string()))
    } else {
        Ok(shader)
    }
}

fn link_program(gl: &Gl, vert_shader: &WebGLShader, frag_shader: &WebGLShader) -> WebGLProgram {
    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, vert_shader);
    gl.attach_shader(&shader_program, frag_shader);
    gl.link_program(&shader_program);
    shader_program
}

// since closure's in Rust can't be recursive, this function has too many args
#[allow(clippy::too_many_arguments)]
fn main_loop(
    mut emulator: Emulator,
    mut screen: Screen,
    mut controller: Controller,
    receiver: mpsc::Receiver<ControllerEvent>,
    end_receiver: mpsc::Receiver<bool>,
    gl: Gl,
    shader_program: WebGLProgram,
    texture: WebGLTexture,
) {
    let mut run = true;

    loop {
        let vblank = emulator.emulate(&mut screen, &mut controller);
        if vblank {
            break;
        }
    }

    loop {
        match receiver.try_recv() {
            Ok(ControllerEvent::Pressed(button)) => controller.press(button),
            Ok(ControllerEvent::Released(button)) => controller.release(button),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                run = false;
                break;
            }
        }
    }
    let frame_buffer = screen.get_frame_buffer();
    render(&gl, &shader_program, &texture, frame_buffer.as_ref());

    match end_receiver.try_recv() {
        Ok(_) | Err(TryRecvError::Disconnected) => run = false,
        _ => (),
    };
    if run {
        window().request_animation_frame(move |_| {
            main_loop(
                emulator,
                screen,
                controller,
                receiver,
                end_receiver,
                gl,
                shader_program,
                texture,
            );
        });
    } else {
        let cartridge = emulator.get_cartridge();
        let ram = cartridge.get_ram().to_vec();
        let name = cartridge.get_name().to_string();
        let has_battery = cartridge.has_battery();
        if has_battery {
            let ram_str: String = ram.iter().map(|byte| format!("{:02x}", byte)).collect();
            window()
                .local_storage()
                .insert(name.as_ref(), &ram_str)
                .unwrap();
        }
    }
}

fn render(gl: &Gl, shader_program: &WebGLProgram, texture: &WebGLTexture, frame_buffer: &[u8]) {
    gl.bind_texture(Gl::TEXTURE_2D, Some(texture));
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
    gl.use_program(Some(shader_program));
    let screen_uniform = gl.get_uniform_location(&shader_program, "screen").unwrap();
    gl.uniform1i(Some(&screen_uniform), 0);
    gl.draw_elements(Gl::TRIANGLES, 6, Gl::UNSIGNED_BYTE, 0);
}
