extern crate gameboy_opengl;

use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
//    let args: Vec<String> = env::args().collect();
    let args = vec!["", "C:/Users/konz/Documents/gameboy_emulator/lib/gameboy_opengl/tests/dr_mario.gb"];

    if args.len() == 2 {
        let filename = &args[1];
        match File::open(filename) {
            Ok(ref mut file) => {
                let mut buffer = Vec::new();
                match file.read_to_end(&mut buffer) {
                    Ok(_) => gameboy_opengl::start(buffer),
                    Err(_) => eprintln!("Error reading the file")
                }
            },
            Err(_) => eprintln!("Error opening the file")
        }
    } else {
        eprintln!("Incorrect usage. Program must be run with one rom file");
    }
}