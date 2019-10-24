#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb_derive;
extern crate gameboy_core;

mod screen;
mod webgl_rendering_context;

use gameboy_core::Emulator;
use screen::Screen;
use stdweb::web::window;

use gameboy_core::Button;
use gameboy_core::Controller;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use stdweb::traits::IKeyboardEvent;
use stdweb::unstable::TryInto;
use stdweb::web::event::{KeyDownEvent, KeyUpEvent, MouseDownEvent, MouseUpEvent};
use stdweb::web::html_element::CanvasElement;
use stdweb::web::IWindowOrWorker;
use stdweb::web::{document, IEventTarget, IParentNode, TypedArray};
use webgl_rendering_context::*;

const VERTEX_SOURCE: &'static str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SOURCE: &'static str = include_str!("shaders/fragment.glsl");
const VERTICIES: [f32; 12] = [
    1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 1.0, 0.0,
];
const TEXTURE_COORDINATE: [f32; 8] = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
const INDICIES: [u8; 6] = [0, 1, 3, 1, 2, 3];

type Gl = WebGLRenderingContext;

enum ControllerEvent {
    Pressed(Button),
    Released(Button),
}

pub fn start(rom: Vec<u8>) {
    let screen = Screen::new();
    let emulator = Emulator::from_rom(rom);
    let controller = gameboy_core::Controller::new();
    let (sender, receiver) = mpsc::channel::<Vec<u8>>();
    let (sender2, receiver2) = mpsc::channel::<ControllerEvent>();
    // TODO: might be worth looking into making a third channel for "frame_rendered"
    // whenever the render-er finishes, send something over the channel
    // in the emulator loop, first check if the queue is empty, if not do teh computation, else just skip it

    stdweb::initialize();

    let up_btn = document().query_selector("#up-btn").unwrap().unwrap();
    let down_btn = document().query_selector("#down-btn").unwrap().unwrap();
    let left_btn = document().query_selector("#left-btn").unwrap().unwrap();
    let right_btn = document().query_selector("#right-btn").unwrap().unwrap();
    let a_btn = document().query_selector("#a-btn").unwrap().unwrap();
    let b_btn = document().query_selector("#b-btn").unwrap().unwrap();
    let start_btn = document().query_selector("#start-btn").unwrap().unwrap();
    let select_btn = document().query_selector("#select-btn").unwrap().unwrap();

    {
        let sender2 = sender2.clone();
        up_btn.add_event_listener(move |_: MouseDownEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Up));
        });
    }
    {
        let sender2 = sender2.clone();
        up_btn.add_event_listener(move |_: MouseUpEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Up));
        });
    }

    {
        let sender2 = sender2.clone();
        down_btn.add_event_listener(move |_: MouseDownEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Down));
        });
    }
    {
        let sender2 = sender2.clone();
        down_btn.add_event_listener(move |_: MouseUpEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Down));
        });
    }

    {
        let sender2 = sender2.clone();
        left_btn.add_event_listener(move |_: MouseDownEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Left));
        });
    }
    {
        let sender2 = sender2.clone();
        left_btn.add_event_listener(move |_: MouseUpEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Left));
        });
    }

    {
        let sender2 = sender2.clone();
        right_btn.add_event_listener(move |_: MouseDownEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Right));
        });
    }
    {
        let sender2 = sender2.clone();
        right_btn.add_event_listener(move |_: MouseUpEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Right));
        });
    }

    {
        let sender2 = sender2.clone();
        a_btn.add_event_listener(move |_: MouseDownEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::A));
        });
    }
    {
        let sender2 = sender2.clone();
        a_btn.add_event_listener(move |_: MouseUpEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::A));
        });
    }

    {
        let sender2 = sender2.clone();
        b_btn.add_event_listener(move |_: MouseDownEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::B));
        });
    }
    {
        let sender2 = sender2.clone();
        b_btn.add_event_listener(move |_: MouseUpEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::B));
        });
    }

    {
        let sender2 = sender2.clone();
        start_btn.add_event_listener(move |_: MouseDownEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Start));
        });
    }
    {
        let sender2 = sender2.clone();
        start_btn.add_event_listener(move |_: MouseUpEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Start));
        });
    }

     {
        let sender2 = sender2.clone();
        select_btn.add_event_listener(move |_: MouseDownEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Select));
        });
    }
    {
        let sender2 = sender2.clone();
        select_btn.add_event_listener(move |_: MouseUpEvent| {
            let _ = sender2.send(ControllerEvent::Pressed(Button::Select));
        });
    }

    // TODO: add touch listeners

    let canvas: CanvasElement = document()
        .query_selector("#canvas")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();

    {
        let sender2 = sender2.clone();
        window().add_event_listener(move |event: KeyDownEvent| {
            let _send_result = match event.key().as_ref() {
                "ArrowUp" => Some(sender2.send(ControllerEvent::Pressed(Button::Up))),
                "ArrowDown" => Some(sender2.send(ControllerEvent::Pressed(Button::Down))),
                "ArrowLeft" => Some(sender2.send(ControllerEvent::Pressed(Button::Left))),
                "ArrowRight" => Some(sender2.send(ControllerEvent::Pressed(Button::Right))),
                "z" => Some(sender2.send(ControllerEvent::Pressed(Button::A))),
                "x" => Some(sender2.send(ControllerEvent::Pressed(Button::B))),
                "Enter" => Some(sender2.send(ControllerEvent::Pressed(Button::Select))),
                " " => Some(sender2.send(ControllerEvent::Pressed(Button::Start))),
                _ => None,
            };
        });
    }

    {
        let sender2 = sender2.clone();
        window().add_event_listener(move |event: KeyUpEvent| {
            let _send_result = match event.key().as_ref() {
                "ArrowUp" => Some(sender2.send(ControllerEvent::Released(Button::Up))),
                "ArrowDown" => Some(sender2.send(ControllerEvent::Released(Button::Down))),
                "ArrowLeft" => Some(sender2.send(ControllerEvent::Released(Button::Left))),
                "ArrowRight" => Some(sender2.send(ControllerEvent::Released(Button::Right))),
                "z" => Some(sender2.send(ControllerEvent::Released(Button::A))),
                "x" => Some(sender2.send(ControllerEvent::Released(Button::B))),
                "Enter" => Some(sender2.send(ControllerEvent::Released(Button::Select))),
                " " => Some(sender2.send(ControllerEvent::Released(Button::Start))),
                _ => None,
            };
        });
    }

    let gl: Gl = canvas.get_context().unwrap();

    gl.clear_color(1.0, 0.0, 0.0, 1.0);
    gl.clear(Gl::COLOR_BUFFER_BIT);

    let verticies = TypedArray::<f32>::from(&VERTICIES[..]).buffer();
    let vertex_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_1(Gl::ARRAY_BUFFER, Some(&verticies), Gl::STATIC_DRAW);

    let textures = TypedArray::<f32>::from(&TEXTURE_COORDINATE[..]).buffer();
    let texture_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&texture_buffer));
    gl.buffer_data_1(Gl::ARRAY_BUFFER, Some(&textures), Gl::STATIC_DRAW);

    let indicies = TypedArray::<u8>::from(&INDICIES[..]).buffer();
    let index_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    gl.buffer_data_1(Gl::ELEMENT_ARRAY_BUFFER, Some(&indicies), Gl::STATIC_DRAW);

    let vert_shader = gl.create_shader(Gl::VERTEX_SHADER).unwrap();
    gl.shader_source(&vert_shader, VERTEX_SOURCE);
    gl.compile_shader(&vert_shader);

    let compiled = gl.get_shader_parameter(&vert_shader, Gl::COMPILE_STATUS);

    if compiled == stdweb::Value::Bool(false) {
        let error = gl.get_shader_info_log(&vert_shader);
        if let Some(e) = error {
            console!(log, e);
        }
    }

    let frag_shader = gl.create_shader(Gl::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, FRAGMENT_SOURCE);
    gl.compile_shader(&frag_shader);

    let compiled = gl.get_shader_parameter(&frag_shader, Gl::COMPILE_STATUS);

    if compiled == stdweb::Value::Bool(false) {
        let error = gl.get_shader_info_log(&frag_shader);
        if let Some(e) = error {
            console!(log, e);
        }
    }

    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, &vert_shader);
    gl.attach_shader(&shader_program, &frag_shader);
    gl.link_program(&shader_program);

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

    emulator_loop(emulator, screen, controller, sender, receiver2);
    render_loop(gl, shader_program, texture, receiver);
}

