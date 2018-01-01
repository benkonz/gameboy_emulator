pub mod flag;

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub f: flag::Flag,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
    pub ime: bool,
    pub cycles: u16,
}

impl Registers {
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f.bits() as u16)
    }

    pub fn set_af(&mut self, af: u16) {
        self.a = (af >> 8) as u8;
        self.f = flag::Flag::from_bits_truncate(af as u8);
    }

    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, bc: u16) {
        self.c = bc as u8;
        self.b = (bc >> 8) as u8;
    }

    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, de: u16) {
        self.e = de as u8;
        self.d = (de >> 8) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, hl: u16) {
        self.l = hl as u8;
        self.h = (hl >> 8) as u8;
    }
}
