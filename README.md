# GameBoy Emulator

This is a GameBoy emulator written in Rust. It can be compiled to native
and web assembly, see the build section for more details. There are a few
missing features, such as sound and supporting ROM's larger than 32KB.

## Screenshots

![Dr. Mario](https://raw.githubusercontent.com/benkonz/gameboy_emulator/master/screenshots/dr_mario.PNG)

![Tetris](https://raw.githubusercontent.com/benkonz/gameboy_emulator/master/screenshots/tetris.PNG)

![Oranges](https://raw.githubusercontent.com/benkonz/gameboy_emulator/master/screenshots/oranges.PNG)

## Building from source

The project uses Cargo as a build system, so building the project is relatively
simple.

### Native

```text
cd lib 
cargo build --package gameboy_opengl --bin gameboy_emulator --release
```

this produces the executable `target/release/gameboy_emulator.exe`

to run it, just supply the rom file as the first file argument

### Web Assembly

`cargo-web` and `npm` are very useful for building the web
port of the emulator.

```text
cd site
npm install
npm run compile
```

use your faviorate static file server to serve the files generated in the
`dist` folder. You can also run `npm run serve`, to serve the files locally.