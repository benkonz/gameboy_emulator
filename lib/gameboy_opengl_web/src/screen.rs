use gameboy_core::joypad::Joypad;
use gameboy_core::joypad::Button;
use gameboy_core::traits::*;
use gameboy_core::Color;
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

type gl = WebGLRenderingContext;

pub struct Screen {
    context: gl,
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


        let context: gl = canvas.get_context().unwrap();

//        canvas.set_width(canvas.offset_width() as u32);
//        canvas.set_height(canvas.offset_height() as u32);

        context.clear_color(1.0, 0.0, 0.0, 1.0);
        context.clear(gl::COLOR_BUFFER_BIT);

        let verticies = TypedArray::<f32>::from(&VERTICIES[..]).buffer();
        let vertex_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&vertex_buffer));
        context.buffer_data_1(gl::ARRAY_BUFFER, Some(&verticies), gl::STATIC_DRAW);

        let textures = TypedArray::<f32>::from(&TEXTURE_COORDINATE[..]).buffer();
        let texture_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&texture_buffer));
        context.buffer_data_1(gl::ARRAY_BUFFER, Some(&textures), gl::STATIC_DRAW);

        let indicies = TypedArray::<u8>::from(&INDICIES[..]).buffer();
        let index_buffer = context.create_buffer().unwrap();
        context.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        context.buffer_data_1(gl::ELEMENT_ARRAY_BUFFER, Some(&indicies), gl::STATIC_DRAW);

        let vert_shader = context.create_shader(gl::VERTEX_SHADER).unwrap();
        context.shader_source(&vert_shader, VERTEX_SOURCE);
        context.compile_shader(&vert_shader);

        let compiled = context.get_shader_parameter(&vert_shader, gl::COMPILE_STATUS);

        if compiled == stdweb::Value::Bool(false) {
            let error = context.get_shader_info_log(&vert_shader);
            if let Some(e) = error {
                console!(log, e);
            }
        }

        let frag_shader = context.create_shader(gl::FRAGMENT_SHADER).unwrap();
        context.shader_source(&frag_shader, FRAGMENT_SOURCE);
        context.compile_shader(&frag_shader);

        let compiled = context.get_shader_parameter(&frag_shader, gl::COMPILE_STATUS);

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

        context.bind_buffer(gl::ARRAY_BUFFER, Some(&vertex_buffer));
        let pos_attr = context.get_attrib_location(&shader_program, "aPos") as u32;
        context.vertex_attrib_pointer(pos_attr, 3, gl::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(pos_attr);

        context.bind_buffer(gl::ARRAY_BUFFER, Some(&texture_buffer));
        let tex_attr = context.get_attrib_location(&shader_program, "aTexCoord") as u32;
        context.vertex_attrib_pointer(tex_attr, 2, gl::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(tex_attr);

        let texture = context.create_texture().unwrap();
        context.bind_texture(gl::TEXTURE_2D, Some(&texture));

        context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

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
}

impl Render for Screen {
    fn render(&mut self) {
        self.context
            .bind_texture(gl::TEXTURE_2D, Some(&self.texture));

        let pixels = &self.pixels[..];

        self.context.tex_image2_d(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            160,
            144,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            Some(pixels.as_ref()),
        );

//        self.context.generate_mipmap(gl::TEXTURE_2D);
        self.context.active_texture(gl::TEXTURE0);

        self.context.use_program(Some(&self.shader_program));

        let screen_uniform = self.context
            .get_uniform_location(&self.shader_program, "screen")
            .unwrap();
        self.context.uniform1i(Some(&screen_uniform), 0);

        self.context
            .draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_BYTE, 0);
    }
}

impl PixelMapper for Screen {
    fn map_pixel(&mut self, x: u8, y: u8, color: Color) {
        let color_bytes: [u8; 3] = match color {
            Color::White => [255, 255, 255],
            Color::LightGray => [178, 178, 178],
            Color::DarkGray => [102, 102, 102],
            Color::Black => [0, 0, 0],
        };

        let offset = (160 * (143 - y as usize) + x as usize) * 3;
        for (i, byte) in color_bytes.iter().enumerate() {
            self.pixels[offset + i] = *byte;
        }
    }

    fn get_pixel(&self, x: u8, y: u8) -> Color {
        let offset = (160 * (143 - y as usize) + x as usize) * 3;
        match self.pixels[offset..offset + 3] {
            [255, 255, 255] => Color::White,
            [178, 178, 178] => Color::LightGray,
            [102, 102, 102] => Color::DarkGray,
            [0, 0, 0] => Color::Black,
            _ => Color::Black
        }
    }
}
