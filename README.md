# GameBoy Emulator

![crates.io](https://img.shields.io/crates/v/gameboy_opengl)
[![Build Status](https://travis-ci.org/benkonz/gameboy_emulator.svg?branch=master)](https://travis-ci.org/benkonz/gameboy_emulator)

This is a GameBoy emulator written in Rust. It can be compiled to native
and WebAssembly; see the build section for more details.

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

The project uses Cargo as a build system.

### Native (desktop)

Prerequisites: a normal Rust toolchain with cargo; SDL2 is bundled via the `sdl2` crate.

```text
cargo build --package gameboy_opengl --bin gameboy_emulator --release
```

Run the binary with a ROM path:

```text
# Linux / macOS
./target/release/gameboy_emulator /path/to/game.gb

# Windows
.\target\release\gameboy_emulator.exe C:\path\to\game.gb
```

### WebAssembly

The browser build uses [**wasm-pack**](https://rustwasm.github.io/docs/wasm-pack/) (which wraps [**wasm-bindgen**](https://rustwasm.github.io/docs/wasm-bindgen/)) and loads the UI from [`static/index.html`](static/index.html).

Prerequisites:

```text
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

From the repository root (the `gameboy_emulator` crate that builds `gameboy_lib` as a `cdylib`):

```text
wasm-pack build . --release --target web --out-dir static/pkg
```

This writes JavaScript and Wasm bindings under `static/pkg/`. The HTML page imports `./pkg/gameboy_lib.js` and calls `init()` before `start`.

Serve the `static` directory over HTTP (browsers block Wasm on `file://`):

```text
cd static && python3 -m http.server 8080
```

Then open `http://localhost:8080` in a browser, choose a ROM, and play.

If audio does not start immediately, click or tap the page once; browsers often require a user gesture before `AudioContext` runs.

### Alternative: wasm-bindgen CLI

If you prefer not to install wasm-pack, build the crate and run `wasm-bindgen` yourself:

```text
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/gameboy_lib.wasm \
  --out-dir static/pkg --target web
```

You need `wasm-bindgen` on your PATH (for example `cargo install wasm-bindgen-cli`, matching the `wasm-bindgen` crate version used in this repo).
