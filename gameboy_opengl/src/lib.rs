mod native_rtc;
mod screen;
mod shader;

use crate::native_rtc::NativeRTC;
use crate::screen::Screen;
use crate::shader::Shader;
use directories::BaseDirs;
use gameboy_core::{Gameboy,Button, Cartridge, Controller, Rtc, StepResult};
use gl::types::*;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::cell::RefCell;
use std::ffi::CString;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;
use std::{mem, ptr};

const VERTEX_SOURCE: &str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SOURCE: &str = include_str!("shaders/fragment.glsl");
const VERTICIES: [f32; 20] = [
    1.0, 1.0, 0.0, 1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.0,
    0.0, 1.0,
];
const INDICIES: [u32; 6] = [0, 1, 3, 1, 2, 3];

pub fn start(rom: Vec<u8>) -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let mut timer_subsystem = sdl_context.timer().unwrap();

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2),
        samples: Some(4096),
    };
    let device = audio_subsystem.open_queue(None, &desired_spec).unwrap();
    device.resume();

    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Gameboy Emulator", 900, 700)
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

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



    let rtc = Box::new(NativeRTC::new());
    let mut emulator = Gameboy::from_rom(rom, rtc);

    load_ram_save_data(emulator.get_cartridge_mut());
    load_timestamp_data(emulator.get_cartridge_mut());

    let mut ram_save_file = get_ram_save_file(emulator.get_cartridge());
    let mut timestamp_save_file = get_timestamp_save_file(emulator.get_cartridge());

    let ram_changed = Rc::new(RefCell::new(true));
    {
        let ram_changed = ram_changed.clone();
        emulator.set_ram_change_callback(Box::new(move |_, _| {
            *ram_changed.borrow_mut() = true;
        }));
    }
    let mut controller = Controller::new();
    let mut screen = Screen::new();

    let mut event_pump = sdl_context.event_pump()?;
    'game_loop: loop {
        loop {
            let step_result = emulator.emulate(&mut screen);
            match step_result {
                StepResult::VBlank => {
                    let frame_buffer = screen.get_frame_buffer();
                    draw_texture(texture, &shader, frame_buffer);
                    unsafe {
                        gl::BindVertexArray(vao);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
                        gl::BindVertexArray(0);
                    }
                    window.gl_swap_window();
                    break;
                }
                StepResult::AudioBufferFull => {
                    let audio_buffer = emulator.get_audio_buffer();
                    while device.size() > (audio_buffer.len() * mem::size_of::<f32>()) as u32 {
                        timer_subsystem.delay(1);
                    }
                    device.queue(audio_buffer);
                    break;
                }
                StepResult::Nothing => (),
            }
        }

        if *ram_changed.borrow() && emulator.get_cartridge().has_battery() {
            if let Some(ref mut ram_save_file) = ram_save_file {
                save_ram_data(emulator.get_cartridge(), ram_save_file);
            }
            *ram_changed.borrow_mut() = false;
        }

        if emulator.get_cartridge().has_rtc() {
            if let Some(ref mut timestamp_save_file) = timestamp_save_file {
                save_timestamp_data(emulator.get_cartridge(), timestamp_save_file);
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'game_loop,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(button) = keycode_to_button(keycode) {
                        controller.press(button);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(button) = keycode_to_button(keycode) {
                        controller.release(button);
                    }
                }
                _ => (),
            };
        }
    }

    Ok(())
}

fn keycode_to_button(keycode: Keycode) -> Option<Button> {
    match keycode {
        Keycode::Z => Some(Button::A),
        Keycode::X => Some(Button::B),
        Keycode::Space => Some(Button::Start),
        Keycode::Return => Some(Button::Select),
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        _ => None,
    }
}

fn get_ram_saves_path() -> Option<PathBuf> {
    let base_dir = BaseDirs::new()?;
    let path_buf = base_dir
        .config_dir()
        .join("gameboy_emulator")
        .join("ram_saves");
    Some(path_buf)
}

