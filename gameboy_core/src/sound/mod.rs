use bit_utils;

mod noise_channel;
mod pulse_channel;
mod wave_channel;

use self::noise_channel::NoiseChannel;
use self::pulse_channel::PulseChannel;
use self::wave_channel::WaveChannel;

const READ_BYTE_OR_MASKS: [u8; 23] = [
    0x80, 0x3f, 0x00, 0xff, 0xbf, 0xff, 0x3f, 0x00, 0xff, 0xbf, 0x7f, 0xff, 0x9f, 0xff, 0xbf, 0xff,
    0xff, 0x00, 0x00, 0xbf, 0x00, 0x00, 0x70,
];
const SAMPLE_SIZE: usize = 4096;

pub struct Sound {
    wave_channel: WaveChannel,
    pulse_channel_1: PulseChannel,
    pulse_channel_2: PulseChannel,
    noise_channel: NoiseChannel,
    audio_buffer: [f32; SAMPLE_SIZE],
    vin_l_enable: bool,
    vin_l_volume: u8,
    vin_r_enable: bool,
    vin_r_volume: u8,
    left_enables: [bool; 4],
    right_enables: [bool; 4],
    power_control: bool,
    frame_sequence_count_down: i32,
    frame_sequencer: u8,
    down_sample_count: i32,
    buffer_fill_amount: usize,
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            wave_channel: WaveChannel::new(),
            pulse_channel_1: PulseChannel::new(),
            pulse_channel_2: PulseChannel::new(),
            noise_channel: NoiseChannel::new(),
            audio_buffer:  [0.5; SAMPLE_SIZE],
            vin_l_enable: false,
            vin_l_volume: 0,
            vin_r_enable: false,
            vin_r_volume: 0,
            left_enables: [false; 4],
            right_enables: [false; 4],
            power_control: false,
            frame_sequence_count_down: 8192,
            frame_sequencer: 0,
            down_sample_count: 95,
            buffer_fill_amount: 0,
        }
    }

    pub fn step(&mut self, cycles: i32) -> bool {
        let mut audio_buffer_full = false;
        let mut cycles = cycles;
        while cycles != 0 {
            cycles -= 1;
            self.frame_sequence_count_down -= 1;
            if self.frame_sequence_count_down == 0 {
                self.frame_sequence_count_down = 8192;
                match self.frame_sequencer {
                    0 | 4 => {
                        self.pulse_channel_1.length_click();
                        self.pulse_channel_2.length_click();
                        self.wave_channel.length_click();
                        self.noise_channel.length_click();
                    }
                    2 | 6 => {
                        self.pulse_channel_1.sweep_click();
                        self.pulse_channel_1.length_click();
                        self.pulse_channel_2.length_click();
                        self.wave_channel.length_click();
                        self.noise_channel.length_click();
                    }
                    7 => {
                        self.pulse_channel_1.env_click();
                        self.pulse_channel_2.env_click();
                        self.noise_channel.env_click();
                    }
                    _ => (),
                };
                self.frame_sequencer += 1;
                if self.frame_sequencer >= 8 {
                    self.frame_sequencer = 0;
                }
            }

            self.pulse_channel_1.step();
            self.pulse_channel_2.step();
            self.wave_channel.step();
            self.noise_channel.step();

            self.down_sample_count -= 1;
            if self.down_sample_count <= 0 {
                self.down_sample_count = 95;

                // left
                let mut bufferin_0 = 0.0;
                let mut bufferin_1;
                let volume = (128 * self.vin_l_volume as i32) / 7;
                if self.left_enables[0] {
                    bufferin_1 = (self.pulse_channel_1.get_output_vol() as f32) / 100.0;
                    Sound::mix_audio(&mut bufferin_0, bufferin_1, volume);
                }
                if self.left_enables[1] {
                    bufferin_1 = (self.pulse_channel_2.get_output_vol() as f32) / 100.0;
                    Sound::mix_audio(&mut bufferin_0, bufferin_1, volume);
                }
                if self.left_enables[2] {
                    bufferin_1 = (self.wave_channel.get_output_vol() as f32) / 100.0;
                    Sound::mix_audio(&mut bufferin_0, bufferin_1, volume);
                }
                if self.left_enables[3] {
                    bufferin_1 = (self.noise_channel.get_output_vol() as f32) / 100.0;
                    Sound::mix_audio(&mut bufferin_0, bufferin_1, volume);
                }
                // self.audio_buffer[self.buffer_fill_amount] = bufferin_0;

                // right
                bufferin_0 = 0.0;
                if self.right_enables[0] {
                    bufferin_1 = (self.pulse_channel_1.get_output_vol() as f32) / 100.0;
                    Sound::mix_audio(&mut bufferin_0, bufferin_1, volume);
                }
                if self.right_enables[1] {
                    bufferin_1 = (self.pulse_channel_2.get_output_vol() as f32) / 100.0;
                    Sound::mix_audio(&mut bufferin_0, bufferin_1, volume);
                }
                if self.right_enables[2] {
                    bufferin_1 = (self.wave_channel.get_output_vol() as f32) / 100.0;
                    Sound::mix_audio(&mut bufferin_0, bufferin_1, volume);
                }
                if self.right_enables[3] {
                    bufferin_1 = (self.noise_channel.get_output_vol() as f32) / 100.0;
                    Sound::mix_audio(&mut bufferin_0, bufferin_1, volume);
                }
                // self.audio_buffer[self.buffer_fill_amount + 1] = bufferin_0;
                self.buffer_fill_amount += 2;
            }

            if self.buffer_fill_amount >= SAMPLE_SIZE {
                self.buffer_fill_amount = 0;
                audio_buffer_full = true;
                break;
            }
        }
        audio_buffer_full
    }

    // copied from the SDL2 SDL_MixAudioFormat function
    // adapted for mixing two individual floats
    fn mix_audio(dst: &mut f32, src: f32, volume: i32) {
        let fmax_volume = 1.0f32 / 128f32;
        let fvolume = volume as f32;
        let max_audioval = 3.402_823_466e+38f64;
        let min_audioval = -3.402_823_466e+38f64;

        let src1 = src * fvolume * fmax_volume;
        let src2 = *dst;

        let mut dst_sample = src1 as f64 + src2 as f64;
        if dst_sample > max_audioval {
            dst_sample = max_audioval;
        } else if dst_sample < min_audioval {
            dst_sample = min_audioval;
        }
        *dst = dst_sample as f32;
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let value = match address {
            0xFF10..=0xFF14 => self.pulse_channel_1.read_byte(address),
            0xFF15..=0xFF19 => self.pulse_channel_2.read_byte(address),
            0xFF1A..=0xFF1E => self.wave_channel.read_byte(address),
            0xFF1F..=0xFF23 => self.noise_channel.read_byte(address),
            0xFF24 => {
                let vin_l_enable = if self.vin_l_enable { 1 } else { 0 };
                let vin_r_enable = if self.vin_r_enable { 1 } else { 0 };
                (vin_l_enable << 7)
                    | (self.vin_l_volume << 4)
                    | (vin_r_enable << 3)
                    | self.vin_r_volume
            }
            0xFF25 => {
                let mut value = 0;
                for i in 0..4 {
                    value |= if self.right_enables[i] { 1 << i } else { 0 };
                }

                for i in 0..4 {
                    value |= if self.left_enables[i] {
                        1 << (i + 4)
                    } else {
                        0
                    };
                }
                value
            }
            0xFF26 => {
                let mut value = 0;
                value |= if self.power_control { 1 } else { 0 } << 7;
                value |= if self.pulse_channel_1.get_status() {
                    1
                } else {
                    0
                };
                value |= if self.pulse_channel_2.get_status() {
                    1
                } else {
                    0
                } << 1;
                value |= if self.wave_channel.get_status() { 1 } else { 0 } << 2;
                value |= if self.noise_channel.get_status() {
                    1
                } else {
                    0
                } << 3;
                value
            }
            0xFF27..=0xFF2F => 0,
            0xFF30..=0xFF3F => self.wave_channel.read_byte(address),
            _ => panic!("unknown address: {:04X}", address),
        };
        value | READ_BYTE_OR_MASKS[(address - 0xFF10) as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF10..=0xFF14 => self.pulse_channel_1.write_byte(address, value),
            0xFF15..=0xFF19 => self.pulse_channel_2.write_byte(address, value),
            0xFF1A..=0xFF1E => self.wave_channel.write_byte(address, value),
            0xFF1F..=0xFF23 => self.noise_channel.write_byte(address, value),
            0xFF24 => {
                if bit_utils::is_set(value, 7) {
                    self.vin_l_enable = true;
                } else {
                    self.vin_l_enable = false;
                }

                self.vin_l_volume = (value & 0b0111_0000) >> 4;

                if bit_utils::is_set(value, 3) {
                    self.vin_r_enable = true;
                } else {
                    self.vin_r_enable = false;
                }

                self.vin_r_volume = value & 0b111;
            }
            0xFF25 => {
                for i in 0..4 {
                    self.left_enables[i] = bit_utils::is_set(value, i as u8);
                }

                for i in 0..4 {
                    self.right_enables[i] = bit_utils::is_set(value, i as u8 + 4);
                }
            }
            0xFF26 => {
                if !bit_utils::is_set(value, 7) {
                    for i in 0xFF10..=0xFF25 {
                        self.write_byte(i, 0);
                    }
                    self.power_control = false;
                } else if !self.power_control {
                    // reset the wave table
                    for i in 0xFF30..=0xFF3F {
                        self.wave_channel.write_byte(i, 0);
                    }
                    self.power_control = true;
                }
            }
            0xFF27..=0xFF2F => (),
            0xFF30..=0xFF3F => self.wave_channel.write_byte(address, value),
            _ => panic!("unknown address: {:04X}", address),
        }
    }

    pub fn get_audio_buffer(&self) -> &[f32] {
        self.audio_buffer.as_ref()
    }
}

impl Default for Sound {
    fn default() -> Sound {
        Sound::new()
    }
}
