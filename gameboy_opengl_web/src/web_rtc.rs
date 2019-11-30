use gameboy_core::RTC;
use stdweb::web::Date;

pub struct WebRTC {}

impl RTC for WebRTC {
    fn get_current_time(&self) -> u64 {
        // get current time in ms and convert it to seconds
        (Date::now() / 1000f64) as u64
    }
}

impl WebRTC {
    pub fn new() -> WebRTC {
        WebRTC {}
    }
}
