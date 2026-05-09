use gameboy_core::RTC;

pub struct WebRTC {}

impl RTC for WebRTC {
    fn get_current_time(&self) -> u64 {
        (js_sys::Date::now() / 1000.0) as u64
    }
}

impl WebRTC {
    pub fn new() -> WebRTC {
        WebRTC {}
    }
}