fn draw_texture(texture: GLuint, shader: &Shader, frame_buffer: &[u8]) {
    let screen_uniform_str = CString::new("screen").unwrap();

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            160,
            144,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            frame_buffer.as_ptr() as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::ActiveTexture(gl::TEXTURE0);
        shader.use_program();

        let screen_uniform = gl::GetUniformLocation(shader.program, screen_uniform_str.as_ptr());
        gl::Uniform1i(screen_uniform, 0);
    }
}

fn load_ram_save_data(cartridge: &mut Cartridge) {
    if cartridge.has_battery() {
        if let Some(ram_saves_dir) = get_ram_saves_path() {
            let ram_save_file = ram_saves_dir.join(format!("{}.bin", cartridge.get_name()));
            if ram_save_file.exists() {
                let mut ram_save_file = OpenOptions::new().read(true).open(ram_save_file).unwrap();
                if let Ok(metadata) = ram_save_file.metadata() {
                    // sometimes two different roms have the same name,
                    // so we make sure that the ram length is the same before reading
                    if metadata.len() == cartridge.get_ram().len() as u64 {
                        ram_save_file.read_exact(cartridge.get_ram_mut()).unwrap();
                    }
                }
            }
        }
    }
}

fn load_timestamp_data(cartridge: &mut Cartridge) {
    if cartridge.has_rtc() {
        if let Some(ram_saves_dir) = get_ram_saves_path() {
            let timestamp_save_file =
                ram_saves_dir.join(format!("{}-timestamp.bin", cartridge.get_name()));
            if timestamp_save_file.exists() {
                let mut timestamp_save_file = OpenOptions::new()
                    .read(true)
                    .open(timestamp_save_file)
                    .unwrap();
                let mut rtc_data = [0; 5];
                timestamp_save_file.read_exact(&mut rtc_data).unwrap();
                let mut timestamp_data = [0; 8];
                timestamp_save_file.read_exact(&mut timestamp_data).unwrap();
                let last_rtc = Rtc::from_bytes(&rtc_data);
                let last_timestamp = u64::from_ne_bytes(timestamp_data);
                cartridge.set_last_timestamp(last_rtc, last_timestamp);
            }
        }
    }
}

fn get_ram_save_file(cartridge: &Cartridge) -> Option<impl Write + Seek> {
    if cartridge.has_battery() {
        let ram_saves_path = get_ram_saves_path()?;
        if !ram_saves_path.exists() {
            fs::create_dir_all(&ram_saves_path).unwrap();
        }
        let ram_save_file_path = ram_saves_path.join(format!("{}.bin", cartridge.get_name()));
        Some(
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(ram_save_file_path)
                .unwrap(),
        )
    } else {
        None
    }
}

fn get_timestamp_save_file(cartridge: &Cartridge) -> Option<impl Write + Seek> {
    if cartridge.has_rtc() {
        let ram_saves_path = get_ram_saves_path()?;
        if !ram_saves_path.exists() {
            fs::create_dir_all(&ram_saves_path).unwrap();
        }
        let timestamp_save_file_path =
            ram_saves_path.join(format!("{}-timestamp.bin", cartridge.get_name()));
        Some(
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(timestamp_save_file_path)
                .unwrap(),
        )
    } else {
        None
    }
}

fn save_ram_data<T: Write + Seek>(cartridge: &Cartridge, ram_save_file: &mut T) {
    if cartridge.has_battery() {
        let ram = cartridge.get_ram();
        ram_save_file.seek(SeekFrom::Start(0)).unwrap();
        ram_save_file.write_all(ram).unwrap();
    }
}

fn save_timestamp_data<T: Write + Seek>(cartridge: &Cartridge, timestamp_save_file: &mut T) {
    let (rtc_data, rtc_last_time) = cartridge.get_last_timestamp();
    let mut rtc_data = rtc_data.to_bytes().to_vec();
    let mut rtc_last_time_data = rtc_last_time.to_ne_bytes().to_vec();
    rtc_data.append(&mut rtc_last_time_data);
    timestamp_save_file.seek(SeekFrom::Start(0)).unwrap();
    timestamp_save_file.write_all(&rtc_data).unwrap();
}
