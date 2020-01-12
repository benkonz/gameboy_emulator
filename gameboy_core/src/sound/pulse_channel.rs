use bit_utils;

pub struct PulseChannel {
    sweep_shift: u8,
    sweep_negate: bool,
    sweep_period_load: u8,
    length_load: u8,
    duty: u8,
    envelope_period: u8,
    envelope_period_load: u8,
    envelope_add_mode: bool,
    volume_load: u8,
    volume: u8,
    timer_load: u16,
    length_enable: bool,
    trigger_bit: bool,
    length_counter: u8,
    dac_enabled: bool,
    enabled: bool,
    timer: i32,
    envelope_running: bool,
    sweep_shadow: u16,
    sweep_enable: bool,
    sweep_period: i32,
    output_vol: u8,
}

impl PulseChannel {
    pub fn new() -> PulseChannel {
        PulseChannel {
            sweep_shift: 0,
            sweep_negate: false,
            sweep_period_load: 0,
            length_load: 0,
            duty: 0,
            envelope_period: 0,
            envelope_period_load: 0,
            envelope_add_mode: false,
            volume_load: 0,
            volume: 0,
            timer_load: 0,
            length_enable: false,
            trigger_bit: false,
            length_counter: 0,
            dac_enabled: true,
            timer: 0,
            sweep_enable: false,
            envelope_running: true,
            sweep_shadow: 0,
            enabled: false,
            sweep_period: 0,
            output_vol: 0
        }
    }

    pub fn step(&mut self) {}

    pub fn length_click(&mut self) {}

    pub fn sweep_click(&mut self) {}

    pub fn env_click(&mut self) {}

    pub fn read_byte(&self, address: u16) -> u8 {
        match (address % 0xF) % 0x5 {
            0x0 => {
                let sweep_negate = if self.sweep_negate { 1 } else { 0 };
                self.sweep_shift | (sweep_negate << 3) | (self.sweep_period_load << 4)
            }
            0x1 => (self.length_load & 0x3F) | ((self.duty & 0x03) << 6),
            0x2 => {
                let envelope_add_mode = if self.envelope_add_mode { 1 } else { 0 };
                (self.envelope_period_load & 0x07)
                    | (envelope_add_mode << 3)
                    | ((self.volume_load & 0x0F) << 4)
            }
            0x3 => (self.timer_load & 0xFF) as u8,
            0x4 => {
                let trigger_bit = if self.trigger_bit { 1 } else { 0 };
                let length_enable = if self.length_enable { 1 } else { 0 };
                ((self.timer_load >> 8) & 0x07) as u8 | (length_enable << 6) | (trigger_bit << 7)
            }
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match (address % 0xF) % 0x5 {
            0x0 => {
                self.sweep_shift = value & 0x07;
                self.sweep_negate = bit_utils::is_set(value, 3);
                self.sweep_period_load = (value >> 4) & 0x07;
            }
            0x1 => {
                self.length_load = value & 0x3F;
                self.duty = (value >> 6) & 0x3;
            }
            0x2 => {
                self.dac_enabled = (value & 0xF8) != 0;
                self.volume_load = (value >> 4) & 0x0F;
                self.envelope_add_mode = bit_utils::is_set(value, 3);
                self.envelope_period_load = value & 0x7;
                self.envelope_period = self.envelope_period_load;
                self.volume = self.volume_load;
            }
            0x3 => self.timer_load = (self.timer_load & 0x700) | value as u16,
            0x4 => {
                self.timer_load = (self.timer_load & 0xFF) | ((value as u16 & 0x7) << 8);
                self.length_enable = bit_utils::is_set(value, 6);
                self.trigger_bit = bit_utils::is_set(value, 7);
                if bit_utils::is_set(value, 7) {
                    self.trigger();
                }
            }
            _ => unreachable!(),
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        self.timer = (2048 - self.timer_load as i32) * 4;
        self.envelope_running = true;
        self.envelope_period = self.envelope_period_load;
        self.volume = self.volume_load;
        self.sweep_shadow = self.timer_load;
        self.sweep_period = self.sweep_period_load as i32;
        if self.sweep_period == 0 {
            self.sweep_period = 8;
        }
        self.sweep_enable = self.sweep_period > 0 || self.sweep_shift > 0;
        if self.sweep_shift > 0 {
            self.sweep_calculation();
        }
    }

    fn sweep_calculation(&mut self) -> u16 {
        let mut new_freq = self.sweep_shadow >> self.sweep_shift;
        if self.sweep_negate {
            new_freq = self.sweep_shadow - new_freq;
        } else {
            new_freq += self.sweep_shadow;
        }
        if new_freq > 2047 {
            self.enabled = false;
        }
        new_freq
    }

    pub fn get_status(&self) -> bool {
        self.length_counter > 0
    }

    pub fn get_output_vol(&self) -> u8 {
        self.output_vol
    }
}
