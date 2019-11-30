#[derive(Default, Debug, Clone, Copy)]
pub struct Rtc {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub days_low: u8,
    pub days_high: u8,
}

impl Rtc {
    pub fn new() -> Rtc {
        Default::default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Rtc {
        Rtc {
            seconds: bytes[0],
            minutes: bytes[1],
            hours: bytes[2],
            days_low: bytes[3],
            days_high: bytes[4],
        }
    }

    pub fn to_bytes(self) -> [u8; 5] {
        [
            self.seconds,
            self.minutes,
            self.hours,
            self.days_low,
            self.days_high,
        ]
    }
}
