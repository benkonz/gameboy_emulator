use gameboy_core::joypad::Joypad;
use gameboy_core::joypad::Button;
use gameboy_core::Color;
use gameboy_core::emulator::traits::PixelMapper;
use stdweb;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, IParentNode, TypedArray, window, IEventTarget};
use stdweb::web::event::{KeyUpEvent, KeyDownEvent};
use stdweb::traits::IKeyboardEvent;
use webgl_rendering_context::*;
use std::rc::Rc;
use std::cell::RefCell;

const VERTEX_SOURCE: &'static str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SOURCE: &'static str = include_str!("shaders/fragment.glsl");
const VERTICIES: [f32; 12] = [
    1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 1.0, 0.0,
];
const TEXTURE_COORDINATE: [f32; 8] = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
const INDICIES: [u8; 6] = [0, 1, 3, 1, 2, 3];

type Gl = WebGLRenderingContext;

pub struct Screen {
    context: Gl,
    joypad: Rc<RefCell<Joypad>>,
    texture: WebGLTexture,
    shader_program: WebGLProgram,
    pixels: Vec<u8>,
}

impl Screen {
    pub fn new() -> Screen {
        stdweb::initialize();

        let canvas: CanvasElement = document()
            .query_selector("#canvas")
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();

        let joypad = Rc::new(RefCell::new(Joypad::new()));

        {
            let joypad = joypad.clone();
            window().add_event_listener(move |event: KeyDownEvent| {
                match event.key().as_ref() {
                    "ArrowUp" => joypad.borrow_mut().press(Button::Up),
                    "ArrowDown" => joypad.borrow_mut().press(Button::Down),
                    "ArrowLeft" => joypad.borrow_mut().press(Button::Left),
                    "ArrowRight" => joypad.borrow_mut().press(Button::Right),
                    "z" => joypad.borrow_mut().press(Button::A),
                    "x" => joypad.borrow_mut().press(Button::B),
                    "Enter" => joypad.borrow_mut().press(Button::Select),
                    " " => joypad.borrow_mut().press(Button::Start),
                    _ => console!(log, "unknown key")
                }
            });
        }

        {
            let joypad = joypad.clone();

            window().add_event_listener(move |event: KeyUpEvent| {
                match event.key().as_ref() {
                    "ArrowUp" => joypad.borrow_mut().release(Button::Up),
                    "ArrowDown" => joypad.borrow_mut().release(Button::Down),
                    "ArrowLeft" => joypad.borrow_mut().release(Button::Left),
                    "ArrowRight" => joypad.borrow_mut().release(Button::Right),
                    "z" => joypad.borrow_mut().release(Button::A),
                    "x" => joypad.borrow_mut().release(Button::B),
                    "Enter" => joypad.borrow_mut().release(Button::Select),
                    " " => joypad.borrow_mut().release(Button::Start),
                    _ => console!(log, "unknown key")
                }
            });
        }


        let context: Gl = canvas.get_context().unwrap();

        context.clear_color(1.0, 0.0, 0.0, 1.0);
        context.clear(Gl::COLOR_BUFFER_BIT);

        let verticies = TypedArray::<f32>::from(&VERTICIES[..]).buffer();
        let vertex_buffer = context.create_buffer().unwrap();
        context.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));
        context.buffer_data_1(Gl::ARRAY_BUFFER, Some(&verticies), Gl::STATIC_DRAW);

        let textures = TypedArray::<f32>::from(&TEXTURE_COORDINATE[..]).buffer();
        let texture_buffer = context.create_buffer().unwrap();
        context.bind_buffer(Gl::ARRAY_BUFFER, Some(&texture_buffer));
        context.buffer_data_1(Gl::ARRAY_BUFFER, Some(&textures), Gl::STATIC_DRAW);

        let indicies = TypedArray::<u8>::from(&INDICIES[..]).buffer();
        let index_buffer = context.create_buffer().unwrap();
        context.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        context.buffer_data_1(Gl::ELEMENT_ARRAY_BUFFER, Some(&indicies), Gl::STATIC_DRAW);

        let vert_shader = context.create_shader(Gl::VERTEX_SHADER).unwrap();
        context.shader_source(&vert_shader, VERTEX_SOURCE);
        context.compile_shader(&vert_shader);

        let compiled = context.get_shader_parameter(&vert_shader, Gl::COMPILE_STATUS);

        if compiled == stdweb::Value::Bool(false) {
            let error = context.get_shader_info_log(&vert_shader);
            if let Some(e) = error {
                console!(log, e);
            }
        }

        let frag_shader = context.create_shader(Gl::FRAGMENT_SHADER).unwrap();
        context.shader_source(&frag_shader, FRAGMENT_SOURCE);
        context.compile_shader(&frag_shader);

        let compiled = context.get_shader_parameter(&frag_shader, Gl::COMPILE_STATUS);

        if compiled == stdweb::Value::Bool(false) {
            let error = context.get_shader_info_log(&frag_shader);
            if let Some(e) = error {
                console!(log, e);
            }
        }

        let shader_program = context.create_program().unwrap();
        context.attach_shader(&shader_program, &vert_shader);
        context.attach_shader(&shader_program, &frag_shader);
        context.link_program(&shader_program);

        context.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));
        let pos_attr = context.get_attrib_location(&shader_program, "aPos") as u32;
        context.vertex_attrib_pointer(pos_attr, 3, Gl::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(pos_attr);

        context.bind_buffer(Gl::ARRAY_BUFFER, Some(&texture_buffer));
        let tex_attr = context.get_attrib_location(&shader_program, "aTexCoord") as u32;
        context.vertex_attrib_pointer(tex_attr, 2, Gl::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(tex_attr);

        let texture = context.create_texture().unwrap();
        context.bind_texture(Gl::TEXTURE_2D, Some(&texture));

        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::NEAREST as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::NEAREST as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);

        let pixels = vec![0; 144 * 160 * 3];

        Screen {
            context,
            joypad,
            texture,
            shader_program,
            pixels,
        }
    }

    pub fn get_input(&mut self) -> Rc<RefCell<Joypad>> {
        self.joypad.clone()
    }

    pub fn render(&mut self) {
        self.context
            .bind_texture(Gl::TEXTURE_2D, Some(&self.texture));

        let pixels = &self.pixels[..];

        self.context.tex_image2_d(
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

        self.context.active_texture(Gl::TEXTURE0);

        self.context.use_program(Some(&self.shader_program));

        let screen_uniform = self.context
            .get_uniform_location(&self.shader_program, "screen")
            .unwrap();
        self.context.uniform1i(Some(&screen_uniform), 0);

        self.context
            .draw_elements(Gl::TRIANGLES, 6, Gl::UNSIGNED_BYTE, 0);
    }
}


impl PixelMapper for Screen {
    fn map_pixel(&mut self, pixel: usize, color: Color) {
        let color_bytes: [u8; 3] = match color {
            Color::White => [255, 255, 255],
            Color::LightGray => [178, 178, 178],
            Color::DarkGray => [102, 102, 102],
            Color::Black => [0, 0, 0],
        };

        for (i, byte) in color_bytes.iter().enumerate() {
            self.pixels[pixel * 3 + i] = *byte;
        }
    }
}
