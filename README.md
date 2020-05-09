# GameBoy Emulator

![crates.io](https://img.shields.io/crates/v/gameboy_opengl)
[![Build Status](https://travis-ci.org/benkonz/gameboy_emulator.svg?branch=master)](https://travis-ci.org/benkonz/gameboy_emulator)
[![Build status](https://ci.appveyor.com/api/projects/status/mllgt64mwj4966lu/branch/master?svg=true)](https://ci.appveyor.com/project/benkonz/gameboy-emulator/branch/master)

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
    <img src="screenshots/blaarg_tests.png" height=240 />
</p>

## Installing

The native version is published to [crates.io](https://crates.io/crates/gameboy_opengl) and can be 
installed by running:

```text
cargo install gameboy_opengl
```

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
