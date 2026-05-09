#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub unsafe fn start(pointer: *mut u8, length: usize) {
    let rom = Vec::from_raw_parts(pointer, length, length);
    gameboy_opengl::start(rom).unwrap();
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(rom: Vec<u8>, dom_ids: String) {
    if let Err(msg) = start_with_result(rom, dom_ids) {
        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&msg));
    }
}

#[cfg(target_arch = "wasm32")]
fn start_with_result(rom: Vec<u8>, dom_ids: String) -> Result<(), String> {
    let dom_info = serde_json::from_str(dom_ids.as_str()).map_err(|e| format!("{:?}", e))?;
    gameboy_opengl_web::start(rom, dom_info)?;
    Ok(())
}
