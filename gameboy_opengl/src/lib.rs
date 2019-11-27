extern crate directories;
extern crate gameboy_core;
extern crate glutin;

mod native_rtc;
mod opengl_rendering_context;
mod screen;
mod shader;

use directories::BaseDirs;
use gameboy_core::{Button, Cartridge, Controller, ControllerEvent, Emulator};
use glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use native_rtc::NativeRTC;
use opengl_rendering_context::types::*;
use opengl_rendering_context::Gl;
use screen::Screen;
use shader::Shader;
use std::cell::RefCell;
use std::ffi::CString;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{SendError, TryRecvError};
use std::thread;
use std::time::{Duration, SystemTime};
use std::{mem, ptr};

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

    let shader = Shader::new(&gl, VERTEX_SOURCE, FRAGMENT_SOURCE);
    let (mut vao, mut vbo, mut ebo, mut texture) = (0, 0, 0, 0);

    unsafe {
        gl.ClearColor(0.0, 1.0, 0.0, 1.0);
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
    let (end_sender, end_receiver) = mpsc::channel();
    {
        thread::spawn(move || {
            let mut cartridge = Cartridge::from_rom(rom);

            if let Some(ram_saves_dir) = get_ram_saves_path() {
                let ram_save_file = ram_saves_dir.join(format!("{}.bin", cartridge.get_name()));
                if ram_save_file.exists() {
                    let mut ram_save_file =
                        OpenOptions::new().read(true).open(ram_save_file).unwrap();
                    if let Ok(metadata) = ram_save_file.metadata() {
                        // sometimes two different roms have the same name,
                        // so we make sure that the ram length is the same before reading
                        if metadata.len() == cartridge.get_ram().len() as u64 {
                            ram_save_file.read_exact(cartridge.get_ram_mut()).unwrap();
                        }
                    }
                }
            }
            let rtc = Box::new(NativeRTC::new());
            let mut emulator = Emulator::from_cartridge(cartridge, rtc);

            let mut ram_save_file = None;
            if let Some(ram_saves_path) = get_ram_saves_path() {
                if !ram_saves_path.exists() {
                    fs::create_dir_all(&ram_saves_path).unwrap();
                }
                let ram_save_file_path =
                    ram_saves_path.join(format!("{}.bin", emulator.get_cartridge().get_name()));

                ram_save_file = Some(
                    OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(ram_save_file_path)
                        .unwrap(),
                )
            }

            let ram_changed = Rc::new(RefCell::new(true));
            {
                let ram_changed = ram_changed.clone();
                emulator.set_ram_change_callback(Box::new(move |_, _| {
                    *ram_changed.borrow_mut() = true;
                }));
            }
            let mut controller = Controller::new();
            let mut screen = Screen::new();
            let frame_rate = 60f64;
            let frame_duration = Duration::from_secs_f64(1f64 / frame_rate);
            'game_loop: loop {
                let start_time = SystemTime::now();
                loop {
                    let vblank = emulator.emulate(&mut screen, &mut controller);
                    if vblank {
                        break;
                    }
                }

                if *ram_changed.borrow() && emulator.get_cartridge().has_battery() {
                    if let Some(ref mut ram_save_file) = ram_save_file {
                        let ram = emulator.get_cartridge().get_ram();
                        ram_save_file.seek(SeekFrom::Start(0)).unwrap();
                        ram_save_file.write_all(ram).unwrap();
                    }
                    *ram_changed.borrow_mut() = false;
                }

                match frame_sender.send(screen.get_frame_buffer().clone()) {
                    Ok(()) => (),
                    Err(SendError(_)) => break,
                };
                loop {
                    match controller_receiver.try_recv() {
                        Ok(input) => match input {
                            ControllerEvent::Pressed(button) => controller.press(button),
                            ControllerEvent::Released(button) => controller.release(button),
                        },
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => break 'game_loop,
                    }
                }
                match end_receiver.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => break,
                    _ => (),
                }

                let end_time = SystemTime::now();
                let last_frame_duration = end_time
                    .duration_since(start_time)
                    .unwrap_or_else(|_| Duration::new(0, 0));
                if frame_duration >= last_frame_duration {
                    let sleep_duration = frame_duration - last_frame_duration;
                    thread::sleep(sleep_duration);
                }
            }
        });
    }

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => {
                end_sender.send(true).unwrap();
                shader.delete_program(&gl);
                return;
            }
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
            Err(TryRecvError::Disconnected) => {
                *control_flow = ControlFlow::Exit;
            }
        };
    });
}

fn get_ram_saves_path() -> Option<PathBuf> {
    let base_dir = BaseDirs::new()?;
    let path_buf = base_dir
        .config_dir()
        .join("gameboy_emulator")
        .join("ram_saves");
    Some(path_buf)
}

fn draw_texture(gl: &Gl, texture: GLuint, shader: &Shader, frame_buffer: &[u8]) {
    let screen_uniform_str = CString::new("string").unwrap();

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

        let screen_uniform = gl.GetUniformLocation(shader.program, screen_uniform_str.as_ptr());
        gl.Uniform1i(screen_uniform, 0);
    }
}
