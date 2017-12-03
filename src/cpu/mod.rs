mod registers;

use cpu::registers::Registers;
use cpu::registers::flag::Flag;
use mmu::Memory;

const CYCLES_TABLE: [u8; 0x100] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
];

const LENGTH_TABLE: [u8; 0x100] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
];

pub struct Cpu {
    registers: Registers,
    pub stopped: bool,
    interrupt_enabled: bool
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Default::default(),
            stopped: false,
            interrupt_enabled: true
        }
    }

    pub fn step(&mut self, memory: &mut Memory) -> u8 {
        let opcode = memory.read_byte(self.registers.pc);
        let get_n = || memory.read_byte(self.registers.pc + 1);
        let get_nn = || memory.read_word(self.registers.pc + 1);

        match opcode {
            0x00 => self.nop(),
            0x01 => self.ld_bc_nn(get_nn()),
            0x02 => self.ld_bc_a(&mut memory),
            0x03 => self.inc_bc(),
            0x04 => self.inc_b(),
            0x05 => self.dec_b(),
            0x06 => self.ld_b_n(get_n()),
            0x07 => self.rlc_a(),
            0x08 => self.ld_nn_sp(get_nn(), &mut memory),
            0x09 => self.add_hl_bc(),
            0x0A => self.ld_a_bc(&memory),
            0x0B => self.dec_bc(),
            0x0C => self.inc_c(),
            0x0D => self.dec_c(),
            0x0E => self.ld_c_n(get_n()),
            0x0F => self.rrc_a(),
            0x10 => self.stop(),
            0x11 => self.ld_de_nn(get_nn()),
            0x12 => self.ld_de_a(&mut memory),
            0x13 => self.inc_de(),
            0x14 => self.inc_d(),
            0x15 => self.dec_d(),
            0x16 => self.ld_d_n(get_n()),
            0x17 => self.rl_a(),
            0x18 => self.jr_n(get_n()),
            0x19 => self.add_hl_de(),
            0x1A => self.ld_a_de(&memory),
            0x1B => self.dec_de(),
            0x1C => self.inc_e(),
            0x1D => self.dec_e(),
            0x1E => self.ld_l_n(get_n()),
            0x1F => self.rr_a(),
            0x20 => self.jr_nz_n(get_n()),
            0x21 => self.ld_hl_nn(get_nn()),
            0x22 => self.ldi_hl_a(&mut memory),
            0x23 => self.inc_hl(),
            0x24 => self.inc_h(),
            0x25 => self.dec_h(),
            0x26 => self.ld_h_n(get_n()),
            0x27 => self.daa(),
            0x28 => self.jr_z_n(get_n()),
            0x29 => self.add_hl_hl(),
            0x2A => self.ldi_a_hl(&memory),
            0x2B => self.dec_hl(),
            0x2C => self.inc_l(),
            0x2D => self.dec_l(),
            0x2E => self.ld_l_n(get_n()),
            0x2F => self.cpl(),
            0x30 => self.jr_nc_n(get_n()),
            0x31 => self.ld_hl_nn(get_nn()),
            0x32 => self.ldd_hl_a(&mut memory),
            0x33 => self.inc_sp(),
            0x34 => self.inc_hl(),
            0x35 => self.dec_hl_ref(&mut memory),
            0x36 => self.dec_hl_ref(&mut memory),
            0x37 => self.ld_hl_n(&mut memory, get_n()),
            0x38 => self.jr_c_n(get_n()),
            0x39 => self.add_hl_sp(),
            0x3A => self.ldd_a_hl(&memory),
            0x3B => self.dec_sp(),
            0x3C => self.inc_a(),
            0x3D => self.dec_a(),
            0x3E => self.ld_a_n(get_n()),
            0x3F => self.ccf(),
            0x40 => self.ld_b_b(),
            0x41 => self.ld_b_c(),
            0x42 => self.ld_b_d(),
            0x43 => self.ld_b_e(),
            0x44 => self.ld_b_h(),
            0x45 => self.ld_b_l(),
            0x46 => self.ld_b_hl(&memory),
            0x47 => self.ld_b_a(),
            0x48 => self.ld_c_b(),
            0x49 => self.ld_c_c(),
            0x4A => self.ld_c_d(),
            0x4B => self.ld_c_e(),
            0x4C => self.ld_c_h(),
            0x4D => self.ld_c_l(),
            0x4E => self.ld_c_hl(&memory),
            0x4F => self.ld_c_a(),
            0x50 => self.ld_d_b(),
            0x51 => self.ld_d_c(),
            0x52 => self.ld_d_d(),
            0x53 => self.ld_d_e(),
            0x54 => self.ld_d_h(),
            0x55 => self.ld_d_l(),
            0x56 => self.ld_d_hl(&memory),
            0x57 => self.ld_d_a(),
            0x58 => self.ld_e_b(),
            0x59 => self.ld_e_c(),
            0x5A => self.ld_e_d(),
            0x5B => self.ld_e_e(),
            0x5C => self.ld_e_h(),
            0x5D => self.ld_e_l(),
            0x5E => self.ld_e_hl(&memory),
            0x5F => self.ld_e_a(),
            0x60 => self.ld_h_b(),
            0x61 => self.ld_h_c(),
            0x62 => self.ld_h_d(),
            0x63 => self.ld_h_e(),
            0x64 => self.ld_h_h(),
            0x65 => self.ld_h_l(),
            0x66 => self.ld_h_hl(&memory),
            0x67 => self.ld_h_a(),
            0x68 => self.ld_l_b(),
            0x69 => self.ld_l_c(),
            0x6A => self.ld_l_d(),
            0x6B => self.ld_l_e(),
            0x6C => self.ld_l_h(),
            0x6D => self.ld_l_l(),
            0x6E => self.ld_l_hl(&memory),
            0x6F => self.ld_l_a(),
            0x70 => self.ld_hl_b(&mut memory),
            0x71 => self.ld_hl_c(&mut memory),
            0x72 => self.ld_hl_d(&mut memory),
            0x73 => self.ld_hl_e(&mut memory),
            0x74 => self.ld_hl_h(&mut memory),
            0x75 => self.ld_hl_l(&mut memory),
            0x76 => self.halt(),
            0x77 => self.ld_hl_a(&mut memory),
            0x78 => self.ld_a_b(),
            0x79 => self.ld_a_c(),
            0x7A => self.ld_a_d(),
            0x7B => self.ld_a_e(),
            0x7C => self.ld_a_h(),
            0x7D => self.ld_a_l(),
            0x7E => self.ld_a_hl(&memory),
            0x7F => self.ld_a_a(),
            0x80 => self.add_a_b(),
            0x81 => self.add_a_c(),
            0x82 => self.add_a_d(),
            0x83 => self.add_a_e(),
            0x84 => self.add_a_h(),
            0x85 => self.add_a_l(),
            0x86 => self.add_a_hl(&memory),
            0x87 => self.add_a_a(),
            0x88 => self.adc_a_b(),
            0x89 => self.adc_a_c(),
            0x8A => self.adc_a_d(),
            0x8B => self.adc_a_e(),
            0x8C => self.adc_a_h(),
            0x8D => self.adc_a_l(),
            0x8E => self.adc_a_hl(&memory),
            0x8F => self.adc_a_a(),
            0x90 => self.sub_a_b(),
            0x91 => self.sub_a_c(),
            0x92 => self.sub_a_d(),
            0x93 => self.sub_a_e(),
            0x94 => self.sub_a_h(),
            0x95 => self.sub_a_l(),
            0x96 => self.sub_a_hl(&memory),
            0x97 => self.sub_a_a(),
            0x98 => self.sbc_a_b(),
            0x99 => self.sbc_a_c(),
            0x9A => self.sbc_a_d(),
            0x9B => self.sbc_a_e(),
            0x9C => self.sbc_a_h(),
            0x9D => self.sbc_a_l(),
            0x9E => self.sbc_a_hl(&memory),
            0x9F => self.sbc_a_a(),
            0xA0 => self.and_b(),
            0xA1 => self.and_c(),
            0xA2 => self.and_d(),
            0xA3 => self.and_e(),
            0xA4 => self.and_h(),
            0xA5 => self.and_l(),
            0xA6 => self.and_hl(&memory),
            0xA7 => self.and_a(),
            0xA8 => self.xor_b(),
            0xA9 => self.xor_c(),
            0xAA => self.xor_d(),
            0xAB => self.xor_e(),
            0xAC => self.xor_h(),
            0xAD => self.xor_l(),
            0xAE => self.xor_hl(&memory),
            0xAF => self.xor_a(),
            0xB0 => self.or_b(),
            0xB1 => self.or_c(),
            0xB2 => self.or_d(),
            0xB3 => self.or_e(),
            0xB4 => self.or_h(),
            0xB5 => self.or_l(),
            0xB6 => self.or_hl(&memory),
            0xB7 => self.or_a(),
            0xB8 => self.cp_b(),
            0xB9 => self.cp_c(),
            0xBA => self.cp_d(),
            0xBB => self.cp_e(),
            0xBC => self.cp_h(),
            0xBD => self.cp_l(),
            0xBE => self.cp_hl(&memory),
            0xBF => self.cp_a(),
            0xC0 => self.ret_nz(&memory),
            0xC1 => self.pop_bc(&memory),
            0xC2 => self.jp_nz_nn(get_nn()),
            0xC3 => self.jp_nn(get_nn()),
            0xC4 => self.undefined(),
            0xC5 => self.call_nz_nn(&mut memory, get_nn()),
            0xC6 => self.push_bc(&mut memory),
            0xC7 => self.add_a_n(get_n()),
            0xC8 => self.rst_0(&mut memory),
            0xC9 => self.ret_z(&memory),
            0xCA => self.ret(&memory),
            0xCB => self.ext_ops(get_n(), &mut memory),
            0xCC => self.call_z_nn(get_nn(), &mut memory),
            0xCD => self.call_nn(get_nn(), &mut memory),
            0xCE => self.adc_a_n(get_n()),
            0xCF => self.rst_8(&mut memory),
            0xD0 => self.ret_nc(&memory),
            0xD1 => self.pop_de(&memory),
            0xD2 => self.jp_nc_nn(get_nn()),
            0xD3 => self.undefined(),
            0xD4 => self.call_nc_nn(get_nn(), &mut memory),
            0xD5 => self.push_de(&mut memory),
            0xD6 => self.sub_a_n(get_n()),
            0xD7 => self.rst_10(&mut memory),
            0xD8 => self.ret_c(&memory),
            0xD9 => self.ret_i(&memory),
            0xDA => self.jp_c_nn(get_nn()),
            0xDB => self.undefined(),
            0xDC => self.call_c_nn(get_nn(), &mut memory),
            0xDD => self.undefined(),
            0xDE => self.sbc_a_n(get_n()),
            0xDF => self.rst_18(&mut memory),
            0xE0 => self.ldh_n_a(get_n(), &mut memory),
            0xE1 => self.pop_hl(&memory),
            0xE2 => self.ldh_c_a(),
            0xE3 => self.undefined(),
            0xE4 => self.undefined(),
            0xE5 => self.push_hl(&mut memory),
            0xE6 => self.and_n(get_n()),
            0xE7 => self.rst_20(&mut memory),
            0xE8 => self.add_sp_d(get_n()),
            0xE9 => self.jp_hl(&memory),
            0xEA => self.ld_nn_a(&mut memory),
            0xEB => self.undefined(),
            0xEC => self.undefined(),
            0xED => self.undefined(),
            0xEE => self.xor_n(get_n()),
            0xEF => self.rst_28(&mut memory),
            0xF0 => self.ldh_a_n(get_n(), &memory),
            0xF1 => self.pop_af(&memory),
            0xF2 => self.undefined(),
            0xF3 => self.di(),
            0xF4 => self.undefined(),
            0xF5 => self.push_af(&mut memory),
            0xF6 => self.or_n(get_n()),
            0xF7 => self.rst_30(&mut memory),
            0xF8 => self.ldhl_sp_d(),
            0xF9 => self.ld_sp_hl(),
            0xFA => self.ld_a_nn(&memory),
            0xFB => self.ei(),
            0xFC => self.undefined(),
            0xFD => self.undefined(),
            0xFE => self.cp_n(get_n()),
            0xFF => self.rst_38(&mut memory)
        }

        self.registers.pc += LENGTH_TABLE[opcode];
        self.registers.cycles += CYCLES_TABLE[opcode];

        CYCLES_TABLE[opcode]
    }

    //opcodes

    fn nop(&self) {}

    fn ld_bc_nn(&mut self, nn: u16) {
        self.registers.set_bc(nn);
    }

    fn ld_bc_a(&self, memory: &mut Memory) {
        memory[self.registers.get_bc()] = self.registers.a;
    }

    fn inc_bc(&mut self) {
        let mut bc = self.registers.get_bc();
        bc = self.inc_nn(bc);
        self.registers.set_bc(bc);
    }

    fn inc_b(&mut self) {
        self.registers.b = self.inc_n(self.registers.b);
    }

    fn dec_b(&mut self) {
        self.registers.b = self.dec_n(self.registers.b);
    }

    fn ld_b_n(&mut self, n: u8) {
        self.registers.b = n;
    }

    fn rlc_a(&mut self) {
        self.registers.a = self.rlc_n(self.registers.a);
    }

    fn ld_nn_sp(&self, nn: u16, memory: &mut Memory) {
        //TODO: does this write the word or the pointer?
    }

    fn add_hl_bc(&mut self) {
        let mut hl = self.registers.get_hl();
        hl = self.add_nn_mm(hl, self.registers.get_bc());
        self.registers.set_hl(hl);
    }

    fn ld_a_bc(&mut self, memory: &Memory) {
        self.registers.a = memory[self.registers.bc];
    }

    fn dec_bc(&mut self) {
        self.registers.set_bc(self.dec_nn(self.registers.get_bc()));
    }
    fn inc_c(&mut self) {
        self.registers.c = self.inc_n(self.registers.c);
    }
    fn dec_c(&mut self) {
        self.registers.c = self.dec_n(self.registers.c);
    }

    fn ld_c_n(&mut self, n: u8) {
        self.registers.c = n;
    }

    fn rrc_a(&mut self) {
        self.registers.a = self.rrc_n(self.registers.a);
    }

    fn stop(&self) {
        self.stopped = true;
    }

    fn ld_de_nn(&mut self, nn: u16) {
        self.registers.set_de(nn);
    }

    fn ld_de_a(&self, memory: &mut Memory) {
        memory[self.registers.get_de()] = self.registers.a;
    }

    fn inc_de(&mut self) {
        let mut de = self.registers.get_de();
        de = self.inc_nn(de);
        self.registers.set_de(de);
    }

    fn inc_d(&mut self) {
        self.registers.d = self.inc_n(self.registers.d);
    }

    fn dec_d(&mut self) {
        self.registers.d = self.dec_n(self.registers.d);
    }

    fn ld_d_n(&mut self, n: u8) {
        self.registers.d = n;
    }

    fn rl_a(&mut self) {
        self.a = self.rl_n(self.registers.a);
    }

    fn jr_n(&mut self, n: u8) {
    }

    fn add_hl_de(&mut self) {
        let mut hl = self.registers.get_hl();
        self.hl = add_nn_mm(hl, self.registers.get_de());
        self.registers.set_hl(hl);
    }

    fn ld_a_de(&mut self, memory: &Memory) {
        self.registers.a = memory[self.registers.get_de()];
    }

    fn dec_de(&mut self) {
        let mut de = self.registers.get_de();
        de = dec_nn(de);
        self.registers.set_de(de);
    }

    fn inc_e(&mut self) {
        self.registers.e = inc_n(self.registers.e);
    }

    fn dec_e(&mut self) {
        self.registers.e = self.dec_n(self.registers.e);
    }

    fn ld_e_n(&mut self, n: u8) {
        self.registers.e = n;
    }

    fn rr_a(&mut self) {
        self.registers.a = rr_n(self.registers.a);
    }

    fn jr_nz_n(&mut self, n: u8) {
        self.jr_cc_n(!self.registers.f.contains(Flag::ZERO), n);
    }

    fn ld_hl_nn(&mut self, nn: u16) {
        self.registers.set_hl(nn);
    }

    fn ldi_hl_a(&mut self, memory: &mut Memory) {
        //TODO
    }

    fn inc_hl(&mut self) {
        let mut hl = self.registers.get_hl();
        hl = self.inc_nn(hl);
        self.registers.set_hl(hl);
    }

    fn inc_h(&mut self) {
        self.registers.h = self.inc_n(self.registers.h);
    }

    fn dec_h(&mut self) {
        self.registers.h = self.dec_n(self.registers.h);
    }

    fn ld_h_n(&mut self, n: u8) {
        self.registers.h = n;
    }

    fn daa(&mut self) {
        //TODO:
    }

    fn jr_z_n(&mut self, n: u8) {
        self.jr_cc_n(self.registers.f.contains(Flag::ZERO), n);
    }

    fn add_hl_hl(&mut self) {
        let mut hl = self.registers.get_hl();
        hl = add_mm_nn(hl, hl);
        self.registers.set_hl(hl);
    }

    fn ldi_a_hl(&mut self, memory: &Memory) {
        //TODO
    }

    fn dec_hl(&mut self) {
        let mut hl = self.registers.get_hl();
        hl = self.dec_nn(hl);
        self.registers.set_hl(hl);
    }

    fn inc_l(&mut self) {
        self.registers.l = self.inc_n(self.registers.l);
    }

    fn dec_l(&mut self) {
        self.registers.l = self.dec_n(self.registers.l);
    }

    fn ld_l_n(&mut self, n: u8) {
        self.registers.l = n;
    }

    fn cpl(&mut self) {
        //TODO:
    }

    fn jr_nc_n(&mut self, n: u8) {
        let cc = !self.registers.f.contains(Flag::FULL_CARRY);
        self.jr_cc_n(cc, n);
    }

    fn ld_sp_nn(&mut self, nn: u16) {
        self.registers.sp = nn;
    }

    fn ldd_hl_a(&self, memory: &mut Memory) {
        //TODO
    }

    fn inc_sp(&mut self) {
        self.registers.sp = self.inc_nn(self.registers.sp);
    }

    fn inc_hl_ref(&self, memory: &mut Memory) {
        let mut hl = self.registers.get_hl();
        memroy[hl] = self.inc_n(hl);
    }

    fn dec_hl_ref(&self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory[hl] = self.dec_n(memory[hl]);
    }

    fn ld_hl_n(&self, memory: &mut Memory, n: u8) {
        memory[self.registers.get_hl()] = n;
    }

    fn scf(&mut self) {
        self.registers.f.set(Flag::FULL_CARRY);
    }

    fn jr_c_n(&mut self, n: u8) {
        self.registers.c = n;
    }

    fn add_hl_sp(&mut self) {
        let mut hl = self.registers.get_hl();
        hl = add_mm_nn(hl, self.registers.sp);
        self.registers.set_hl(hl);
    }

    fn ldd_a_hl(&mut self, memory: &Memory) {}

    fn dec_sp(&mut self) {
        self.registers.sp = self.dec_nn(self.registers.sp);
    }

    fn inc_a(&mut self) {
        self.registers.a = self.inc_n(self.registers.a);
    }

    fn dec_a(&mut self) {
        self.registers.a = self.dec_n(self.registers.a);
    }

    fn ld_a_n(&mut self, n: u8) {
        self.registers.a = n;
    }

    fn ccf(&mut self) {
        self.registers.f.toggle(Flag::FULL_CARRY);
    }

    fn ld_b_b(&mut self) { self.registers.b = self.registers.b; }
    fn ld_b_c(&mut self) { self.registers.b = self.registers.c; }
    fn ld_b_d(&mut self) { self.registers.b = self.registers.d; }
    fn ld_b_e(&mut self) { self.registers.b = self.registers.e; }
    fn ld_b_h(&mut self) { self.registers.b = self.registers.h; }
    fn ld_b_l(&mut self) { self.registers.b = self.registers.l; }
    fn ld_b_hl(&mut self, memory: &Memory) { self.registers.b = memory[self.registers.get_hl()]; }
    fn ld_b_a(&mut self) { self.registers.b = self.registers.a; }

    fn ld_c_b(&mut self) { self.registers.c = self.registers.b; }
    fn ld_c_c(&mut self) { self.registers.c = self.registers.c; }
    fn ld_c_d(&mut self) { self.registers.c = self.registers.d; }
    fn ld_c_e(&mut self) { self.registers.c = self.registers.e; }
    fn ld_c_h(&mut self) { self.registers.c = self.registers.h; }
    fn ld_c_l(&mut self) { self.registers.c = self.registers.l; }
    fn ld_c_hl(&mut self, memory: &Memory) { self.registers.c = memory[self.registers.get_hl()]; }
    fn ld_c_a(&mut self) { self.registers.c = self.registers.a; }

    fn ld_d_b(&mut self) {self.registers.d = self.registers.b;}
    fn ld_d_c(&mut self) {self.registers.d = self.registers.c;}
    fn ld_d_d(&mut self) {self.registers.d = self.registers.d;}
    fn ld_d_e(&mut self) {self.registers.d = self.registers.e;}
    fn ld_d_h(&mut self) {self.registers.d = self.registers.h;}
    fn ld_d_l(&mut self) {self.registers.d = self.registers.l;}
    fn ld_d_hl(&mut self, memory: &Memory) {self.registers.d = memory[self.registers.get_hl()];}
    fn ld_d_a(&mut self) {self.registers.d = self.registers.a;}

    fn ld_e_b(&mut self) {self.registers.e = self.registers.b;}
    fn ld_e_c(&mut self) {self.registers.e = self.registers.c;}
    fn ld_e_d(&mut self) {self.registers.e = self.registers.d;}
    fn ld_e_e(&mut self) {self.registers.e = self.registers.e;}
    fn ld_e_h(&mut self) {self.registers.e = self.registers.h;}
    fn ld_e_l(&mut self) {self.registers.e = self.registers.l;}
    fn ld_e_hl(&mut self, memory: &Memory) {self.registers.e = memory[self.registers.get_hl()];}
    fn ld_e_a(&mut self) {self.registers.e = self.registers.a;}

    fn ld_h_b(&mut self) {self.registers.h = self.registers.b;}
    fn ld_h_c(&mut self) {self.registers.h = self.registers.c;}
    fn ld_h_d(&mut self) {self.registers.h = self.registers.d;}
    fn ld_h_e(&mut self) {self.registers.h = self.registers.e;}
    fn ld_h_h(&mut self) {self.registers.h = self.registers.h;}
    fn ld_h_l(&mut self) {self.registers.h = self.registers.l;}
    fn ld_h_hl(&mut self, memory: &Memory) {self.registers.h = memory[self.registers.get_hl()];}
    fn ld_h_a(&mut self) {self.registers.h = self.registers.a;}

    fn ld_l_b(&mut self) {self.registers.l = self.registers.b;}
    fn ld_l_c(&mut self) {self.registers.l = self.registers.c;}
    fn ld_l_d(&mut self) {self.registers.l = self.registers.d;}
    fn ld_l_e(&mut self) {self.registers.l = self.registers.e;}
    fn ld_l_h(&mut self) {self.registers.l = self.registers.h;}
    fn ld_l_l(&mut self) {self.registers.l = self.registers.l;}
    fn ld_l_hl(&mut self, memory: &Memory) {self.registers.l = memory[self.registers.get_hl()];}
    fn ld_l_a(&mut self) {self.registers.l = self.registers.a;}

    fn ld_hl_b(&self, memory: &mut Memory) {memory[self.registers.get_hl()] = self.registers.b;}
    fn ld_hl_c(&self, memory: &mut Memory) {memory[self.registers.get_hl()] = self.registers.c;}
    fn ld_hl_d(&self, memory: &mut Memory) {memory[self.registers.get_hl()] = self.registers.d;}
    fn ld_hl_e(&self, memory: &mut Memory) {memory[self.registers.get_hl()] = self.registers.e;}
    fn ld_hl_h(&self, memory: &mut Memory) {memory[self.registers.get_hl()] = self.registers.h;}
    fn ld_hl_l(&self, memory: &mut Memory) {memory[self.registers.get_hl()] = self.registers.l;}

    fn halt(&self) {}

    fn ld_hl_a(&mut self, memory: &mut Memory) {memory[self.registers.get_hl()] = self.registers.a;}

    fn ld_a_b(&mut self) {self.registers.a = self.registers.b;}
    fn ld_a_c(&mut self) {self.registers.a = self.registers.c;}
    fn ld_a_d(&mut self) {self.registers.a = self.registers.d;}
    fn ld_a_e(&mut self) {self.registers.a = self.registers.e;}
    fn ld_a_h(&mut self) {self.registers.a = self.registers.h;}
    fn ld_a_l(&mut self) {self.registers.a = self.registers.l;}
    fn ld_a_hl(&mut self, memory: &Memory) {self.registers.a = memory[self.registers.get_hl()];}
    fn ld_a_a(&mut self) {self.registers.a = self.registers.a;}

    fn add_a_b(&mut self) {self.add_n(self.registers.b);}
    fn add_a_c(&mut self) {self.add_n(self.registers.c);}
    fn add_a_d(&mut self) {self.add_n(self.registers.d);}
    fn add_a_e(&mut self) {self.add_n(self.registers.e);}
    fn add_a_h(&mut self) {self.add_n(self.registers.h);}
    fn add_a_l(&mut self) {self.add_n(self.registers.l);}
    fn add_a_hl(&mut self, memory: &Memory) { self.add_a_n(memory[self.registers.get_hl()]); }
    fn add_a_a(&mut self) {self.add_a_n(self.registers.a);}

    fn adc_a_b(&mut self) {self.adc_a_n(self.registers.b);}
    fn adc_a_c(&mut self) {self.adc_a_n(self.registers.c);}
    fn adc_a_d(&mut self) {self.adc_a_n(self.registers.d);}
    fn adc_a_e(&mut self) {self.adc_a_n(self.registers.e);}
    fn adc_a_h(&mut self) {self.adc_a_n(self.registers.h);}
    fn adc_a_l(&mut self) {self.adc_a_n(self.registers.l);}
    fn adc_a_hl(&mut self, memory: &Memory) {self.adc_a_n(memory[self.registers.get_hl()]);}
    fn adc_a_a(&mut self) {self.adc_a_n(self.registers.a);}

    fn sub_a_b(&mut self) {self.sub_a_n(self.registers.b);}
    fn sub_a_c(&mut self) {self.sub_a_n(self.registers.c);}
    fn sub_a_d(&mut self) {self.sub_a_n(self.registers.d);}
    fn sub_a_e(&mut self) {self.sub_a_n(self.registers.e);}
    fn sub_a_h(&mut self) {self.sub_a_n(self.registers.h);}
    fn sub_a_l(&mut self) {self.sub_a_n(self.registers.l);}
    fn sub_a_hl(&mut self, memory: &Memory) {self.sub_a_n(memory[self.registers.get_hl()]);}
    fn sub_a_a(&mut self) {self.sub_a_n(self.registers.a);}

    fn sbc_a_b(&mut self) {}
    fn sbc_a_c(&mut self) {}
    fn sbc_a_d(&mut self) {}
    fn sbc_a_e(&mut self) {}
    fn sbc_a_h(&mut self) {}
    fn sbc_a_l(&mut self) {}
    fn sbc_a_hl(&mut self, memory: &Memory) {}
    fn sbc_a_a(&mut self) {}

    fn and_b(&mut self) {}
    fn and_c(&mut self) {}
    fn and_d(&mut self) {}
    fn and_e(&mut self) {}
    fn and_h(&mut self) {}
    fn and_l(&mut self) {}
    fn and_hl(&mut self, memory: &Memory) {}
    fn and_a(&mut self) {}

    fn xor_b(&mut self) {}
    fn xor_c(&mut self) {}
    fn xor_d(&mut self) {}
    fn xor_e(&mut self) {}
    fn xor_h(&mut self) {}
    fn xor_l(&mut self) {}
    fn xor_hl(&mut self, memory: &Memory) {}
    fn xor_a(&mut self) {}

    fn or_b(&mut self) {}
    fn or_c(&mut self) {}
    fn or_d(&mut self) {}
    fn or_e(&mut self) {}
    fn or_h(&mut self) {}
    fn or_l(&mut self) {}
    fn or_hl(&mut self, memory: &Memory) {}
    fn or_a(&mut self) {}

    fn cp_b(&mut self) {}
    fn cp_c(&mut self) {}
    fn cp_d(&mut self) {}
    fn cp_e(&mut self) {}
    fn cp_h(&mut self) {}
    fn cp_l(&mut self) {}
    fn cp_hl(&mut self, memory: &Memory) {}
    fn cp_a(&mut self) {}

    fn ret_nz(&mut self, memory: &Memory) {}

    fn pop_bc(&mut self, memory: &Memory) {}

    fn jp_nz_nn(&mut self, nn: u16) {}

    fn jp_nn(&mut self, nn: u16) {}

    fn call_nz_nn(&mut self, memory: &mut Memory, nn: u16) {}

    fn push_bc(&mut self, memory: &mut Memory) {}

    fn add_a_n(&mut self, n: u8) {}

    fn rst_0(&mut self, memory: &mut Memory) {}

    fn ret_z(&mut self, memory: &Memory) {}

    fn ret(&mut self, memory: &Memory) {}

    fn jp_z_nn(&mut self, nn: u16) {}

    fn ext_ops(&mut self, opcode: u8, memory: &mut Memory) {
        match opcode {
            0x00 => self.rlc_b(),
            0x01 => self.rlc_c(),
            0x02 => self.rlc_d(),
            0x03 => self.rlc_e(),
            0x04 => self.rlc_h(),
            0x05 => self.rlc_l(),
            0x06 => self.rlc_hl(&mut memory),
            0x07 => self.rlc_a(),
            0x08 => self.rrc_b(),
            0x09 => self.rrc_c(),
            0x0A => self.rrc_d(),
            0x0B => self.rrc_e(),
            0x0C => self.rrc_h(),
            0x0D => self.rrc_l(),
            0x0E => self.rrc_hl(&mut memory),
            0x0F => self.rrc_a(),
            0x10 => self.rl_b(),
            0x11 => self.rl_c(),
            0x12 => self.rl_d(),
            0x13 => self.rl_e(),
            0x14 => self.rl_h(),
            0x15 => self.rl_l(),
            0x16 => self.rl_hl(&mut memory),
            0x17 => self.rl_a(),
            0x18 => self.rr_b(),
            0x19 => self.rr_c(),
            0x1A => self.rr_d(),
            0x1B => self.rr_e(),
            0x1C => self.rr_h(),
            0x1D => self.rr_l(),
            0x1E => self.rr_hl(&mut memory),
            0x1F => self.rr_a(),
            0x20 => self.sla_b(),
            0x21 => self.sla_c(),
            0x22 => self.sla_d(),
            0x23 => self.sla_e(),
            0x24 => self.sla_h(),
            0x25 => self.sla_l(),
            0x26 => self.sla_hl(&mut memory),
            0x27 => self.sla_a(),
            0x28 => self.sra_b(),
            0x29 => self.sra_c(),
            0x2A => self.sra_d(),
            0x2B => self.sra_e(),
            0x2C => self.sra_h(),
            0x2D => self.sra_l(),
            0x2E => self.sra_hl(&mut memory),
            0x2F => self.sra_a(),
            0x30 => self.swap_b(),
            0x31 => self.swap_c(),
            0x32 => self.swap_d(),
            0x33 => self.swap_e(),
            0x34 => self.swap_h(),
            0x35 => self.swap_l(),
            0x36 => self.swap_hl(&mut memory),
            0x37 => self.swap_a(),
            0x38 => self.srl_b(),
            0x39 => self.srl_c(),
            0x3A => self.srl_d(),
            0x3B => self.srl_e(),
            0x3C => self.srl_h(),
            0x3D => self.srl_l(),
            0x3E => self.srl_hl(&mut memory),
            0x3F => self.srl_a(),
            0x40 => self.bit_0_b(),
            0x41 => self.bit_0_c(),
            0x42 => self.bit_0_d(),
            0x43 => self.bit_0_e(),
            0x44 => self.bit_0_h(),
            0x45 => self.bit_0_l(),
            0x46 => self.bit_0_hl(&memory),
            0x47 => self.bit_0_a(),
            0x48 => self.bit_1_b(),
            0x49 => self.bit_1_c(),
            0x4A => self.bit_1_d(),
            0x4B => self.bit_1_e(),
            0x4C => self.bit_1_h(),
            0x4D => self.bit_1_l(),
            0x4E => self.bit_1_hl(&memory),
            0x4F => self.bit_1_a(),
            0x50 => self.bit_2_b(),
            0x51 => self.bit_2_c(),
            0x52 => self.bit_2_d(),
            0x53 => self.bit_2_e(),
            0x54 => self.bit_2_h(),
            0x55 => self.bit_2_l(),
            0x56 => self.bit_2_hl(&memory),
            0x57 => self.bit_2_a(),
            0x58 => self.bit_3_b(),
            0x59 => self.bit_3_c(),
            0x5A => self.bit_3_d(),
            0x5B => self.bit_3_e(),
            0x5C => self.bit_3_h(),
            0x5D => self.bit_3_l(),
            0x5E => self.bit_3_hl(&memory),
            0x5F => self.bit_3_a(),
            0x60 => self.bit_4_b(),
            0x61 => self.bit_4_c(),
            0x62 => self.bit_4_d(),
            0x63 => self.bit_4_e(),
            0x64 => self.bit_4_h(),
            0x65 => self.bit_4_l(),
            0x66 => self.bit_4_hl(&memory),
            0x67 => self.bit_4_a(),
            0x68 => self.bit_5_b(),
            0x69 => self.bit_5_c(),
            0x6A => self.bit_5_d(),
            0x6B => self.bit_5_e(),
            0x6C => self.bit_5_h(),
            0x6D => self.bit_5_l(),
            0x6E => self.bit_5_hl(&memory),
            0x6F => self.bit_5_a(),
            0x70 => self.bit_6_b(),
            0x71 => self.bit_6_c(),
            0x72 => self.bit_6_d(),
            0x73 => self.bit_6_e(),
            0x74 => self.bit_6_h(),
            0x75 => self.bit_6_l(),
            0x76 => self.bit_6_hl(&memory),
            0x77 => self.bit_6_a(),
            0x78 => self.bit_7_b(),
            0x79 => self.bit_7_c(),
            0x7A => self.bit_7_d(),
            0x7B => self.bit_7_e(),
            0x7C => self.bit_7_h(),
            0x7D => self.bit_7_l(),
            0x7E => self.bit_7_hl(&memory),
            0x7F => self.bit_7_a(),
            0x80 => self.res_0_b(),
            0x81 => self.res_0_c(),
            0x82 => self.res_0_d(),
            0x83 => self.res_0_e(),
            0x84 => self.res_0_h(),
            0x85 => self.res_0_l(),
            0x86 => self.res_0_hl(&memory),
            0x87 => self.res_0_a(),
            0x88 => self.res_1_b(),
            0x89 => self.res_1_c(),
            0x8A => self.res_1_d(),
            0x8B => self.res_1_e(),
            0x8C => self.res_1_h(),
            0x8D => self.res_1_l(),
            0x8E => self.res_1_hl(&memory),
            0x8F => self.res_1_a(),
            0x90 => self.res_2_b(),
            0x91 => self.res_2_c(),
            0x92 => self.res_2_d(),
            0x93 => self.res_2_e(),
            0x94 => self.res_2_h(),
            0x95 => self.res_2_l(),
            0x96 => self.res_2_hl(&memory),
            0x97 => self.res_2_a(),
            0x98 => self.res_3_b(),
            0x99 => self.res_3_c(),
            0x9A => self.res_3_d(),
            0x9B => self.res_3_e(),
            0x9C => self.res_3_h(),
            0x9D => self.res_3_l(),
            0x9E => self.res_3_hl(&memory),
            0x9F => self.res_3_a(),
            0xA0 => self.res_4_b(),
            0xA1 => self.res_4_c(),
            0xA2 => self.res_4_d(),
            0xA3 => self.res_4_e(),
            0xA4 => self.res_4_h(),
            0xA5 => self.res_4_l(),
            0xA6 => self.res_4_hl(&memory),
            0xA7 => self.res_4_a(),
            0xA8 => self.res_5_b(),
            0xA9 => self.res_5_c(),
            0xAA => self.res_5_d(),
            0xAB => self.res_5_e(),
            0xAC => self.res_5_h(),
            0xAD => self.res_5_l(),
            0xAE => self.res_5_hl(&memory),
            0xAF => self.res_5_a(),
            0xB0 => self.res_6_b(),
            0xB1 => self.res_6_c(),
            0xB2 => self.res_6_d(),
            0xB3 => self.res_6_e(),
            0xB4 => self.res_6_h(),
            0xB5 => self.res_6_l(),
            0xB6 => self.res_6_hl(&memory),
            0xB7 => self.res_6_a(),
            0xB8 => self.res_7_b(),
            0xB9 => self.res_7_c(),
            0xBA => self.res_7_d(),
            0xBB => self.res_7_e(),
            0xBC => self.res_7_h(),
            0xBD => self.res_7_l(),
            0xBE => self.res_7_hl(&memory),
            0xBF => self.res_7_a(),
            0xC0 => self.set_0_b(),
            0xC1 => self.set_0_c(),
            0xC2 => self.set_0_d(),
            0xC3 => self.set_0_e(),
            0xC4 => self.set_0_h(),
            0xC5 => self.set_0_l(),
            0xC6 => self.set_0_hl(&memory),
            0xC7 => self.set_0_a(),
            0xC8 => self.set_1_b(),
            0xC9 => self.set_1_c(),
            0xCA => self.set_1_d(),
            0xCB => self.set_1_e(),
            0xCC => self.set_1_h(),
            0xCD => self.set_1_l(),
            0xCE => self.set_1_hl(&memory),
            0xCF => self.set_1_a(),
            0xD0 => self.set_2_b(),
            0xD1 => self.set_2_c(),
            0xD2 => self.set_2_d(),
            0xD3 => self.set_2_e(),
            0xD4 => self.set_2_h(),
            0xD5 => self.set_2_l(),
            0xD6 => self.set_2_hl(&memory),
            0xD7 => self.set_2_a(),
            0xD8 => self.set_3_b(),
            0xD9 => self.set_3_c(),
            0xDA => self.set_3_d(),
            0xDB => self.set_3_e(),
            0xDC => self.set_3_h(),
            0xDD => self.set_3_l(),
            0xDE => self.set_3_hl(&memory),
            0xDF => self.set_3_a(),
            0xE0 => self.set_4_b(),
            0xE1 => self.set_4_c(),
            0xE2 => self.set_4_d(),
            0xE3 => self.set_4_e(),
            0xE4 => self.set_4_h(),
            0xE5 => self.set_4_l(),
            0xE6 => self.set_4_hl(&memory),
            0xE7 => self.set_4_a(),
            0xE8 => self.set_5_b(),
            0xE9 => self.set_5_c(),
            0xEA => self.set_5_d(),
            0xEB => self.set_5_e(),
            0xEC => self.set_5_h(),
            0xED => self.set_5_l(),
            0xEE => self.set_5_hl(&memory),
            0xEF => self.set_5_a(),
            0xF0 => self.set_6_b(),
            0xF1 => self.set_6_c(),
            0xF2 => self.set_6_d(),
            0xF3 => self.set_6_e(),
            0xF4 => self.set_6_h(),
            0xF5 => self.set_6_l(),
            0xF6 => self.set_6_hl(&memory),
            0xF7 => self.set_6_a(),
            0xF8 => self.set_7_b(),
            0xF9 => self.set_7_c(),
            0xFA => self.set_7_d(),
            0xFB => self.set_7_e(),
            0xFC => self.set_7_h(),
            0xFD => self.set_7_l(),
            0xFE => self.set_7_hl(&memory),
            0xFF => self.set_7_a(),
        }
    }

    fn call_z_nn(&mut self, nn: u16, memory: &Memory) {}

    fn call_nn(&mut self, nn: u16, memory: &mut Memory) {}

    fn adc_a_n(&mut self, n: u8) {}

    fn rst_8(&mut self, memory: &mut Memory) {}

    fn ret_nc(&mut self, memory: &Memory) {}

    fn pop_de(&mut self, memory: &Memory) {}

    fn jp_nc_nn(&mut self, nn: u16) {}

    fn undefined(&mut self) {}

    fn call_nc_nn(&mut self, nn: u16, memory: &mut Memory) {}

    fn push_de(&self, memory: &mut Memory) {}

    fn sub_a_n(&mut self, n: u8) {}

    fn rst_10(&mut self, memory: &mut Memory) {}

    fn ret_c(&mut self, memory: &Memory) {}

    fn ret_i(&self, memory: &Memory) {}

    fn jp_c_nn(&mut self, nn: u16) {}

    fn call_c_nn(&mut self, nn: u16, memory: &mut Memory) {}

    fn sbc_a_n(&mut self, n: u8) {}

    fn rst_18(&mut self, memory: &mut Memory) {}

    fn ldh_n_a(&self, n: u8, memory: &mut Memory) {}

    fn pop_hl(&mut self, memory: &Memory) {}

    fn ldh_c_a(&mut self) {}

    fn push_hl(&self, memory: &mut Memory) {}

    fn and_n(&mut self, n: u8) {}

    fn rst_20(&mut self, memory: &mut Memory) {}

    fn add_sp_d(&mut self, d: u8) {}

    fn jp_hl(&mut self, memory: &Memory) {}

    fn ld_nn_a(&self, memory: &mut Memory) {}

    fn xor_n(&mut self, n: u8) {}

    fn rst_28(&mut self, memory: &mut Memory) {}

    fn ldh_a_n(&mut self, n: u8, memory: &Memory) {}

    fn pop_af(&mut self, memory: &Memory) {}

    fn di(&mut self) {}

    fn push_af(&self, memory: &mut Memory) {}

    fn or_n(&mut self, n: u8) {}

    fn rst_30(&mut self, memory: &mut Memory) {}

    fn ldhl_sp_d(&mut self) {}

    fn ld_sp_hl(&mut self) {}

    fn ld_a_nn(&mut self, memory: &Memory) {}

    fn ei(&mut self) {}

    fn cp_n(&mut self, n: u8) {}

    fn rst_38(&mut self, memory: &mut Memory) {}

    //extended opcodes

    fn rlc_b(&mut self) {}
    fn rlc_c(&mut self) {}
    fn rlc_d(&mut self) {}
    fn rlc_e(&mut self) {}
    fn rlc_h(&mut self) {}
    fn rlc_l(&mut self) {}
    fn rlc_hl(&mut self, memory: &Memory) {}
    fn rrc_b(&mut self) {}
    fn rrc_c(&mut self) {}
    fn rrc_d(&mut self) {}
    fn rrc_e(&mut self) {}
    fn rrc_h(&mut self) {}
    fn rrc_l(&mut self) {}
    fn rrc_hl(&mut self, memory: &Memory) {}
    fn rl_b(&mut self) {}
    fn rl_c(&mut self) {}
    fn rl_d(&mut self) {}
    fn rl_e(&mut self) {}
    fn rl_h(&mut self) {}
    fn rl_l(&mut self) {}
    fn rl_hl(&mut self, memory: &Memory) {}
    fn rr_b(&mut self) {}
    fn rr_c(&mut self) {}
    fn rr_d(&mut self) {}
    fn rr_e(&mut self) {}
    fn rr_h(&mut self) {}
    fn rr_l(&mut self) {}
    fn rr_hl(&mut self, memory: &Memory) {}
    fn sla_b(&mut self) {}
    fn sla_c(&mut self) {}
    fn sla_d(&mut self) {}
    fn sla_e(&mut self) {}
    fn sla_h(&mut self) {}
    fn sla_l(&mut self) {}
    fn sla_hl(&mut self, memory: &Memory) {}
    fn sla_a(&mut self) {}
    fn sra_b(&mut self) {}
    fn sra_c(&mut self) {}
    fn sra_d(&mut self) {}
    fn sra_e(&mut self) {}
    fn sra_h(&mut self) {}
    fn sra_l(&mut self) {}
    fn sra_hl(&mut self, memory: &Memory) {}
    fn sra_a(&mut self) {}
    fn swap_b(&mut self) {}
    fn swap_c(&mut self) {}
    fn swap_d(&mut self) {}
    fn swap_e(&mut self) {}
    fn swap_h(&mut self) {}
    fn swap_l(&mut self) {}
    fn swap_hl(&mut self, memory: &Memory) {}
    fn swap_a(&mut self) {}
    fn srl_b(&mut self) {}
    fn srl_c(&mut self) {}
    fn srl_d(&mut self) {}
    fn srl_e(&mut self) {}
    fn srl_h(&mut self) {}
    fn srl_l(&mut self) {}
    fn srl_hl(&mut self, memory: &Memory) {}
    fn srl_a(&mut self) {}
    fn bit_0_b(&mut self) {self.bit_i_n(0, self.registers.b);}
    fn bit_0_c(&mut self) {self.bit_i_n(0, self.registers.c);}
    fn bit_0_d(&mut self) {self.bit_i_n(0, self.registers.d);}
    fn bit_0_e(&mut self) {self.bit_i_n(0, self.registers.e);}
    fn bit_0_h(&mut self) {self.bit_i_n(0, self.registers.h);}
    fn bit_0_l(&mut self) {self.bit_i_n(0, self.registers.l);}
    fn bit_0_hl(&mut self, memory: &Memory) {self.bit_i_n(0, memory[self.registers.get_hl()]);}
    fn bit_0_a(&mut self) {self.registers.bit_i_n(0, self.registers.a);}
    fn bit_1_b(&mut self) {self.bit_i_n(1, self.registers.b);}
    fn bit_1_c(&mut self) {self.bit_i_n(1, self.registers.c);}
    fn bit_1_d(&mut self) {self.bit_i_n(1, self.registers.d);}
    fn bit_1_e(&mut self) {self.bit_i_n(1, self.registers.e);}
    fn bit_1_h(&mut self) {self.bit_i_n(1, self.registers.h);}
    fn bit_1_l(&mut self) {self.bit_i_n(1, self.registers.l);}
    fn bit_1_hl(&mut self, memory: &Memory) {self.bit_i_n(1, memory[self.registers.get_hl()]);}
    fn bit_1_a(&mut self) {self.registers.bit_i_n(1, self.registers.a);}
    fn bit_2_b(&mut self) {self.bit_i_n(2, self.registers.b);}
    fn bit_2_c(&mut self) {self.bit_i_n(2, self.registers.c);}
    fn bit_2_d(&mut self) {self.bit_i_n(2, self.registers.d);}
    fn bit_2_e(&mut self) {self.bit_i_n(2, self.registers.e);}
    fn bit_2_h(&mut self) {self.bit_i_n(2, self.registers.h);}
    fn bit_2_l(&mut self) {self.bit_i_n(2, self.registers.l);}
    fn bit_2_hl(&mut self, memory: &Memory) {self.bit_i_n(2, memory[self.registers.get_hl()]);}
    fn bit_2_a(&mut self) {self.registers.bit_i_n(2, self.registers.a);}
    fn bit_3_b(&mut self) {self.bit_i_n(3, self.registers.b);}
    fn bit_3_c(&mut self) {self.bit_i_n(3, self.registers.c);}
    fn bit_3_d(&mut self) {self.bit_i_n(3, self.registers.d);}
    fn bit_3_e(&mut self) {self.bit_i_n(3, self.registers.e);}
    fn bit_3_h(&mut self) {self.bit_i_n(3, self.registers.h);}
    fn bit_3_l(&mut self) {self.bit_i_n(3, self.registers.l);}
    fn bit_3_hl(&mut self, memory: &Memory) {self.bit_i_n(3, memory[self.registers.get_hl()]);}
    fn bit_3_a(&mut self) {self.registers.bit_i_n(3, self.registers.a);}
    fn bit_4_b(&mut self) {self.bit_i_n(4, self.registers.b);}
    fn bit_4_c(&mut self) {self.bit_i_n(4, self.registers.c);}
    fn bit_4_d(&mut self) {self.bit_i_n(4, self.registers.d);}
    fn bit_4_e(&mut self) {self.bit_i_n(4, self.registers.e);}
    fn bit_4_h(&mut self) {self.bit_i_n(4, self.registers.h);}
    fn bit_4_l(&mut self) {self.bit_i_n(4, self.registers.l);}
    fn bit_4_hl(&mut self, memory: &Memory) {self.bit_i_n(4, memory[self.registers.get_hl()]);}
    fn bit_4_a(&mut self) {self.registers.bit_i_n(4, self.registers.a);}
    fn bit_5_b(&mut self) {self.bit_i_n(5, self.registers.b);}
    fn bit_5_c(&mut self) {self.bit_i_n(5, self.registers.c);}
    fn bit_5_d(&mut self) {self.bit_i_n(5, self.registers.d);}
    fn bit_5_e(&mut self) {self.bit_i_n(5, self.registers.e);}
    fn bit_5_h(&mut self) {self.bit_i_n(5, self.registers.h);}
    fn bit_5_l(&mut self) {self.bit_i_n(5, self.registers.l);}
    fn bit_5_hl(&mut self, memory: &Memory) {self.bit_i_n(5, memory[self.registers.get_hl()]);}
    fn bit_5_a(&mut self) {self.registers.bit_i_n(5, self.registers.a);}
    fn bit_6_b(&mut self) {self.bit_i_n(6, self.registers.b);}
    fn bit_6_c(&mut self) {self.bit_i_n(6, self.registers.c);}
    fn bit_6_d(&mut self) {self.bit_i_n(6, self.registers.d);}
    fn bit_6_e(&mut self) {self.bit_i_n(6, self.registers.e);}
    fn bit_6_h(&mut self) {self.bit_i_n(6, self.registers.h);}
    fn bit_6_l(&mut self) {self.bit_i_n(6, self.registers.l);}
    fn bit_6_hl(&mut self, memory: &Memory) {self.bit_i_n(6, memory[self.registers.get_hl()]);}
    fn bit_6_a(&mut self) {self.registers.bit_i_n(6, self.registers.a);}
    fn bit_7_b(&mut self) {self.bit_i_n(7, self.registers.b);}
    fn bit_7_c(&mut self) {self.bit_i_n(7, self.registers.c);}
    fn bit_7_d(&mut self) {self.bit_i_n(7, self.registers.d);}
    fn bit_7_e(&mut self) {self.bit_i_n(7, self.registers.e);}
    fn bit_7_h(&mut self) {self.bit_i_n(7, self.registers.h);}
    fn bit_7_l(&mut self) {self.bit_i_n(7, self.registers.l);}
    fn bit_7_hl(&mut self, memory: &Memory) {self.bit_i_n(7, memory[self.registers.get_hl()]);}
    fn bit_7_a(&mut self) {self.registers.bit_i_n(7, self.registers.a);}
    fn res_0_b(&mut self) {}
    fn res_0_c(&mut self) {}
    fn res_0_d(&mut self) {}
    fn res_0_e(&mut self) {}
    fn res_0_h(&mut self) {}
    fn res_0_l(&mut self) {}
    fn res_0_hl(&mut self, memory: &Memory) {}
    fn res_0_a(&mut self) {}
    fn res_1_b(&mut self) {}
    fn res_1_c(&mut self) {}
    fn res_1_d(&mut self) {}
    fn res_1_e(&mut self) {}
    fn res_1_h(&mut self) {}
    fn res_1_l(&mut self) {}
    fn res_1_hl(&mut self, memory: &Memory) {}
    fn res_1_a(&mut self) {}
    fn res_2_b(&mut self) {}
    fn res_2_c(&mut self) {}
    fn res_2_d(&mut self) {}
    fn res_2_e(&mut self) {}
    fn res_2_h(&mut self) {}
    fn res_2_l(&mut self) {}
    fn res_2_hl(&mut self, memory: &Memory) {}
    fn res_2_a(&mut self) {}
    fn res_3_b(&mut self) {}
    fn res_3_c(&mut self) {}
    fn res_3_d(&mut self) {}
    fn res_3_e(&mut self) {}
    fn res_3_h(&mut self) {}
    fn res_3_l(&mut self) {}
    fn res_3_hl(&mut self, memory: &Memory) {}
    fn res_3_a(&mut self) {}
    fn res_4_b(&mut self) {}
    fn res_4_c(&mut self) {}
    fn res_4_d(&mut self) {}
    fn res_4_e(&mut self) {}
    fn res_4_h(&mut self) {}
    fn res_4_l(&mut self) {}
    fn res_4_hl(&mut self, memory: &Memory) {}
    fn res_4_a(&mut self) {}
    fn res_5_b(&mut self) {}
    fn res_5_c(&mut self) {}
    fn res_5_d(&mut self) {}
    fn res_5_e(&mut self) {}
    fn res_5_h(&mut self) {}
    fn res_5_l(&mut self) {}
    fn res_5_hl(&mut self, memory: &Memory) {}
    fn res_5_a(&mut self) {}
    fn res_6_b(&mut self) {}
    fn res_6_c(&mut self) {}
    fn res_6_d(&mut self) {}
    fn res_6_e(&mut self) {}
    fn res_6_h(&mut self) {}
    fn res_6_l(&mut self) {}
    fn res_6_hl(&mut self, memory: &Memory) {}
    fn res_6_a(&mut self) {}
    fn res_7_b(&mut self) {}
    fn res_7_c(&mut self) {}
    fn res_7_d(&mut self) {}
    fn res_7_e(&mut self) {}
    fn res_7_h(&mut self) {}
    fn res_7_l(&mut self) {}
    fn res_7_hl(&mut self, memory: &Memory) {}
    fn res_7_a(&mut self) {}
    fn set_0_b(&mut self) {}
    fn set_0_c(&mut self) {}
    fn set_0_d(&mut self) {}
    fn set_0_e(&mut self) {}
    fn set_0_h(&mut self) {}
    fn set_0_l(&mut self) {}
    fn set_0_hl(&mut self, memory: &Memory) {}
    fn set_0_a(&mut self) {}
    fn set_1_b(&mut self) {}
    fn set_1_c(&mut self) {}
    fn set_1_d(&mut self) {}
    fn set_1_e(&mut self) {}
    fn set_1_h(&mut self) {}
    fn set_1_l(&mut self) {}
    fn set_1_hl(&mut self, memory: &Memory) {}
    fn set_1_a(&mut self) {}
    fn set_2_b(&mut self) {}
    fn set_2_c(&mut self) {}
    fn set_2_d(&mut self) {}
    fn set_2_e(&mut self) {}
    fn set_2_h(&mut self) {}
    fn set_2_l(&mut self) {}
    fn set_2_hl(&mut self, memory: &Memory) {}
    fn set_2_a(&mut self) {}
    fn set_3_b(&mut self) {}
    fn set_3_c(&mut self) {}
    fn set_3_d(&mut self) {}
    fn set_3_e(&mut self) {}
    fn set_3_h(&mut self) {}
    fn set_3_l(&mut self) {}
    fn set_3_hl(&mut self, memory: &Memory) {}
    fn set_3_a(&mut self) {}
    fn set_4_b(&mut self) {}
    fn set_4_c(&mut self) {}
    fn set_4_d(&mut self) {}
    fn set_4_e(&mut self) {}
    fn set_4_h(&mut self) {}
    fn set_4_l(&mut self) {}
    fn set_4_hl(&mut self, memory: &Memory) {}
    fn set_4_a(&mut self) {}
    fn set_5_b(&mut self) {}
    fn set_5_c(&mut self) {}
    fn set_5_d(&mut self) {}
    fn set_5_e(&mut self) {}
    fn set_5_h(&mut self) {}
    fn set_5_l(&mut self) {}
    fn set_5_hl(&mut self, memory: &Memory) {}
    fn set_5_a(&mut self) {}
    fn set_6_b(&mut self) {}
    fn set_6_c(&mut self) {}
    fn set_6_d(&mut self) {}
    fn set_6_e(&mut self) {}
    fn set_6_h(&mut self) {}
    fn set_6_l(&mut self) {}
    fn set_6_hl(&mut self, memory: &Memory) {}
    fn set_6_a(&mut self) {}
    fn set_7_b(&mut self) {}
    fn set_7_c(&mut self) {}
    fn set_7_d(&mut self) {}
    fn set_7_e(&mut self) {}
    fn set_7_h(&mut self) {}
    fn set_7_l(&mut self) {}
    fn set_7_hl(&mut self, memory: &Memory) {}
    fn set_7_a(&mut self) {}

    //interrupts

    pub fn rst_40(&mut self, memory: &mut Memory) {}

    pub fn rst_48(&mut self, memory: &mut Memory) {}

    pub fn rst_50(&mut self, memory: &mut Memory) {}

    pub fn rst_58(&mut self, memory: &mut Memory) {}

    pub fn rst_60(&mut self, memory: &mut Memory) {}

    //helpers

}