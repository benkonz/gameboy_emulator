extern crate clap;
extern crate gameboy_opengl;

use clap::{App, Arg};
use std::fs::File;
use std::io;
use std::io::Read;

fn main() -> io::Result<()> {
    let matches = App::new("Gameboy Emulator")
        .version("0.1.9")
        .author("Benjamin K. <benkonz@protonmail.com>")
        .about("Gameboy Emulator written in Rust!")
        .arg(
            Arg::with_name("rom filename")
                .help("rom file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let rom_filename = matches.value_of("rom filename").unwrap();
    let mut file = File::open(rom_filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    gameboy_opengl::start(buffer);

    Ok(())
}
