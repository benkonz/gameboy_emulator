use bit_utils;

const DIVISORS: [i32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

pub struct NoiseChannel {
    length_load: u8,
    volume: u8,
    volume_load: u8,
    envelope_add_mode: bool,
    envelope_period: i32,
    envelope_period_load: u8,
    length_counter: u8,
    divisor_code: u8,
    width_mode: bool,
    clock_shift: u8,
    length_enable: bool,
    trigger_bit: bool,
    dac_enabled: bool,
    enabled: bool,
    timer: i32,
    envelope_running: bool,
    lfsr: u16,
    output_vol: u8,
}

impl NoiseChannel {
    pub fn new() -> NoiseChannel {
        NoiseChannel {
            length_load: 0,
            volume: 0,
            volume_load: 0,
            envelope_add_mode: false,
            envelope_period: 0,
            envelope_period_load: 0,
            length_counter: 0,
            divisor_code: 0,
            width_mode: false,
            clock_shift: 0,
            length_enable: false,
            trigger_bit: false,
            dac_enabled: false,
            enabled: false,
            timer: 0,
            envelope_running: false,
            lfsr: 0,
            output_vol: 0,
        }
    }

    pub fn step(&mut self) {
        self.timer -= 1;
        if self.timer <= 0 {
            self.timer = DIVISORS[self.divisor_code as usize] << self.clock_shift;
            let result = (self.lfsr & 0x1) ^ ((self.lfsr >> 1) & 0x1);
            self.lfsr >>= 1;
            self.lfsr |= result << 14;
            if self.width_mode {
                self.lfsr &= !0x40;
                self.lfsr |= result << 6;
            }
            if self.enabled && self.dac_enabled && (self.lfsr & 0x1) == 0 {
                self.output_vol = self.volume;
            } else {
                self.output_vol = 0;
            }
        }
    }

    pub fn length_click(&mut self) {
        if self.length_counter > 0 && self.length_enable {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn env_click(&mut self) {
        self.envelope_period -= 1;
        if self.envelope_period <= 0 {
            self.envelope_period = self.envelope_period_load as i32;
            if self.envelope_period == 0 {
                self.envelope_period = 8;
            }
            if self.envelope_running && self.envelope_period_load > 0 {
                if self.envelope_add_mode && self.volume < 15 {
                    self.volume += 1;
                } else if !self.envelope_add_mode && self.volume > 0 {
                    self.volume -= 1;
                }
            }
            if self.volume == 0 || self.volume == 15 {
                self.envelope_running = false;
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF1F => 0,
            0xFF20 => self.length_load & 0x3F,
            0xFF21 => {
                let envelope_add_mode = if self.envelope_add_mode { 1 } else { 0 };
                (self.envelope_period_load & 0x07)
                    | (envelope_add_mode << 3)
                    | ((self.volume_load & 0x0F) << 4)
            }
            0xFF22 => {
                let width_mode = if self.width_mode { 1 } else { 0 };
                (self.divisor_code) | (width_mode << 3) | (self.clock_shift << 4)
            }
            0xFF23 => {
                let length_enable = if self.length_enable { 1 } else { 0 };
                let trigger_bit = if self.trigger_bit { 1 } else { 0 };
                (length_enable << 6) | (trigger_bit << 7)
            }
            _ => panic!("unkonwn address: {:04X}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF1F => (),
            0xFF20 => self.length_load = value & 0x3F,
            0xFF21 => {
                self.dac_enabled = (value & 0xF8) != 0;
                self.volume_load = (value >> 4) & 0x0F;
                self.envelope_add_mode = bit_utils::is_set(value, 3);
                self.envelope_period_load = value & 0x07;
            }
            0xFF22 => {
                self.divisor_code = value & 0x07;
                self.width_mode = bit_utils::is_set(value, 3);
                self.clock_shift = (value >> 4) & 0x0F;
            }
            0xFF23 => {
                self.length_enable = bit_utils::is_set(value, 6);
                self.trigger_bit = bit_utils::is_set(value, 7);
                if self.trigger_bit {
                    self.trigger();
                }
            }
            _ => panic!("unknown address: {:04X}", address),
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        self.timer = DIVISORS[self.divisor_code as usize] << self.clock_shift;
        self.envelope_period = self.envelope_period_load as i32;
        self.envelope_running = true;
        self.volume = self.volume_load;
        self.lfsr = 0x7FFF;
    }

    pub fn get_status(&self) -> bool {
        self.length_counter > 0
    }

    pub fn reset_length_counter(&mut self) {
        self.length_counter = 0;
    }

    pub fn get_output_vol(&self) -> u8 {
        self.output_vol
    }
}
