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
}

impl Registers {
    pub fn get_af(&self) -> u16 {
        (u16::from(self.a) << 8) | u16::from(self.f.bits())
    }

    pub fn set_af(&mut self, af: u16) {
        self.a = (af >> 8) as u8;
        self.f = flag::Flag::from_bits_truncate(af as u8);
    }

    pub fn get_bc(&self) -> u16 {
        (u16::from(self.b) << 8) | u16::from(self.c)
    }

    pub fn set_bc(&mut self, bc: u16) {
        self.c = bc as u8;
        self.b = (bc >> 8) as u8;
    }

    pub fn get_de(&self) -> u16 {
        (u16::from(self.d) << 8) | u16::from(self.e)
    }

    pub fn set_de(&mut self, de: u16) {
        self.e = de as u8;
        self.d = (de >> 8) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (u16::from(self.h) << 8) | u16::from(self.l)
    }

    pub fn set_hl(&mut self, hl: u16) {
        self.l = hl as u8;
        self.h = (hl >> 8) as u8;
    }
}
