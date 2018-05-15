use gameboy_core::joypad::button::Button;
use gameboy_core::joypad::Joypad;
use gameboy_core::traits::*;
use gameboy_core::Color;
use gl;
use gl::types::*;
use glutin;
use glutin::*;
use shader::Shader;
use std;
use std::os::raw::c_void;
use std::{mem, ptr};
use std::rc::Rc;
use std::cell::RefCell;

const VERTEX_SOURCE: &'static str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SOURCE: &'static str = include_str!("shaders/fragment.glsl");
const VERTICIES: [f32; 20] = [
    1.0, 1.0, 0.0, 1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.0,
    0.0, 1.0,
];
const INDICIES: [u32; 6] = [0, 1, 3, 1, 2, 3];

#[allow(dead_code)]
pub struct Screen {
    gl_window: glutin::GlWindow,
    events_loop: glutin::EventsLoop,
    shader: Shader,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    texture: GLuint,
    is_running: bool,
    joypad: Rc<RefCell<Joypad>>,
    pixels: [u8; 144 * 160],
}

impl Screen {
    pub fn new() -> Screen {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

        unsafe {
            gl_window.make_current().unwrap();
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.0, 1.0, 0.0, 1.0);
        }

        let shader = Shader::new(VERTEX_SOURCE, FRAGMENT_SOURCE);

        let (mut vao, mut vbo, mut ebo, mut texture) = (0, 0, 0, 0);

        unsafe {
            //setup vertex data
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (VERTICIES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                VERTICIES.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (INDICIES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                INDICIES.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;

            //position attribute
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);

            //texture attribute
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * mem::size_of::<GLfloat>()) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);

            //load and create texture
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            //set texture wrapping params
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            //set texture filtering params
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }

        let joypad = Rc::new(RefCell::new(Joypad::new()));

        Screen {
            gl_window,
            events_loop,
            shader,
            vao,
            vbo,
            ebo,
            texture,
            is_running: true,
            joypad,
            pixels: [0; 144 * 160],
        }
    }

    pub fn should_run(&self) -> bool {
        self.is_running
    }

    pub fn poll_input(&mut self) {
        let mut running = self.is_running;
        let joypad = self.joypad.clone();

        self.events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == glutin::ElementState::Pressed {
                        if let Some(keycode) = input.virtual_keycode {
                            match keycode {
                                VirtualKeyCode::Up => joypad.borrow_mut().press(Button::Up),
                                VirtualKeyCode::Down => joypad.borrow_mut().press(Button::Down),
                                VirtualKeyCode::Left => joypad.borrow_mut().press(Button::Left),
                                VirtualKeyCode::Right => joypad.borrow_mut().press(Button::Right),
                                VirtualKeyCode::Z => joypad.borrow_mut().press(Button::A),
                                VirtualKeyCode::X => joypad.borrow_mut().press(Button::B),
                                VirtualKeyCode::Space => joypad.borrow_mut().press(Button::Start),
                                VirtualKeyCode::LShift => joypad.borrow_mut().press(Button::Select),
                                _ => (),
                            }
                        }
                    } else if input.state == glutin::ElementState::Released {
                        if let Some(keycode) = input.virtual_keycode {
                            match keycode {
                                VirtualKeyCode::Up => joypad.borrow_mut().release(Button::Up),
                                VirtualKeyCode::Down => joypad.borrow_mut().release(Button::Down),
                                VirtualKeyCode::Left => joypad.borrow_mut().release(Button::Left),
                                VirtualKeyCode::Right => joypad.borrow_mut().release(Button::Right),
                                VirtualKeyCode::Z => joypad.borrow_mut().release(Button::A),
                                VirtualKeyCode::X => joypad.borrow_mut().release(Button::B),
                                VirtualKeyCode::Space => joypad.borrow_mut().release(Button::Start),
                                VirtualKeyCode::LShift => joypad.borrow_mut().release(Button::Select),
                                _ => (),
                            }
                        }
                    }
                }
                WindowEvent::Closed => running = false,
                _ => (),
            },
            _ => (),
        });
        self.is_running = running;
    }

    pub fn get_input(&mut self) -> Rc<RefCell<Joypad>> {
        let joypad = self.joypad.clone();
        joypad
    }

    pub fn render(&mut self) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R3_G3_B2 as i32,
                160,
                144,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE_3_3_2,
                self.pixels.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::ActiveTexture(gl::TEXTURE0);

            self.shader.use_program();
        }
        let screen_uniform_str = c_str!("screen");
        unsafe {
            let screen_uniform =
                gl::GetUniformLocation(self.shader.program, screen_uniform_str.as_ptr());
            gl::Uniform1i(screen_uniform, 0);

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }

        self.gl_window.swap_buffers().unwrap();
    }
}

impl PixelMapper for Screen {
    fn map_pixel(&mut self, x: u8, y: u8, color: Color) {
        let color_byte = match color {
            Color::White => 0b11111111,
            Color::LightGray => 0b01001010,
            Color::DarkGray => 0b00100101,
            Color::Black => 0b0000000
        };

        self.pixels[160 * (143 - y as usize) + x as usize] = color_byte;
    }

    fn get_pixel(&self, x: u8, y: u8) -> Color {
        match self.pixels[160 * (143 - y as usize) + x as usize] {
            0b11111111 => Color::White,
            0b01001010 => Color::LightGray,
            0b00100101 => Color::DarkGray,
            _ => Color::Black
        }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
