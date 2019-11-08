#[cfg(target_arch = "wasm32")]
extern crate gameboy_opengl_web;
#[cfg(target_arch = "wasm32")]
extern crate stdweb;
#[cfg(target_arch = "wasm32")]
use stdweb::js_export;

#[cfg(not(target_arch = "wasm32"))]
extern crate gameboy_opengl;

/// # Safety
///
/// creates a vector from the pointer and length.
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub unsafe fn start(pointer: *mut u8, length: usize) {
    let rom = Vec::from_raw_parts(pointer, length, length);
    gameboy_opengl::start(rom);
}

#[cfg(target_arch = "wasm32")]
#[js_export]
fn start(rom: Vec<u8>) {
    gameboy_opengl_web::start(rom);
}
