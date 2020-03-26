#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

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
    if let Err(msg) = start_with_result(rom, dom_ids) {
        console!(error, msg);
    }
}

#[cfg(target_arch = "wasm32")]
fn start_with_result(rom: Vec<u8>, dom_ids: String) -> Result<(), String> {
    let dom_info = serde_json::from_str(dom_ids.as_str()).map_err(|e| format!("{:?}", e))?;
    gameboy_opengl_web::start(rom, dom_info)?;
    Ok(())
}
