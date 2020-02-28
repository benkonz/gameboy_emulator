#[cfg(target_arch = "wasm32")]
extern crate gameboy_opengl_web;
#[cfg(target_arch = "wasm32")]
pub use gameboy_opengl_web::DOMInfo;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;
#[cfg(target_arch = "wasm32")]
use stdweb::js_export;
#[cfg(target_arch = "wasm32")]
use stdweb::JsSerialize;

#[cfg(not(target_arch = "wasm32"))]
extern crate gameboy_opengl;

/// # Safety
///
/// creates a vector from the pointer and length.
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub unsafe fn start(pointer: *mut u8, length: usize) {
    let rom = Vec::from_raw_parts(pointer, length, length);
    gameboy_opengl::start(rom).unwrap();
}

#[cfg(target_arch = "wasm32")]
#[js_export]
pub fn start(rom: Vec<u8>, dom_ids: String) {
    let res = serde_json::from_str(dom_ids.as_str());
    if res.is_ok() {
        gameboy_opengl_web::start(rom, res.ok().unwrap());
    } else {
        js! {
            console.log("bad parse");
        }
    }
}