fn emulator_loop(
    mut emulator: Emulator,
    mut screen: Screen,
    mut controller: Controller,
    sender: mpsc::Sender<Vec<u8>>,
    receiver: mpsc::Receiver<ControllerEvent>,
) {
    loop {
        let vblank = emulator.emulate(&mut screen, &mut controller);
        if vblank {
            break;
        }
    }

    match sender.send(screen.get_frame_buffer().clone()) {
        Ok(()) => {
            loop {
                match receiver.try_recv() {
                    Ok(ControllerEvent::Pressed(button)) => controller.press(button),
                    Ok(ControllerEvent::Released(button)) => controller.release(button),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => (),
                }
            }
            window().set_timeout(
                move || emulator_loop(emulator, screen, controller, sender, receiver),
                18, // wait 18msc to allow for controller inputs to queue up
            );
        }
        Err(_) => (),
    };
}

fn render_loop(
    gl: Gl,
    shader_program: WebGLProgram,
    texture: WebGLTexture,
    receiver: mpsc::Receiver<Vec<u8>>,
) {
    match receiver.try_recv() {
        Ok(frame_buffer) => {
            gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));

            let pixels = &frame_buffer[..];

            gl.tex_image2_d(
                Gl::TEXTURE_2D,
                0,
                Gl::RGB as i32,
                160,
                144,
                0,
                Gl::RGB,
                Gl::UNSIGNED_BYTE,
                Some(pixels.as_ref()),
            );

            gl.active_texture(Gl::TEXTURE0);

            gl.use_program(Some(&shader_program));

            let screen_uniform = gl.get_uniform_location(&shader_program, "screen").unwrap();
            gl.uniform1i(Some(&screen_uniform), 0);

            gl.draw_elements(Gl::TRIANGLES, 6, Gl::UNSIGNED_BYTE, 0);

            window().request_animation_frame(move |_| {
                render_loop(gl, shader_program, texture, receiver)
            });
        }
        Err(TryRecvError::Empty) => {
            window().request_animation_frame(move |_| {
                render_loop(gl, shader_program, texture, receiver)
            });
        }
        Err(TryRecvError::Disconnected) => (),
    }
}
