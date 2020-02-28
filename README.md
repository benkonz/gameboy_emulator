# GameBoy Emulator

![crates.io](https://img.shields.io/crates/v/gameboy_opengl)

This is a GameBoy emulator written in Rust. It can be compiled to native
and web assembly, see the build section for more details.

Emulator supports sound, several hardware types, RTC, gameboy color emulation,
sprites, and saving to browser local storage (web) and user config directories (native)

The web assembly port is currently hosted [here](https://benkonz.github.io/assets/emulator)

## Screenshots

<p align="center">
    <img src="screenshots/pokemon_crystal.png" height=240 />
    <img src="screenshots/super_mario.png" height=240 />
</p>
<p align="center">
    <img src="screenshots/tetris.png" height=240 />
    <img src="screenshots/mario.png" height=240 />
</p>
<p align="center">
    <img src="screenshots/pokemon_yellow.png" height=240 />
    <img src="screenshots/shantae.png" height=240 />
</p>
<p align="center">
    <img src="screenshots/zelda.png" height=240 />
    <img src="screenshots/metroid.png" height=240 />
</p>
<p align="center">
    <img src="screenshots/kirby2.png" height=240 />
    <img src="screenshots/zelda_ooa.png" height=240 />
</p>

## Blargg Tests

### CPU Instrs

<p>
    <img src="screenshots/cpu_instrs.png" height=240 />
</p>

### Instr Timings

<p>
    <img src="screenshots/instr_timings.png" height=240 />
</p>

### Mem Timings

<p>
    <img src="screenshots/mem_timings.png" height=240 />
</p>

### Mem Timings 2

<p>
    <img src="screenshots/mem_timings_2.png" height=240 />
</p>

### DMG Sound

<p>
    <img src="screenshots/dmg_sound.png" height=240 />
</p>

## Installing

The native version is published to [crates.io](https://crates.io/crates/gameboy_opengl) and can be 
installed by running:

```text
cargo install gameboy_opengl
```

Note: this library uses [Rust-SDL2](https://github.com/Rust-SDL2/rust-sdl2), so you will need
some sort of distribution of the SDL2 development libraries and CMake installed on your system before you can compile
this application. If you're having errros compiling, please look [here](https://github.com/Rust-SDL2/rust-sdl2#sdl20-development-libraries).

Then you can run it by running: `gameboy_emulator` from your terminal

## Building from source

The project uses Cargo as a build system, so building the project is relatively
simple.

### Native

```text
cargo build --package gameboy_opengl --bin gameboy_emulator --release
```

this produces the executable `target/release/gameboy_emulator.exe`

to run it, just supply the rom file as the first file argument

### Web Assembly

`cargo-web` is very useful for building the web
port of the emulator.

```text
cargo-web deploy --release
```

use your favorite static file server to serve the files generated in the
`target/deploy` directory. You can also run `cargo-web start --release`, to serve the files locally.
