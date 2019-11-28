use gameboy_core::RTC;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct NativeRTC {}

impl RTC for NativeRTC {
    fn get_current_time(&self) -> u64 {
        let now = SystemTime::now();
        now.duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

impl NativeRTC {
    pub fn new() -> NativeRTC {
        NativeRTC {}
    }
}
