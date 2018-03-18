use gameboy_core::emulator::traits::Io;
use gameboy_core::joypad::Joypad;
use gameboy_core::joypad::button::Button;
use glutin;
use glutin::*;
use gl;
use gl::types::*;
use std::os::raw::c_void;
use std::mem;
use std::ptr;
use shader::Shader;
use std;

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
    pub is_running: bool,
}

impl Screen {
    pub fn new() -> Screen {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new();
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

        Screen {
            gl_window,
            events_loop,
            shader,
            vao,
            vbo,
            ebo,
            texture,
            is_running: true,
        }
    }
}

impl Screen {
    pub fn draw(&self, pixels: &[u8; 144 * 160]) {
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
                pixels.as_ptr() as *const c_void,
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

impl Io for Screen {
    fn update_joypad(&mut self, joypad: &mut Joypad) {
        let mut running = self.is_running;
        self.events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == glutin::ElementState::Pressed {
                        if let Some(keycode) = input.virtual_keycode {
                            match keycode {
                                VirtualKeyCode::Up => {
                                    joypad.press(Button::Up);
                                }
                                VirtualKeyCode::Down => joypad.press(Button::Down),
                                VirtualKeyCode::Left => joypad.press(Button::Left),
                                VirtualKeyCode::Right => joypad.press(Button::Right),
                                VirtualKeyCode::Z => joypad.press(Button::A),
                                VirtualKeyCode::X => joypad.press(Button::B),
                                VirtualKeyCode::Space => joypad.press(Button::Start),
                                VirtualKeyCode::LShift => joypad.press(Button::Select),
                                _ => (),
                            }
                        }
                    } else if input.state == glutin::ElementState::Released {
                        if let Some(keycode) = input.virtual_keycode {
                            match keycode {
                                VirtualKeyCode::Up => {
                                    joypad.release(Button::Up);
                                }
                                VirtualKeyCode::Down => joypad.release(Button::Down),
                                VirtualKeyCode::Left => joypad.release(Button::Left),
                                VirtualKeyCode::Right => joypad.release(Button::Right),
                                VirtualKeyCode::Z => joypad.release(Button::A),
                                VirtualKeyCode::X => joypad.release(Button::B),
                                VirtualKeyCode::Space => joypad.release(Button::Start),
                                VirtualKeyCode::LShift => joypad.release(Button::Select),
                                _ => (),
                            }
                        }
                    }
                }
                WindowEvent::Closed => {
                    running = false;
                },
                _ => (),
            },
            _ => (),
        });
        self.is_running = running;
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
