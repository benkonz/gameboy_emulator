#[macro_use]
extern crate c_str_macro;
extern crate directories;
extern crate gameboy_core;
extern crate glutin;

mod opengl_rendering_context;
mod screen;
mod shader;

use directories::BaseDirs;
use gameboy_core::{Button, Cartridge, Controller, Emulator};
use glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use opengl_rendering_context::types::*;
use opengl_rendering_context::Gl;
use screen::Screen;
use shader::Shader;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::raw::c_void;
use std::sync::mpsc;
use std::sync::mpsc::{SendError, TryRecvError};
use std::thread;
use std::time::{Duration, SystemTime};
use std::{mem, ptr};

enum ControllerEvent {
    Pressed(Button),
    Released(Button),
}

const VERTEX_SOURCE: &str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SOURCE: &str = include_str!("shaders/fragment.glsl");
const VERTICIES: [f32; 20] = [
    1.0, 1.0, 0.0, 1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.0,
    0.0, 1.0,
];
const INDICIES: [u32; 6] = [0, 1, 3, 1, 2, 3];

pub fn start(rom: Vec<u8>) {
    let events_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_title("Gameboy Emulator");

    let windowed_context = ContextBuilder::new()
        .build_windowed(window_builder, &events_loop)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let context = windowed_context.context();

    let gl = Gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    unsafe { gl.ClearColor(0.0, 1.0, 0.0, 1.0) };

    let shader = Shader::new(&gl, VERTEX_SOURCE, FRAGMENT_SOURCE);

    let (mut vao, mut vbo, mut ebo, mut texture) = (0, 0, 0, 0);

    unsafe {
        //setup vertex data
        gl.GenVertexArrays(1, &mut vao);
        gl.GenBuffers(1, &mut vbo);
        gl.GenBuffers(1, &mut ebo);

        gl.BindVertexArray(vao);

        gl.BindBuffer(opengl_rendering_context::ARRAY_BUFFER, vbo);
        gl.BufferData(
            opengl_rendering_context::ARRAY_BUFFER,
            (VERTICIES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            VERTICIES.as_ptr() as *const c_void,
            opengl_rendering_context::STATIC_DRAW,
        );

        gl.BindBuffer(opengl_rendering_context::ELEMENT_ARRAY_BUFFER, ebo);
        gl.BufferData(
            opengl_rendering_context::ELEMENT_ARRAY_BUFFER,
            (INDICIES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            INDICIES.as_ptr() as *const c_void,
            opengl_rendering_context::STATIC_DRAW,
        );

        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;

        //position attribute
        gl.VertexAttribPointer(
            0,
            3,
            opengl_rendering_context::FLOAT,
            opengl_rendering_context::FALSE,
            stride,
            ptr::null(),
        );
        gl.EnableVertexAttribArray(0);

        //texture attribute
        gl.VertexAttribPointer(
            1,
            2,
            opengl_rendering_context::FLOAT,
            opengl_rendering_context::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl.EnableVertexAttribArray(1);

        //load and create texture
        gl.GenTextures(1, &mut texture);
        gl.BindTexture(opengl_rendering_context::TEXTURE_2D, texture);

        //set texture wrapping params
        gl.TexParameteri(
            opengl_rendering_context::TEXTURE_2D,
            opengl_rendering_context::TEXTURE_WRAP_S,
            opengl_rendering_context::REPEAT as i32,
        );
        gl.TexParameteri(
            opengl_rendering_context::TEXTURE_2D,
            opengl_rendering_context::TEXTURE_WRAP_T,
            opengl_rendering_context::REPEAT as i32,
        );
        //set texture filtering params
        gl.TexParameteri(
            opengl_rendering_context::TEXTURE_2D,
            opengl_rendering_context::TEXTURE_MIN_FILTER,
            opengl_rendering_context::NEAREST as i32,
        );
        gl.TexParameteri(
            opengl_rendering_context::TEXTURE_2D,
            opengl_rendering_context::TEXTURE_MAG_FILTER,
            opengl_rendering_context::NEAREST as i32,
        );
    }

    let (controller_sender, controller_receiver) = mpsc::channel();
    let (frame_sender, frame_receiver) = mpsc::channel();
    let (end_sender, end_receiver) = mpsc::channel::<bool>();
    let (cartridge_sender, cartridge_receiver) = mpsc::channel::<(String, bool, Vec<u8>)>();

    thread::spawn(move || {
        let mut cartridge = Cartridge::from_rom(rom);

        if let Some(base_dirs) = BaseDirs::new() {
            let name = cartridge.get_name().to_string();
            let ram_save_file = base_dirs
                .config_dir()
                .join("gameboy_emulator")
                .join("ram_saves")
                .join(format!("{}.bin", name));

            if ram_save_file.exists() {
                let mut file = OpenOptions::new().read(true).open(ram_save_file).unwrap();
                if let Ok(metadata) = file.metadata() {
                    // sometimes two different roms have the same name,
                    // so we make sure that the ram length is the same before reading
                    if metadata.len() == cartridge.get_ram().len() as u64 {
                        file.read_exact(cartridge.get_ram_mut()).unwrap();
                    }
                }
            }
        }

        let mut emulator = Emulator::from_cartridge(cartridge);
        let mut controller = Controller::new();
        let mut mapper = Screen::new();
        let mut run = true;
        let frame_rate = 60f64;
        let frame_duration = Duration::from_secs_f64(1f64 / frame_rate);

        while run {
            let start_time = SystemTime::now();
            match end_receiver.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => run = false,
                _ => (),
            };

            loop {
                let vblank = emulator.emulate(&mut mapper, &mut controller);
                if vblank {
                    break;
                }
            }

            match frame_sender.send(mapper.get_frame_buffer().clone()) {
                Ok(()) => (),
                Err(SendError(_)) => run = false,
            };

            loop {
                match controller_receiver.try_recv() {
                    Ok(input) => match input {
                        ControllerEvent::Pressed(button) => controller.press(button),
                        ControllerEvent::Released(button) => controller.release(button),
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        run = false;
                        break;
                    }
                }
            }
            let end_time = SystemTime::now();
            let last_frame_duration = end_time.duration_since(start_time).unwrap();
            if frame_duration >= last_frame_duration {
                let sleep_duration = frame_duration - last_frame_duration;
                thread::sleep(sleep_duration);
            }
        }

        let cartridge = emulator.get_cartridge();
        let ram = cartridge.get_ram().to_vec();
        let name = cartridge.get_name().to_string();
        let has_battery = cartridge.has_battery();
        cartridge_sender.send((name, has_battery, ram)).unwrap();
    });

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    let result = match input {
                        // key pressed
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Up),
                            ..
                        } => controller_sender.send(ControllerEvent::Pressed(Button::Up)),
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Down),
                            ..
                        } => controller_sender.send(ControllerEvent::Pressed(Button::Down)),
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Left),
                            ..
                        } => controller_sender.send(ControllerEvent::Pressed(Button::Left)),
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Right),
                            ..
                        } => controller_sender.send(ControllerEvent::Pressed(Button::Right)),
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Z),
                            ..
                        } => controller_sender.send(ControllerEvent::Pressed(Button::A)),
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::X),
                            ..
                        } => controller_sender.send(ControllerEvent::Pressed(Button::B)),
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Return),
                            ..
                        } => controller_sender.send(ControllerEvent::Pressed(Button::Select)),
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        } => controller_sender.send(ControllerEvent::Pressed(Button::Start)),
                        // key released
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Up),
                            ..
                        } => controller_sender.send(ControllerEvent::Released(Button::Up)),
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Down),
                            ..
                        } => controller_sender.send(ControllerEvent::Released(Button::Down)),
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Left),
                            ..
                        } => controller_sender.send(ControllerEvent::Released(Button::Left)),
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Right),
                            ..
                        } => controller_sender.send(ControllerEvent::Released(Button::Right)),
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Z),
                            ..
                        } => controller_sender.send(ControllerEvent::Released(Button::A)),
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::X),
                            ..
                        } => controller_sender.send(ControllerEvent::Released(Button::B)),
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Return),
                            ..
                        } => controller_sender.send(ControllerEvent::Released(Button::Select)),
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        } => controller_sender.send(ControllerEvent::Released(Button::Start)),
                        _ => Ok(()),
                    };

                    if let Err(SendError(_)) = result {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }

        match frame_receiver.try_recv() {
            Ok(frame_buffer) => {
                draw_texture(&gl, texture, &shader, &frame_buffer);
                unsafe {
                    gl.BindVertexArray(vao);
                    gl.DrawElements(
                        opengl_rendering_context::TRIANGLES,
                        6,
                        opengl_rendering_context::UNSIGNED_INT,
                        ptr::null(),
                    );
                    gl.BindVertexArray(0);
                }
                windowed_context.swap_buffers().unwrap()
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => *control_flow = ControlFlow::Exit,
        };

        if *control_flow == ControlFlow::Exit && end_sender.send(true).is_ok() {
            if let Ok((name, has_battery, ram)) = cartridge_receiver.recv() {
                if has_battery {
                    if let Some(base_dir) = BaseDirs::new() {
                        let rom_path = base_dir
                            .config_dir()
                            .join("gameboy_emulator")
                            .join("ram_saves");
                        let ram_save_file = rom_path.join(format!("{}.bin", name));

                        if !rom_path.exists() {
                            fs::create_dir_all(rom_path).unwrap();
                        }

                        let mut file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(ram_save_file)
                            .unwrap();
                        file.write_all(ram.as_ref()).unwrap();
                    }
                }
                shader.delete_program(&gl);
            }
        }
    });
}

fn draw_texture(gl: &Gl, texture: GLuint, shader: &Shader, frame_buffer: &[u8]) {
    unsafe {
        gl.ClearColor(0.0, 0.0, 0.0, 1.0);
        gl.Clear(opengl_rendering_context::COLOR_BUFFER_BIT);
        gl.BindTexture(opengl_rendering_context::TEXTURE_2D, texture);
        gl.TexImage2D(
            opengl_rendering_context::TEXTURE_2D,
            0,
            opengl_rendering_context::RGB as i32,
            160,
            144,
            0,
            opengl_rendering_context::RGB,
            opengl_rendering_context::UNSIGNED_BYTE,
            frame_buffer.as_ptr() as *const c_void,
        );
        gl.GenerateMipmap(opengl_rendering_context::TEXTURE_2D);
        gl.ActiveTexture(opengl_rendering_context::TEXTURE0);
        shader.use_program(gl);
    }
    let screen_uniform_str = c_str!("screen");
    unsafe {
        let screen_uniform = gl.GetUniformLocation(shader.program, screen_uniform_str.as_ptr());
        gl.Uniform1i(screen_uniform, 0);
    }
}
