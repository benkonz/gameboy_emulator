mod registers;
mod mode;

use self::mode::Mode;
use mmu::Memory;
use mmu::interrupt::Interrupt;
use emulator::traits::Io;

const SCAN_LINE_INDEX: u16 = 0xFF44;

pub struct GPU {
    pub pixels: [u8; 144 * 160],
    cycles: u64,
    mode: Mode,
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            pixels: [0; 144 * 160],
            cycles: 0,
            mode: Mode::HBlank,
        }
    }

    pub fn step<T: Io>(&mut self, steps: u8, memory: &mut Memory, drawer: &T) {
        self.cycles += steps as u64;
        match self.mode {
            Mode::HBlank => self.h_blank(memory, drawer),
            Mode::VBlank => self.v_blank(memory),
            Mode::OAM => self.oam(),
            Mode::VRAM => self.vram()
        }
    }

    fn h_blank<T: Io>(&mut self, memory: &mut Memory, drawer: &T) {
        if self.cycles >= 204 {

            self.increment_scanline(memory);

            if memory.read_byte(SCAN_LINE_INDEX) == 0x143 {

                self.mode = Mode::VBlank;
                drawer.draw(&self.pixels);
                memory.request_interrupt(Interrupt::Vblank);
            }
        }
    }

    fn oam(&mut self) {
        if self.cycles >= 80 {
            self.mode = Mode::VRAM;
            self.cycles = 0;
        }
    }

    fn v_blank(&mut self, memory: &mut Memory) {
        if self.cycles >= 456 {

            self.cycles = 0;
            self.increment_scanline(memory);

            if memory.read_byte(SCAN_LINE_INDEX) > 153 {
                self.mode = Mode::OAM;
                memory.write_byte(SCAN_LINE_INDEX, 0);
            }
        }
    }

    fn vram(&mut self) {
        if self.cycles >= 172 {
            self.mode = Mode::HBlank;
            self.render_scan();
            self.cycles = 0;
        }
    }

    fn increment_scanline(&self, memory: &mut Memory) {
        let mut scanline = memory.read_byte(SCAN_LINE_INDEX);
        scanline += 1;
        memory.write_byte(SCAN_LINE_INDEX, scanline);
    }

    fn render_scan(&self) {
        //TODO:: finish this
    }
}