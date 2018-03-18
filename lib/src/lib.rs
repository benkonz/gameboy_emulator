extern crate gameboy_opengl;

#[no_mangle]
pub extern fn start(rom: *const u8, size: usize) {
    let rom = unsafe { Vec::from_raw_parts(rom as *mut u8, size, size) };
    gameboy_opengl::start(rom);
}