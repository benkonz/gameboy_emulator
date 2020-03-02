mod native_rtc;
mod screen;

use crate::native_rtc::NativeRTC;
use crate::screen::Screen;
use directories::BaseDirs;
use gameboy_core::{Button, Cartridge, Controller, Emulator, Rtc, StepResult};
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use std::cell::RefCell;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::rc::Rc;

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
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
        .unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let mut cartridge = Cartridge::from_rom(rom);
    load_ram_save_data(&mut cartridge);
    load_timestamp_data(&mut cartridge);

    let rtc = Box::new(NativeRTC::new());
    let mut emulator = Emulator::from_cartridge(cartridge, rtc);

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
            let step_result = emulator.emulate(&mut screen, &mut controller);
            match step_result {
                StepResult::VBlank => {
                    let frame_buffer = screen.get_frame_buffer();
                    texture
                        .with_lock(None, |buffer, _| buffer.clone_from_slice(frame_buffer))
                        .unwrap();
                    canvas.clear();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();
                    break;
                }
                StepResult::AudioBufferFull => {
                    let audio_buffer = emulator.get_audio_buffer();
                    while device.size() > (audio_buffer.len() * std::mem::size_of::<f32>()) as u32 {
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
