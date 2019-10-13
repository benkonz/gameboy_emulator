use gameboy_core::Button;
use gameboy_core::Color;
use gameboy_core::Controller;
use gameboy_core::Emulator;
use gameboy_core::PixelMapper;
use glutin::event::ElementState;
use glutin::event::Event;
use glutin::event::KeyboardInput;
use glutin::event::VirtualKeyCode;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use opengl_rendering_context;
use opengl_rendering_context::types::*;
use opengl_rendering_context::Gl;
use shader::Shader;
use std;
use std::os::raw::c_void;
use std::time::{Duration, SystemTime};
use std::{mem, ptr};

const VERTEX_SOURCE: &'static str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SOURCE: &'static str = include_str!("shaders/fragment.glsl");
const VERTICIES: [f32; 20] = [
    1.0, 1.0, 0.0, 1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.0,
    0.0, 1.0,
];
const INDICIES: [u32; 6] = [0, 1, 3, 1, 2, 3];

struct Mapper {
    frame_buffer: [u8; 144 * 160 * 3],
}

impl Mapper {
    pub fn new() -> Mapper {
        Mapper {
            frame_buffer: [0; 144 * 160 * 3],
        }
    }

    pub fn get_frame_buffer(&self) -> &[u8; 144 * 160 * 3] {
        &self.frame_buffer
    }
}

impl PixelMapper for Mapper {
    fn map_pixel(&mut self, pixel: usize, color: Color) {
        let color_bytes: [u8; 3] = match color {
            Color::White => [255, 255, 255],
            Color::LightGray => [178, 178, 178],
            Color::DarkGray => [102, 102, 102],
            Color::Black => [0, 0, 0],
        };

        for (i, byte) in color_bytes.iter().enumerate() {
            self.frame_buffer[pixel * 3 + i] = *byte;
        }
    }
}

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

    let mut mapper = Mapper::new();
    let mut controller = Controller::new();
    let mut emulator = Emulator::from_rom(rom);

    let frame_rate = 60f64;
    let frame_duration = Duration::from_millis((1000f64 * (1f64 / frame_rate)) as u64);

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => match input {
                    // key pressed
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Up),
                        ..
                    } => controller.press(Button::Up),
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Down),
                        ..
                    } => controller.press(Button::Down),
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Left),
                        ..
                    } => controller.press(Button::Left),
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Right),
                        ..
                    } => controller.press(Button::Right),
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Z),
                        ..
                    } => controller.press(Button::A),
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::X),
                        ..
                    } => controller.press(Button::B),
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Return),
                        ..
                    } => controller.press(Button::Select),
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    } => controller.press(Button::Start),
                    // key released
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Up),
                        ..
                    } => controller.release(Button::Up),
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Down),
                        ..
                    } => controller.release(Button::Down),
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Left),
                        ..
                    } => controller.release(Button::Left),
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Right),
                        ..
                    } => controller.release(Button::Right),
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Z),
                        ..
                    } => controller.release(Button::A),
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::X),
                        ..
                    } => controller.release(Button::B),
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Return),
                        ..
                    } => controller.release(Button::Select),
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    } => controller.release(Button::Start),
                    _ => (),
                },
                WindowEvent::CloseRequested => {
                    shader.delete_program(&gl);
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            _ => (),
        }

        // render the next frame of emulation
        let vblank = emulator.emulate(&mut mapper, &mut controller);

        if vblank {
            draw_texture(&gl, texture, &shader, &mapper);
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
            windowed_context.swap_buffers().unwrap();
        }
    });
}

fn draw_texture(gl: &Gl, texture: GLuint, shader: &Shader, mapper: &Mapper) {
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
            mapper.get_frame_buffer().as_ptr() as *const c_void,
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
