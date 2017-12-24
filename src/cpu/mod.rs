mod registers;

use cpu::registers::Registers;
use cpu::registers::flag::Flag;
use mmu::Memory;
use std::u8;

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

pub struct Cpu {
    registers: Registers,
    pub stopped: bool,
    interrupt_enabled: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Default::default(),
            stopped: false,
            interrupt_enabled: true,
        }
    }

    fn get_n(&mut self, memory: &Memory) -> u8 {
        let byte = memory.read_byte(self.registers.pc);
        self.registers.pc += 1;

        byte
    }

    fn get_nn(&mut self, memory: &Memory) -> u16 {
        let word = memory.read_word(self.registers.pc);
        self.registers.pc += 2;

        word
    }

    pub fn step(&mut self, memory: &mut Memory) -> u8 {
        let opcode = memory.read_byte(self.registers.pc);
        self.registers.pc += 1;

        match opcode {
            0x00 => self.nop(),
            0x01 => {
                let nn = self.get_nn(memory);
                self.ld_bc_nn(nn);
            }
            0x02 => self.ld_bc_a(memory),
            0x03 => self.inc_bc(),
            0x04 => self.inc_b(),
            0x05 => self.dec_b(),
            0x06 => {
                let n = self.get_n(memory);
                self.ld_b_n(n);
            }
            0x07 => self.rlc_a(),
            0x08 => {
                let nn = self.get_nn(memory);
                self.ld_nn_sp(nn, memory);
            }
            0x09 => self.add_hl_bc(),
            0x0A => self.ld_a_bc(memory),
            0x0B => self.dec_bc(),
            0x0C => self.inc_c(),
            0x0D => self.dec_c(),
            0x0E => {
                let n = self.get_n(memory);
                self.ld_c_n(n);
            },
            0x0F => self.rrc_a(),
            0x10 => self.stop(),
            0x11 => {
                let nn = self.get_nn(memory);
                self.ld_de_nn(nn);
            },
            0x12 => self.ld_de_a(memory),
            0x13 => self.inc_de(),
            0x14 => self.inc_d(),
            0x15 => self.dec_d(),
            0x16 => {
                let n = self.get_n(memory);
                self.ld_d_n(n);
            },
            0x17 => self.rl_a(),
            0x18 => {
                let n = self.get_n(memory);
                self.jr_n(n);
            },
            0x19 => self.add_hl_de(),
            0x1A => self.ld_a_de(memory),
            0x1B => self.dec_de(),
            0x1C => self.inc_e(),
            0x1D => self.dec_e(),
            0x1E => {
                let n = self.get_n(memory);
                self.ld_l_n(n);
            },
            0x1F => self.rr_a(),
            0x20 => {
                let n = self.get_n(memory);
                self.jr_nz_n(n);
            },
            0x21 => {
                let nn = self.get_nn(memory);
                self.ld_hl_nn(nn);
            },
            0x22 => self.ldi_hl_a(memory),
            0x23 => self.inc_hl(),
            0x24 => self.inc_h(),
            0x25 => self.dec_h(),
            0x26 => {
                let n = self.get_n(memory);
                self.ld_h_n(n);
            },
            0x27 => self.daa(),
            0x28 => {
                let n = self.get_n(memory);
                self.jr_z_n(n);
            },
            0x29 => self.add_hl_hl(),
            0x2A => self.ldi_a_hl(memory),
            0x2B => self.dec_hl(),
            0x2C => self.inc_l(),
            0x2D => self.dec_l(),
            0x2E => {
                let n = self.get_n(memory);
                self.ld_l_n(n);
            },
            0x2F => self.cpl(),
            0x30 => {
                let n = self.get_n(memory);
                self.jr_nc_n(n);
            },
            0x31 => {
                let nn = self.get_nn(memory);
                self.ld_hl_nn(nn);
            },
            0x32 => self.ldd_hl_a(memory),
            0x33 => self.inc_sp(),
            0x34 => self.inc_hl(),
            0x35 => self.dec_hl_ref(memory),
            0x36 => self.dec_hl_ref(memory),
            0x37 => {
                let n = self.get_n(memory);
                self.ld_hl_n(memory, n);
            },
            0x38 => {
                let n = self.get_n(memory);
                self.jr_c_n(n);
            },
            0x39 => self.add_hl_sp(),
            0x3A => self.ldd_a_hl(memory),
            0x3B => self.dec_sp(),
            0x3C => self.inc_a(),
            0x3D => self.dec_a(),
            0x3E => {
                let n = self.get_n(memory);
                self.ld_a_n(n);
            },
            0x3F => self.ccf(),
            0x40 => self.ld_b_b(),
            0x41 => self.ld_b_c(),
            0x42 => self.ld_b_d(),
            0x43 => self.ld_b_e(),
            0x44 => self.ld_b_h(),
            0x45 => self.ld_b_l(),
            0x46 => self.ld_b_hl(memory),
            0x47 => self.ld_b_a(),
            0x48 => self.ld_c_b(),
            0x49 => self.ld_c_c(),
            0x4A => self.ld_c_d(),
            0x4B => self.ld_c_e(),
            0x4C => self.ld_c_h(),
            0x4D => self.ld_c_l(),
            0x4E => self.ld_c_hl(memory),
            0x4F => self.ld_c_a(),
            0x50 => self.ld_d_b(),
            0x51 => self.ld_d_c(),
            0x52 => self.ld_d_d(),
            0x53 => self.ld_d_e(),
            0x54 => self.ld_d_h(),
            0x55 => self.ld_d_l(),
            0x56 => self.ld_d_hl(memory),
            0x57 => self.ld_d_a(),
            0x58 => self.ld_e_b(),
            0x59 => self.ld_e_c(),
            0x5A => self.ld_e_d(),
            0x5B => self.ld_e_e(),
            0x5C => self.ld_e_h(),
            0x5D => self.ld_e_l(),
            0x5E => self.ld_e_hl(memory),
            0x5F => self.ld_e_a(),
            0x60 => self.ld_h_b(),
            0x61 => self.ld_h_c(),
            0x62 => self.ld_h_d(),
            0x63 => self.ld_h_e(),
            0x64 => self.ld_h_h(),
            0x65 => self.ld_h_l(),
            0x66 => self.ld_h_hl(memory),
            0x67 => self.ld_h_a(),
            0x68 => self.ld_l_b(),
            0x69 => self.ld_l_c(),
            0x6A => self.ld_l_d(),
            0x6B => self.ld_l_e(),
            0x6C => self.ld_l_h(),
            0x6D => self.ld_l_l(),
            0x6E => self.ld_l_hl(memory),
            0x6F => self.ld_l_a(),
            0x70 => self.ld_hl_b(memory),
            0x71 => self.ld_hl_c(memory),
            0x72 => self.ld_hl_d(memory),
            0x73 => self.ld_hl_e(memory),
            0x74 => self.ld_hl_h(memory),
            0x75 => self.ld_hl_l(memory),
            0x76 => self.halt(),
            0x77 => self.ld_hl_a(memory),
            0x78 => self.ld_a_b(),
            0x79 => self.ld_a_c(),
            0x7A => self.ld_a_d(),
            0x7B => self.ld_a_e(),
            0x7C => self.ld_a_h(),
            0x7D => self.ld_a_l(),
            0x7E => self.ld_a_hl(memory),
            0x7F => self.ld_a_a(),
            0x80 => self.add_a_b(),
            0x81 => self.add_a_c(),
            0x82 => self.add_a_d(),
            0x83 => self.add_a_e(),
            0x84 => self.add_a_h(),
            0x85 => self.add_a_l(),
            0x86 => self.add_a_hl(memory),
            0x87 => self.add_a_a(),
            0x88 => self.adc_a_b(),
            0x89 => self.adc_a_c(),
            0x8A => self.adc_a_d(),
            0x8B => self.adc_a_e(),
            0x8C => self.adc_a_h(),
            0x8D => self.adc_a_l(),
            0x8E => self.adc_a_hl(memory),
            0x8F => self.adc_a_a(),
            0x90 => self.sub_a_b(),
            0x91 => self.sub_a_c(),
            0x92 => self.sub_a_d(),
            0x93 => self.sub_a_e(),
            0x94 => self.sub_a_h(),
            0x95 => self.sub_a_l(),
            0x96 => self.sub_a_hl(memory),
            0x97 => self.sub_a_a(),
            0x98 => self.sbc_a_b(),
            0x99 => self.sbc_a_c(),
            0x9A => self.sbc_a_d(),
            0x9B => self.sbc_a_e(),
            0x9C => self.sbc_a_h(),
            0x9D => self.sbc_a_l(),
            0x9E => self.sbc_a_hl(memory),
            0x9F => self.sbc_a_a(),
            0xA0 => self.and_b(),
            0xA1 => self.and_c(),
            0xA2 => self.and_d(),
            0xA3 => self.and_e(),
            0xA4 => self.and_h(),
            0xA5 => self.and_l(),
            0xA6 => self.and_hl(memory),
            0xA7 => self.and_a(),
            0xA8 => self.xor_b(),
            0xA9 => self.xor_c(),
            0xAA => self.xor_d(),
            0xAB => self.xor_e(),
            0xAC => self.xor_h(),
            0xAD => self.xor_l(),
            0xAE => self.xor_hl(memory),
            0xAF => self.xor_a(),
            0xB0 => self.or_b(),
            0xB1 => self.or_c(),
            0xB2 => self.or_d(),
            0xB3 => self.or_e(),
            0xB4 => self.or_h(),
            0xB5 => self.or_l(),
            0xB6 => self.or_hl(memory),
            0xB7 => self.or_a(),
            0xB8 => self.cp_b(),
            0xB9 => self.cp_c(),
            0xBA => self.cp_d(),
            0xBB => self.cp_e(),
            0xBC => self.cp_h(),
            0xBD => self.cp_l(),
            0xBE => self.cp_hl(memory),
            0xBF => self.cp_a(),
            0xC0 => self.ret_nz(memory),
            0xC1 => self.pop_bc(memory),
            0xC2 => {
                let nn = self.get_nn(memory);
                self.jp_nz_nn(nn);
            },
            0xC3 => {
                let nn = self.get_nn(memory);
                self.jp_nn(nn);
            },
            0xC4 => self.undefined(),
            0xC5 => {
                let nn = self.get_nn(memory);
                self.call_nz_nn(memory, nn);
            },
            0xC6 => self.push_bc(memory),
            0xC7 => {
                let n = self.get_n(memory);
                self.add_a_n(n);
            },
            0xC8 => self.rst_0(memory),
            0xC9 => self.ret_z(memory),
            0xCA => self.ret(memory),
            0xCB => {
                let n = self.get_n(memory);
                self.ext_ops(n, memory);
            },
            0xCC => {
                let nn = self.get_nn(memory);
                self.call_z_nn(nn, memory);
            },
            0xCD => {
                let nn = self.get_nn(memory);
                self.call_nn(nn, memory);
            },
            0xCE => {
                let n = self.get_n(memory);
                self.adc_a_n(n);
            },
            0xCF => self.rst_8(memory),
            0xD0 => self.ret_nc(memory),
            0xD1 => self.pop_de(memory),
            0xD2 => {
                let nn = self.get_nn(memory);
                self.jp_nc_nn(nn);
            },
            0xD3 => self.undefined(),
            0xD4 => {
                let nn = self.get_nn(memory);
                self.call_nc_nn(nn, memory);
            },
            0xD5 => self.push_de(memory),
            0xD6 => {
                let n = self.get_n(memory);
                self.sub_a_n(n);
            },
            0xD7 => self.rst_10(memory),
            0xD8 => self.ret_c(memory),
            0xD9 => self.ret_i(memory),
            0xDA => {
                let nn = self.get_nn(memory);
                self.jp_c_nn(nn);
            },
            0xDB => self.undefined(),
            0xDC => {
                let nn = self.get_nn(memory);
                self.call_c_nn(nn, memory);
            },
            0xDD => self.undefined(),
            0xDE => {
                let n = self.get_n(memory);
                self.sbc_a_n(n);
            },
            0xDF => self.rst_18(memory),
            0xE0 => {
                let n = self.get_n(memory);
                self.ldh_n_a(n, memory);
            },
            0xE1 => self.pop_hl(memory),
            0xE2 => self.ldh_c_a(),
            0xE3 => self.undefined(),
            0xE4 => self.undefined(),
            0xE5 => self.push_hl(memory),
            0xE6 => {
                let n = self.get_n(memory);
                self.and_n(n);
            },
            0xE7 => self.rst_20(memory),
            0xE8 => {
                let n = self.get_n(memory);
                self.add_sp_d(n);
            },
            0xE9 => self.jp_hl(memory),
            0xEA => {
                let nn = self.get_nn(memory);
                self.ld_nn_a(nn, memory);
            },
            0xEB => self.undefined(),
            0xEC => self.undefined(),
            0xED => self.undefined(),
            0xEE => {
                let n = self.get_n(memory);
                self.xor_n(n);
            },
            0xEF => self.rst_28(memory),
            0xF0 => {
                let n = self.get_n(memory);
                self.ldh_a_n(n, memory);
            },
            0xF1 => self.pop_af(memory),
            0xF2 => self.undefined(),
            0xF3 => self.di(),
            0xF4 => self.undefined(),
            0xF5 => self.push_af(memory),
            0xF6 => {
                let n = self.get_n(memory);
                self.or_n(n);
            },
            0xF7 => self.rst_30(memory),
            0xF8 => self.ldhl_sp_d(),
            0xF9 => self.ld_sp_hl(),
            0xFA => {
                let n = self.get_nn(memory);
                self.ld_a_nn(n, memory);
            },
            0xFB => self.ei(),
            0xFC => self.undefined(),
            0xFD => self.undefined(),
            0xFE => {
                let n = self.get_n(memory);
                self.cp_n(n);
            },
            0xFF => self.rst_38(memory),
            _ => {}
        }

        self.registers.cycles += CYCLES_TABLE[opcode as usize] as u16;

        CYCLES_TABLE[opcode as usize] as u8
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
        let b = self.registers.b;
        self.registers.b = self.inc_n(b);
    }

    fn dec_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.dec_n(b);
    }

    fn ld_b_n(&mut self, n: u8) {
        self.registers.b = n;
    }

    fn rlc_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rlc_n(a);
    }

    fn ld_nn_sp(&self, nn: u16, memory: &mut Memory) {
        memory.write_word(nn, self.registers.sp);
    }

    fn add_hl_bc(&mut self) {
        let bc = self.registers.get_bc();
        self.add_hl_ss(bc);
    }

    fn ld_a_bc(&mut self, memory: &Memory) {
        self.registers.a = memory[self.registers.get_bc()];
    }

    fn dec_bc(&mut self) {
        let mut bc = self.registers.get_bc();
        bc = self.dec_nn(bc);
        self.registers.set_bc(bc);
    }
    fn inc_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.inc_n(c);
    }
    fn dec_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.dec_n(c);
    }

    fn ld_c_n(&mut self, n: u8) {
        self.registers.c = n;
    }

    fn rrc_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rrc_n(a);
    }

    fn stop(&mut self) {
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
        let d = self.registers.d;
        self.registers.d = self.inc_n(d);
    }

    fn dec_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.dec_n(d);
    }

    fn ld_d_n(&mut self, n: u8) {
        self.registers.d = n;
    }

    fn rl_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rl_n(a);
        self.registers.f.remove(Flag::ZERO);
    }

    fn jr_n(&mut self, n: u8) {

    }

    fn add_hl_de(&mut self) {
        let de = self.registers.get_de();
        self.add_hl_ss(de);
    }

    fn ld_a_de(&mut self, memory: &Memory) {
        self.registers.a = memory[self.registers.get_de()];
    }

    fn dec_de(&mut self) {
        let mut de = self.registers.get_de();
        de = self.dec_nn(de);
        self.registers.set_de(de);
    }

    fn inc_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.inc_n(e);
    }

    fn dec_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.dec_n(e);
    }

    fn ld_e_n(&mut self, n: u8) {
        self.registers.e = n;
    }

    fn rr_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rr_n(a);
        self.registers.f.remove(Flag::ZERO);
    }

    fn jr_nz_n(&mut self, n: u8) {
        let cc = !self.registers.f.contains(Flag::ZERO);
        self.jr_cc_n(cc, n);
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
        let h = self.registers.h;
        self.registers.h = self.inc_n(h);
    }

    fn dec_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.dec_n(h);
    }

    fn ld_h_n(&mut self, n: u8) {
        self.registers.h = n;
    }

    fn daa(&mut self) {
        let low = self.registers.a & 0xF;
        let high = (self.registers.a & 0xF0) >> 4;

        if self.registers.f.contains(Flag::HALF_CARRY) || low > 9 {
            self.registers.a += 6;
            self.registers.f.remove(Flag::FULL_CARRY);
        }

        if self.registers.f.contains(Flag::HALF_CARRY) || high > 9 {
            self.registers.a += 60;
            self.registers.f.insert(Flag::FULL_CARRY);
        }

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.remove(Flag::HALF_CARRY);
    }

    fn jr_z_n(&mut self, n: u8) {
        let cc = self.registers.f.contains(Flag::ZERO);
        self.jr_cc_n(cc, n);
    }

    fn add_hl_hl(&mut self) {
        let hl = self.registers.get_hl();
        self.add_hl_ss(hl);
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
        let l = self.registers.l;
        self.registers.l = self.inc_n(l);
    }

    fn dec_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.dec_n(l);
    }

    fn ld_l_n(&mut self, n: u8) {
        self.registers.l = n;
    }

    fn cpl(&mut self) {
        self.registers.a = !self.registers.a;
        self.registers.f.insert(Flag::HALF_CARRY | Flag::NEGATIVE);
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

    fn inc_hl_ref(&mut self, memory: &mut Memory) {
        let mut hl = self.registers.get_hl();
        memory[hl] = self.inc_n(memory[hl]);
    }

    fn dec_hl_ref(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory[hl] = self.dec_n(memory[hl]);
    }

    fn ld_hl_n(&self, memory: &mut Memory, n: u8) {
        memory[self.registers.get_hl()] = n;
    }

    fn scf(&mut self) {
        self.registers.f.insert(Flag::FULL_CARRY);
    }

    fn jr_c_n(&mut self, n: u8) {
        self.registers.c = n;
    }

    fn add_hl_sp(&mut self) {
        let sp = self.registers.sp;
        self.add_hl_ss(sp);
    }

    fn ldd_a_hl(&mut self, memory: &Memory) {}

    fn dec_sp(&mut self) {
        let sp = self.registers.sp;
        self.registers.sp = self.dec_nn(sp);
    }

    fn inc_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.inc_n(a);
    }

    fn dec_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.dec_n(a);
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

    fn ld_d_b(&mut self) { self.registers.d = self.registers.b; }
    fn ld_d_c(&mut self) { self.registers.d = self.registers.c; }
    fn ld_d_d(&mut self) { self.registers.d = self.registers.d; }
    fn ld_d_e(&mut self) { self.registers.d = self.registers.e; }
    fn ld_d_h(&mut self) { self.registers.d = self.registers.h; }
    fn ld_d_l(&mut self) { self.registers.d = self.registers.l; }
    fn ld_d_hl(&mut self, memory: &Memory) { self.registers.d = memory[self.registers.get_hl()]; }
    fn ld_d_a(&mut self) { self.registers.d = self.registers.a; }

    fn ld_e_b(&mut self) { self.registers.e = self.registers.b; }
    fn ld_e_c(&mut self) { self.registers.e = self.registers.c; }
    fn ld_e_d(&mut self) { self.registers.e = self.registers.d; }
    fn ld_e_e(&mut self) { self.registers.e = self.registers.e; }
    fn ld_e_h(&mut self) { self.registers.e = self.registers.h; }
    fn ld_e_l(&mut self) { self.registers.e = self.registers.l; }
    fn ld_e_hl(&mut self, memory: &Memory) { self.registers.e = memory[self.registers.get_hl()]; }
    fn ld_e_a(&mut self) { self.registers.e = self.registers.a; }

    fn ld_h_b(&mut self) { self.registers.h = self.registers.b; }
    fn ld_h_c(&mut self) { self.registers.h = self.registers.c; }
    fn ld_h_d(&mut self) { self.registers.h = self.registers.d; }
    fn ld_h_e(&mut self) { self.registers.h = self.registers.e; }
    fn ld_h_h(&mut self) { self.registers.h = self.registers.h; }
    fn ld_h_l(&mut self) { self.registers.h = self.registers.l; }
    fn ld_h_hl(&mut self, memory: &Memory) { self.registers.h = memory[self.registers.get_hl()]; }
    fn ld_h_a(&mut self) { self.registers.h = self.registers.a; }

    fn ld_l_b(&mut self) { self.registers.l = self.registers.b; }
    fn ld_l_c(&mut self) { self.registers.l = self.registers.c; }
    fn ld_l_d(&mut self) { self.registers.l = self.registers.d; }
    fn ld_l_e(&mut self) { self.registers.l = self.registers.e; }
    fn ld_l_h(&mut self) { self.registers.l = self.registers.h; }
    fn ld_l_l(&mut self) { self.registers.l = self.registers.l; }
    fn ld_l_hl(&mut self, memory: &Memory) { self.registers.l = memory[self.registers.get_hl()]; }
    fn ld_l_a(&mut self) { self.registers.l = self.registers.a; }

    fn ld_hl_b(&self, memory: &mut Memory) { memory[self.registers.get_hl()] = self.registers.b; }
    fn ld_hl_c(&self, memory: &mut Memory) { memory[self.registers.get_hl()] = self.registers.c; }
    fn ld_hl_d(&self, memory: &mut Memory) { memory[self.registers.get_hl()] = self.registers.d; }
    fn ld_hl_e(&self, memory: &mut Memory) { memory[self.registers.get_hl()] = self.registers.e; }
    fn ld_hl_h(&self, memory: &mut Memory) { memory[self.registers.get_hl()] = self.registers.h; }
    fn ld_hl_l(&self, memory: &mut Memory) { memory[self.registers.get_hl()] = self.registers.l; }

    fn halt(&self) {}

    fn ld_hl_a(&mut self, memory: &mut Memory) { memory[self.registers.get_hl()] = self.registers.a; }

    fn ld_a_b(&mut self) { self.registers.a = self.registers.b; }
    fn ld_a_c(&mut self) { self.registers.a = self.registers.c; }
    fn ld_a_d(&mut self) { self.registers.a = self.registers.d; }
    fn ld_a_e(&mut self) { self.registers.a = self.registers.e; }
    fn ld_a_h(&mut self) { self.registers.a = self.registers.h; }
    fn ld_a_l(&mut self) { self.registers.a = self.registers.l; }
    fn ld_a_hl(&mut self, memory: &Memory) { self.registers.a = memory[self.registers.get_hl()]; }
    fn ld_a_a(&mut self) { self.registers.a = self.registers.a; }

    fn add_a_b(&mut self) { 
        let b = self.registers.b;
        self.add_a_n(b);
    }

    fn add_a_c(&mut self) {
        let c = self.registers.c;
        self.add_a_n(c); 
    }

    fn add_a_d(&mut self) { 
        let d = self.registers.d; 
        self.add_a_n(d); 
    }

    fn add_a_e(&mut self) { 
        let e = self.registers.e; 
        self.add_a_n(e); 
    }

    fn add_a_h(&mut self) { 
        let h = self.registers.h; 
        self.add_a_n(h);
    }

    fn add_a_l(&mut self) { 
        let l = self.registers.l; 
        self.add_a_n(l); 
    }

    fn add_a_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.add_a_n(memory[hl]);
    }

    fn add_a_a(&mut self) { 
        let a = self.registers.a;
        self.add_a_n(a); 
    }

    fn adc_a_b(&mut self) { 
        let b = self.registers.b;
        self.adc_a_n(b); 
    }
    
    fn adc_a_c(&mut self) { 
        let c = self.registers.c;
        self.adc_a_n(c); 
    }
    
    fn adc_a_d(&mut self) { 
        let d = self.registers.d;
        self.adc_a_n(d); 
    }
    
    fn adc_a_e(&mut self) { 
        let e = self.registers.e;
        self.adc_a_n(e); 
    }
    
    fn adc_a_h(&mut self) { 
        let h = self.registers.h;
        self.adc_a_n(h); 
    }
    
    fn adc_a_l(&mut self) { 
        let l = self.registers.l;
        self.adc_a_n(l); 
    }
    
    fn adc_a_hl(&mut self, memory: &Memory) { 
        let hl = self.registers.get_hl();
        self.adc_a_n(memory[hl]); 
    }
    
    fn adc_a_a(&mut self) { 
        let a = self.registers.a;
        self.adc_a_n(a); 
    }

    fn sub_a_b(&mut self) { 
        let b = self.registers.b;
        self.sub_a_n(b); 
    }
    
    fn sub_a_c(&mut self) { 
        let c = self.registers.c;
        self.sub_a_n(c); 
    }
    
    fn sub_a_d(&mut self) { 
        let d = self.registers.d;
        self.sub_a_n(d); 
    }
    
    fn sub_a_e(&mut self) { 
        let e = self.registers.e;
        self.sub_a_n(e); 
    }
    
    fn sub_a_h(&mut self) { 
        let h = self.registers.h;
        self.sub_a_n(h); 
    }
    
    fn sub_a_l(&mut self) { 
        let l = self.registers.l;
        self.sub_a_n(l); 
    }
    
    fn sub_a_hl(&mut self, memory: &Memory) { 
        let hl = self.registers.get_hl();
        self.sub_a_n(memory[hl]); 
    }
    fn sub_a_a(&mut self) { 
        let a = self.registers.a;
        self.sub_a_n(a); 
    }

    fn sbc_a_b(&mut self) { 
        let b = self.registers.b;
        self.sbc_a_n(b); 
    }

    fn sbc_a_c(&mut self) { 
        let c = self.registers.c;
        self.sbc_a_n(c); 
    }

    fn sbc_a_d(&mut self) { 
        let d = self.registers.d;
        self.sbc_a_n(d); 
    }

    fn sbc_a_e(&mut self) { 
        let e = self.registers.e;
        self.sbc_a_n(e); 
    }

    fn sbc_a_h(&mut self) { 
        let h = self.registers.h;
        self.sbc_a_n(h); 
    }

    fn sbc_a_l(&mut self) { 
        let l = self.registers.l;
        self.sbc_a_n(l); 
    }

    fn sbc_a_hl(&mut self, memory: &Memory) { 
        let hl = self.registers.get_hl();
        self.sbc_a_n(memory[hl]); 
    }

    fn sbc_a_a(&mut self) { 
        let a = self.registers.a;
        self.sbc_a_n(a); 
    }

    fn and_b(&mut self) { 
        let b = self.registers.b;
        self.and_n(b); 
    }

    fn and_c(&mut self) { 
        let c = self.registers.c;
        self.and_n(c); 
    }

    fn and_d(&mut self) { 
        let d = self.registers.d;
        self.and_n(d); 
    }

    fn and_e(&mut self) { 
        let e = self.registers.e;
        self.and_n(e); 
    }

    fn and_h(&mut self) { 
        let h = self.registers.h;
        self.and_n(h); 
    }

    fn and_l(&mut self) { 
        let l = self.registers.l;
        self.and_n(l); 
    }

    fn and_hl(&mut self, memory: &Memory) { 
        let hl = self.registers.get_hl();
        self.and_n(memory[hl]); 
    }

    fn and_a(&mut self) { 
        let a = self.registers.a;
        self.and_n(a); 
    }

    fn xor_b(&mut self) { 
        let b = self.registers.b;
        self.xor_n(b); 
    }

    fn xor_c(&mut self) { 
        let c = self.registers.c;
        self.xor_n(c); 
    }

    fn xor_d(&mut self) { 
        let d = self.registers.d;
        self.xor_n(d); 
    }

    fn xor_e(&mut self) { 
        let e = self.registers.e;
        self.xor_n(e); 
    }

    fn xor_h(&mut self) { 
        let h = self.registers.h;
        self.xor_n(h); 
    }

    fn xor_l(&mut self) {
        let l = self.registers.l;
        self.xor_n(l); 
    }

    fn xor_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.xor_n(memory[hl]); 
    }

    fn xor_a(&mut self) { 
        let a = self.registers.a;
        self.xor_n(a); 
    }

    fn or_b(&mut self) { 
        let b = self.registers.b;
        self.or_n(b); 
    }

    fn or_c(&mut self) { 
        let c = self.registers.c;
        self.or_n(c); 
    }

    fn or_d(&mut self) { 
        let d = self.registers.d;
        self.or_n(d); 
    }

    fn or_e(&mut self) { 
        let e = self.registers.e;
        self.or_n(e); 
    }

    fn or_h(&mut self) { 
        let h = self.registers.h;
        self.or_n(h); 
    }

    fn or_l(&mut self) { 
        let l = self.registers.l;
        self.or_n(l); 
    }

    fn or_hl(&mut self, memory: &Memory) { 
        let hl = self.registers.get_hl();
        self.xor_n(memory[hl]); 
    }

    fn or_a(&mut self) { 
        let a = self.registers.a;
        self.xor_n(a); 
    }

    fn cp_b(&mut self) { 
        let b = self.registers.b;
        self.cp_n(b); 
    }

    fn cp_c(&mut self) { 
        let c = self.registers.c;
        self.cp_n(c); 
    }

    fn cp_d(&mut self) {
        let d = self.registers.d;
        self.cp_n(d);
    }

    fn cp_e(&mut self) { 
        let e = self.registers.e;
        self.cp_n(e); 
    }

    fn cp_h(&mut self) { 
        let h = self.registers.h;
        self.cp_n(h); 
    }

    fn cp_l(&mut self) { 
        let l = self.registers.l;
        self.cp_n(l); 
    }

    fn cp_hl(&mut self, memory: &Memory) { 
        let hl = self.registers.get_hl();
        self.cp_n(memory[hl]); 
    }

    fn cp_a(&mut self) { 
        let a = self.registers.a;
        self.cp_n(a); 
    }

    fn ret_nz(&mut self, memory: &Memory) { 
        let cc = !self.registers.f.contains(Flag::ZERO);
        self.ret_cc(cc, memory); 
    }

    fn pop_bc(&mut self, memory: &Memory) { 
        let bc = self.pop(memory);
        self.registers.set_bc(bc); 
    }

    fn jp_nz_nn(&mut self, nn: u16) { 
        let cc = !self.registers.f.contains(Flag::ZERO);
        self.jp_cc_nn(cc, nn); 
    }

    fn jp_nn(&mut self, nn: u16) {
        self.registers.pc = nn;
    }

    fn call_nz_nn(&mut self, memory: &mut Memory, nn: u16) { 
        let cc = !self.registers.f.contains(Flag::ZERO);
        self.call_cc_nn(cc, nn, memory);
    }

    fn push_bc(&mut self, memory: &mut Memory) { 
        let bc = self.registers.get_bc();
        self.push_nn(bc, memory); 
    }

    fn add_a_n(&mut self, n: u8) {

        let half_carry = (((self.registers.a) & 0xF) + (n & 0xF)) & 0x10 == 0x10;
        let full_carry = ((self.registers.a as u16) + (n as u16)) & 0x100 == 0x100;

        self.registers.a = self.registers.a.wrapping_add(n);

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.remove(Flag::NEGATIVE);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
    }

    fn rst_0(&mut self, memory: &mut Memory) { self.rst_n(0x0, memory); }

    fn ret_z(&mut self, memory: &Memory) { 
        let cc = self.registers.f.contains(Flag::ZERO);
        self.ret_cc(cc, memory); 
    }

    fn ret(&mut self, memory: &Memory) {
        self.registers.pc = self.pop(memory);
    }

    fn jp_z_nn(&mut self, nn: u16) { 
        let cc = self.registers.f.contains(Flag::ZERO);
        self.jp_cc_nn(cc, nn); 
    }

    fn ext_ops(&mut self, opcode: u8, memory: &mut Memory) {
        match opcode {
            0x00 => self.rlc_b(),
            0x01 => self.rlc_c(),
            0x02 => self.rlc_d(),
            0x03 => self.rlc_e(),
            0x04 => self.rlc_h(),
            0x05 => self.rlc_l(),
            0x06 => self.rlc_hl(memory),
            0x07 => self.rlc_a(),
            0x08 => self.rrc_b(),
            0x09 => self.rrc_c(),
            0x0A => self.rrc_d(),
            0x0B => self.rrc_e(),
            0x0C => self.rrc_h(),
            0x0D => self.rrc_l(),
            0x0E => self.rrc_hl(memory),
            0x0F => self.rrc_a(),
            0x10 => self.rl_b(),
            0x11 => self.rl_c(),
            0x12 => self.rl_d(),
            0x13 => self.rl_e(),
            0x14 => self.rl_h(),
            0x15 => self.rl_l(),
            0x16 => self.rl_hl(memory),
            0x17 => self.rl_a(),
            0x18 => self.rr_b(),
            0x19 => self.rr_c(),
            0x1A => self.rr_d(),
            0x1B => self.rr_e(),
            0x1C => self.rr_h(),
            0x1D => self.rr_l(),
            0x1E => self.rr_hl(memory),
            0x1F => self.rr_a(),
            0x20 => self.sla_b(),
            0x21 => self.sla_c(),
            0x22 => self.sla_d(),
            0x23 => self.sla_e(),
            0x24 => self.sla_h(),
            0x25 => self.sla_l(),
            0x26 => self.sla_hl(memory),
            0x27 => self.sla_a(),
            0x28 => self.sra_b(),
            0x29 => self.sra_c(),
            0x2A => self.sra_d(),
            0x2B => self.sra_e(),
            0x2C => self.sra_h(),
            0x2D => self.sra_l(),
            0x2E => self.sra_hl(memory),
            0x2F => self.sra_a(),
            0x30 => self.swap_b(),
            0x31 => self.swap_c(),
            0x32 => self.swap_d(),
            0x33 => self.swap_e(),
            0x34 => self.swap_h(),
            0x35 => self.swap_l(),
            0x36 => self.swap_hl(memory),
            0x37 => self.swap_a(),
            0x38 => self.srl_b(),
            0x39 => self.srl_c(),
            0x3A => self.srl_d(),
            0x3B => self.srl_e(),
            0x3C => self.srl_h(),
            0x3D => self.srl_l(),
            0x3E => self.srl_hl(memory),
            0x3F => self.srl_a(),
            0x40 => self.bit_0_b(),
            0x41 => self.bit_0_c(),
            0x42 => self.bit_0_d(),
            0x43 => self.bit_0_e(),
            0x44 => self.bit_0_h(),
            0x45 => self.bit_0_l(),
            0x46 => self.bit_0_hl(memory),
            0x47 => self.bit_0_a(),
            0x48 => self.bit_1_b(),
            0x49 => self.bit_1_c(),
            0x4A => self.bit_1_d(),
            0x4B => self.bit_1_e(),
            0x4C => self.bit_1_h(),
            0x4D => self.bit_1_l(),
            0x4E => self.bit_1_hl(memory),
            0x4F => self.bit_1_a(),
            0x50 => self.bit_2_b(),
            0x51 => self.bit_2_c(),
            0x52 => self.bit_2_d(),
            0x53 => self.bit_2_e(),
            0x54 => self.bit_2_h(),
            0x55 => self.bit_2_l(),
            0x56 => self.bit_2_hl(memory),
            0x57 => self.bit_2_a(),
            0x58 => self.bit_3_b(),
            0x59 => self.bit_3_c(),
            0x5A => self.bit_3_d(),
            0x5B => self.bit_3_e(),
            0x5C => self.bit_3_h(),
            0x5D => self.bit_3_l(),
            0x5E => self.bit_3_hl(memory),
            0x5F => self.bit_3_a(),
            0x60 => self.bit_4_b(),
            0x61 => self.bit_4_c(),
            0x62 => self.bit_4_d(),
            0x63 => self.bit_4_e(),
            0x64 => self.bit_4_h(),
            0x65 => self.bit_4_l(),
            0x66 => self.bit_4_hl(memory),
            0x67 => self.bit_4_a(),
            0x68 => self.bit_5_b(),
            0x69 => self.bit_5_c(),
            0x6A => self.bit_5_d(),
            0x6B => self.bit_5_e(),
            0x6C => self.bit_5_h(),
            0x6D => self.bit_5_l(),
            0x6E => self.bit_5_hl(memory),
            0x6F => self.bit_5_a(),
            0x70 => self.bit_6_b(),
            0x71 => self.bit_6_c(),
            0x72 => self.bit_6_d(),
            0x73 => self.bit_6_e(),
            0x74 => self.bit_6_h(),
            0x75 => self.bit_6_l(),
            0x76 => self.bit_6_hl(memory),
            0x77 => self.bit_6_a(),
            0x78 => self.bit_7_b(),
            0x79 => self.bit_7_c(),
            0x7A => self.bit_7_d(),
            0x7B => self.bit_7_e(),
            0x7C => self.bit_7_h(),
            0x7D => self.bit_7_l(),
            0x7E => self.bit_7_hl(memory),
            0x7F => self.bit_7_a(),
            0x80 => self.res_0_b(),
            0x81 => self.res_0_c(),
            0x82 => self.res_0_d(),
            0x83 => self.res_0_e(),
            0x84 => self.res_0_h(),
            0x85 => self.res_0_l(),
            0x86 => self.res_0_hl(memory),
            0x87 => self.res_0_a(),
            0x88 => self.res_1_b(),
            0x89 => self.res_1_c(),
            0x8A => self.res_1_d(),
            0x8B => self.res_1_e(),
            0x8C => self.res_1_h(),
            0x8D => self.res_1_l(),
            0x8E => self.res_1_hl(memory),
            0x8F => self.res_1_a(),
            0x90 => self.res_2_b(),
            0x91 => self.res_2_c(),
            0x92 => self.res_2_d(),
            0x93 => self.res_2_e(),
            0x94 => self.res_2_h(),
            0x95 => self.res_2_l(),
            0x96 => self.res_2_hl(memory),
            0x97 => self.res_2_a(),
            0x98 => self.res_3_b(),
            0x99 => self.res_3_c(),
            0x9A => self.res_3_d(),
            0x9B => self.res_3_e(),
            0x9C => self.res_3_h(),
            0x9D => self.res_3_l(),
            0x9E => self.res_3_hl(memory),
            0x9F => self.res_3_a(),
            0xA0 => self.res_4_b(),
            0xA1 => self.res_4_c(),
            0xA2 => self.res_4_d(),
            0xA3 => self.res_4_e(),
            0xA4 => self.res_4_h(),
            0xA5 => self.res_4_l(),
            0xA6 => self.res_4_hl(memory),
            0xA7 => self.res_4_a(),
            0xA8 => self.res_5_b(),
            0xA9 => self.res_5_c(),
            0xAA => self.res_5_d(),
            0xAB => self.res_5_e(),
            0xAC => self.res_5_h(),
            0xAD => self.res_5_l(),
            0xAE => self.res_5_hl(memory),
            0xAF => self.res_5_a(),
            0xB0 => self.res_6_b(),
            0xB1 => self.res_6_c(),
            0xB2 => self.res_6_d(),
            0xB3 => self.res_6_e(),
            0xB4 => self.res_6_h(),
            0xB5 => self.res_6_l(),
            0xB6 => self.res_6_hl(memory),
            0xB7 => self.res_6_a(),
            0xB8 => self.res_7_b(),
            0xB9 => self.res_7_c(),
            0xBA => self.res_7_d(),
            0xBB => self.res_7_e(),
            0xBC => self.res_7_h(),
            0xBD => self.res_7_l(),
            0xBE => self.res_7_hl(memory),
            0xBF => self.res_7_a(),
            0xC0 => self.set_0_b(),
            0xC1 => self.set_0_c(),
            0xC2 => self.set_0_d(),
            0xC3 => self.set_0_e(),
            0xC4 => self.set_0_h(),
            0xC5 => self.set_0_l(),
            0xC6 => self.set_0_hl(memory),
            0xC7 => self.set_0_a(),
            0xC8 => self.set_1_b(),
            0xC9 => self.set_1_c(),
            0xCA => self.set_1_d(),
            0xCB => self.set_1_e(),
            0xCC => self.set_1_h(),
            0xCD => self.set_1_l(),
            0xCE => self.set_1_hl(memory),
            0xCF => self.set_1_a(),
            0xD0 => self.set_2_b(),
            0xD1 => self.set_2_c(),
            0xD2 => self.set_2_d(),
            0xD3 => self.set_2_e(),
            0xD4 => self.set_2_h(),
            0xD5 => self.set_2_l(),
            0xD6 => self.set_2_hl(memory),
            0xD7 => self.set_2_a(),
            0xD8 => self.set_3_b(),
            0xD9 => self.set_3_c(),
            0xDA => self.set_3_d(),
            0xDB => self.set_3_e(),
            0xDC => self.set_3_h(),
            0xDD => self.set_3_l(),
            0xDE => self.set_3_hl(memory),
            0xDF => self.set_3_a(),
            0xE0 => self.set_4_b(),
            0xE1 => self.set_4_c(),
            0xE2 => self.set_4_d(),
            0xE3 => self.set_4_e(),
            0xE4 => self.set_4_h(),
            0xE5 => self.set_4_l(),
            0xE6 => self.set_4_hl(memory),
            0xE7 => self.set_4_a(),
            0xE8 => self.set_5_b(),
            0xE9 => self.set_5_c(),
            0xEA => self.set_5_d(),
            0xEB => self.set_5_e(),
            0xEC => self.set_5_h(),
            0xED => self.set_5_l(),
            0xEE => self.set_5_hl(memory),
            0xEF => self.set_5_a(),
            0xF0 => self.set_6_b(),
            0xF1 => self.set_6_c(),
            0xF2 => self.set_6_d(),
            0xF3 => self.set_6_e(),
            0xF4 => self.set_6_h(),
            0xF5 => self.set_6_l(),
            0xF6 => self.set_6_hl(memory),
            0xF7 => self.set_6_a(),
            0xF8 => self.set_7_b(),
            0xF9 => self.set_7_c(),
            0xFA => self.set_7_d(),
            0xFB => self.set_7_e(),
            0xFC => self.set_7_h(),
            0xFD => self.set_7_l(),
            0xFE => self.set_7_hl(memory),
            0xFF => self.set_7_a(),
            _ => {}
        }
    }

    fn call_z_nn(&mut self, nn: u16, memory: &mut Memory) { 
        let cc = self.registers.f.contains(Flag::ZERO);
        self.call_cc_nn(cc, nn, memory); 
    }

    fn call_nn(&mut self, nn: u16, memory: &mut Memory) { 
        let pc = self.registers.pc;
        self.push_nn(pc, memory);
        self.registers.pc = nn;
    }

    fn adc_a_n(&mut self, n: u8) {

        let carry = if self.registers.f.contains(Flag::FULL_CARRY) {
            1
        } else {
            0
        };

        let half_carry = ((self.registers.a & 0xF) + (n & 0xF) + (carry & 0xF)) & 0x10 == 0x10;
        let full_carry = ((self.registers.a as u16) + (n as u16) + (carry as u16)) & 0x100 == 0x100;

        self.registers.a = self.registers.a.wrapping_add(n);
        self.registers.a = self.registers.a.wrapping_add(carry);

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.remove(Flag::NEGATIVE);

    }

    fn rst_8(&mut self, memory: &mut Memory) { self.rst_n(0x8, memory); }

    fn ret_nc(&mut self, memory: &Memory) { 
        let cc = !self.registers.f.contains(Flag::FULL_CARRY);
        self.ret_cc(cc, memory); 
    }

    fn pop_de(&mut self, memory: &Memory) { 
        let hl = self.pop(memory);
        self.registers.set_hl(hl); 
    }

    fn jp_nc_nn(&mut self, nn: u16) { 
        let cc = !self.registers.f.contains(Flag::FULL_CARRY);
        self.jp_cc_nn(cc, nn); 
    }

    fn undefined(&mut self) { 
        panic!("Undefined Opcode!"); 
    }

    fn call_nc_nn(&mut self, nn: u16, memory: &mut Memory) { 
        let cc = !self.registers.f.contains(Flag::FULL_CARRY);
        self.call_cc_nn(cc, nn, memory); 
    }

    fn push_de(&mut self, memory: &mut Memory) { 
        let de = self.registers.get_de();
        self.push_nn(de, memory);
    }

    fn sub_a_n(&mut self, n: u8) {
        let n = !n + 1;

        let half_carry = (((self.registers.a & 0xF) + (n & 0xF)) & 0x10) == 0x10;
        let full_carry = ((self.registers.a as u16) + (n as u16)) & 0x100 == 0x100;

        self.registers.a = self.registers.a.wrapping_add(n);

        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.insert(Flag::NEGATIVE);
    }

    fn rst_10(&mut self, memory: &mut Memory) { self.rst_n(0x10, memory); }

    fn ret_c(&mut self, memory: &Memory) { 
        let cc = self.registers.f.contains(Flag::FULL_CARRY);
        self.ret_cc(cc, memory); 
    }

    fn ret_i(&mut self, memory: &Memory) {
        self.ret(memory);
        self.registers.ime = true;
    }

    fn jp_c_nn(&mut self, nn: u16) { 
        let cc = self.registers.f.contains(Flag::FULL_CARRY);
        self.jp_cc_nn(cc, nn);
    }

    fn call_c_nn(&mut self, nn: u16, memory: &mut Memory) { 
        let cc = self.registers.f.contains(Flag::FULL_CARRY);
        self.call_cc_nn(cc, nn, memory); 
    }

    fn sbc_a_n(&mut self, n: u8) {
        let carry = if self.registers.f.contains(Flag::FULL_CARRY) {
            !1 + 1
        } else {
            0
        };

        let n = !n + 1;

        let half_carry = ((self.registers.a & 0xF) + (n & 0xF) + (carry & 0xF)) & 0x10 == 0x10;
        let full_carry = ((self.registers.a as u16) + (n as u16) + (carry as u16)) & 0x100 == 0x100;

        self.registers.a = self.registers.a.wrapping_add(n);
        self.registers.a = self.registers.a.wrapping_add(carry);

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.insert(Flag::NEGATIVE);

    }

    fn rst_18(&mut self, memory: &mut Memory) { self.rst_n(0x18, memory); }

    fn ldh_n_a(&self, n: u8, memory: &mut Memory) {}

    fn pop_hl(&mut self, memory: &Memory) { 
        let hl = self.pop(memory);
        self.registers.set_hl(hl); 
    }

    fn ldh_c_a(&mut self) {}

    fn push_hl(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        self.push_nn(hl, memory);
    }

    fn and_n(&mut self, n: u8) {
        self.registers.a &= n;

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.insert(Flag::HALF_CARRY);
        self.registers.f.remove(Flag::NEGATIVE | Flag::FULL_CARRY);
    }

    fn rst_20(&mut self, memory: &mut Memory) { self.rst_n(0x20, memory); }

    fn add_sp_d(&mut self, d: u8) {}

    fn jp_hl(&mut self, memory: &Memory) {}

    fn ld_nn_a(&self, nn: u16, memory: &mut Memory) { memory[nn] = self.registers.a; }

    fn xor_n(&mut self, n: u8) {
        self.registers.a ^= n;

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY);
    }

    fn rst_28(&mut self, memory: &mut Memory) { self.rst_n(0x28, memory); }

    fn ldh_a_n(&mut self, n: u8, memory: &Memory) {}

    fn pop_af(&mut self, memory: &Memory) { 
        let af = self.pop(memory);
        self.registers.set_af(af); 
    }

    fn di(&mut self) { self.registers.ime = false; }

    fn push_af(&mut self, memory: &mut Memory) { 
        let af = self.registers.get_af();
        self.push_nn(af, memory); 
    }

    fn or_n(&mut self, n: u8) {
        self.registers.a |= n;

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::FULL_CARRY | Flag::HALF_CARRY);
    }

    fn rst_30(&mut self, memory: &mut Memory) { self.rst_n(0x30, memory); }

    fn ldhl_sp_d(&mut self) {}

    fn ld_sp_hl(&mut self) {}

    fn ld_a_nn(&mut self, nn: u16, memory: &Memory) { self.registers.a = memory[nn]; }

    fn ei(&mut self) { self.registers.ime = true; }

    fn cp_n(&mut self, n: u8) {
        let n = !n + 1;

        let half_carry = (((self.registers.a & 0xF) + (n & 0xF)) & 0x10) == 0x10;
        let overflow = ((self.registers.a as u16) + (n as u16)) & 0x100 == 0x100;

        let result = self.registers.a.wrapping_add(n);

        self.registers.f.set(Flag::FULL_CARRY, overflow);
        self.registers.f.set(Flag::ZERO, result == 0);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.insert(Flag::NEGATIVE);
    }

    fn rst_38(&mut self, memory: &mut Memory) { self.rst_n(0x38, memory); }

    //extended opcodes

    fn rlc_b(&mut self) { 
        let b = self.registers.b;
        self.registers.b = self.rlc_n(b); 
    }
    
    fn rlc_c(&mut self) { 
        let c = self.registers.c;
        self.registers.c = self.rlc_n(c); 
    }
    
    fn rlc_d(&mut self) { 
        let d = self.registers.d;
        self.registers.d = self.rlc_n(d); 
    }
    
    fn rlc_e(&mut self) { 
        let e = self.registers.e;
        self.registers.e = self.rlc_n(e); 
    }
    
    fn rlc_h(&mut self) { 
        let h = self.registers.h;
        self.registers.h = self.rlc_n(h); 
    }
    
    fn rlc_l(&mut self) { 
        let l = self.registers.l;
        self.registers.l = self.rlc_n(l); 
    }
    
    fn rlc_hl(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory[hl] = self.rlc_n(memory[hl]);
    }
    
    fn rrc_b(&mut self) { 
        let b = self.registers.b;
        self.registers.b = self.rrc_n(b); 
    }
    
    fn rrc_c(&mut self) { 
        let c = self.registers.c;
        self.registers.c = self.rrc_n(c); 
    }
    
    fn rrc_d(&mut self) { 
        let d = self.registers.d;
        self.registers.d = self.rrc_n(d); 
    }
    
    fn rrc_e(&mut self) { 
        let e = self.registers.e;
        self.registers.e = self.rrc_n(e); 
    }
    
    fn rrc_h(&mut self) { 
        let h = self.registers.h;
        self.registers.h = self.rrc_n(h); }

        fn rrc_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.rrc_n(l); 
        }

        fn rrc_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.rrc_n(memory[hl]);
        }

        fn rl_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.rl_n(b); 
        }

        fn rl_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.rl_n(c); 
        }

        fn rl_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.rl_n(d); 
        }

        fn rl_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.rl_n(e); 
        }

        fn rl_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.rl_n(h); 
        }

        fn rl_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.rl_n(l); 
        }

        fn rl_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.rl_n(memory[hl]);
        }

        fn rr_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.rr_n(b); 
        }

        fn rr_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.rr_n(c); 
        }

        fn rr_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.rr_n(d); 
        }

        fn rr_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.rr_n(e); 
        }

        fn rr_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.rr_n(h); 
        }

        fn rr_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.rr_n(l); 
        }

        fn rr_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.rr_n(memory[hl]);
        }

        fn sla_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.sla_n(b); 
        }

        fn sla_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.sla_n(c); 
        }

        fn sla_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.sla_n(d); 
        }

        fn sla_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.sla_n(e); 
        }

        fn sla_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.sla_n(h); 
        }

        fn sla_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.sla_n(l); 
        }

        fn sla_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.sla_n(memory[hl]);
        }

        fn sla_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.sla_n(a); 
        }

        fn sra_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.sla_n(b); 
        }

        fn sra_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.sra_n(c); 
        }

        fn sra_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.sra_n(d); 
        }

        fn sra_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.sra_n(e); 
        }

        fn sra_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.sra_n(h); 
        }

        fn sra_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.sra_n(l); 
        }

        fn sra_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.sra_n(memory[hl]);
        }

        fn sra_a(&mut self) {
            let a = self.registers.a;
            self.registers.a = self.sra_n(a);
        }

        fn swap_b(&mut self) {
            let b = self.registers.b;
            self.registers.b = self.swap_n(b);
        }

        fn swap_c(&mut self) {
            let c = self.registers.c;
            self.registers.c = self.swap_n(c);
        }

        fn swap_d(&mut self) {
            let d = self.registers.d;
            self.registers.d = self.swap_n(d);
        }

        fn swap_e(&mut self) {
            let e = self.registers.e;
            self.registers.e = self.swap_n(e);
        }

        fn swap_h(&mut self) {
            let h = self.registers.h;
            self.registers.h = self.swap_n(h);
        }

        fn swap_l(&mut self) {
            let l = self.registers.l;
            self.registers.l = self.swap_n(l);
        }

        fn swap_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.swap_n(memory[hl]);
        }

        fn swap_a(&mut self) {
            let a = self.registers.a;
            self.registers.a = self.swap_n(a);
        }

        fn srl_b(&mut self) {
            let b = self.registers.b;
            self.registers.b = self.srl_n(b);
        }

        fn srl_c(&mut self) {
            let c = self.registers.c;
            self.registers.c = self.srl_n(c);
        }

        fn srl_d(&mut self) {
            let d = self.registers.d;
            self.registers.d = self.srl_n(d);
        }

        fn srl_e(&mut self) {
            let e = self.registers.e;
            self.registers.e = self.srl_n(e);
        }

        fn srl_h(&mut self) {
            let h = self.registers.h;
            self.registers.h = self.srl_n(h);
        }

        fn srl_l(&mut self) {
            let l = self.registers.l;
            self.registers.l = self.srl_n(l);
        }

        fn srl_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.srl_n(memory[hl]);
        }

        fn srl_a(&mut self) {
            let a = self.registers.a;
            self.registers.a = self.srl_n(a);
        }

        fn bit_0_b(&mut self) { 
            let b = self.registers.b;
            self.bit_i_n(0, b); 
        }

        fn bit_0_c(&mut self) { 
            let c = self.registers.c;
            self.bit_i_n(0, c); 
        }

        fn bit_0_d(&mut self) { 
            let d = self.registers.d;
            self.bit_i_n(0, d); 
        }

        fn bit_0_e(&mut self) { 
            let e = self.registers.e;
            self.bit_i_n(0, e); 
        }

        fn bit_0_h(&mut self) { 
            let h = self.registers.h;
            self.bit_i_n(0, h); 
        }

        fn bit_0_l(&mut self) { 
            let l = self.registers.l;
            self.bit_i_n(0, l); 
        }

        fn bit_0_hl(&mut self, memory: &Memory) { 
            let hl = self.registers.get_hl();
            self.bit_i_n(0, memory[hl]); 
        }

        fn bit_0_a(&mut self) { 
            let a = self.registers.a;
            self.bit_i_n(0, a); 
        }

        fn bit_1_b(&mut self) { 
            let b = self.registers.b;
            self.bit_i_n(1, b); 
        }

        fn bit_1_c(&mut self) { 
            let c = self.registers.c;
            self.bit_i_n(1, c); 
        }

        fn bit_1_d(&mut self) { 
            let d = self.registers.d;
            self.bit_i_n(1, d); 
        }

        fn bit_1_e(&mut self) { 
            let e = self.registers.e;
            self.bit_i_n(1, e); 
        }

        fn bit_1_h(&mut self) { 
            let h = self.registers.h;
            self.bit_i_n(1, h); 
        }

        fn bit_1_l(&mut self) { 
            let l = self.registers.l;
            self.bit_i_n(1, l); 
        }

        fn bit_1_hl(&mut self, memory: &Memory) { 
            let hl = self.registers.get_hl();
            self.bit_i_n(1, memory[hl]); 
        }

        fn bit_1_a(&mut self) { 
            let a = self.registers.a;
            self.bit_i_n(1, a); 
        }

        fn bit_2_b(&mut self) { 
            let b = self.registers.b;
            self.bit_i_n(2, b); 
        }

        fn bit_2_c(&mut self) { 
            let c = self.registers.c;
            self.bit_i_n(2, c); 
        }

        fn bit_2_d(&mut self) { 
            let d = self.registers.d;
            self.bit_i_n(2, d); 
        }

        fn bit_2_e(&mut self) { 
            let e = self.registers.e;
            self.bit_i_n(2, e); 
        }

        fn bit_2_h(&mut self) { 
            let h = self.registers.h;
            self.bit_i_n(2, h); 
        }

        fn bit_2_l(&mut self) { 
            let l = self.registers.l;
            self.bit_i_n(2, l); 
        }

        fn bit_2_hl(&mut self, memory: &Memory) { 
            let hl = self.registers.get_hl();
            self.bit_i_n(2, memory[hl]); 
        }

        fn bit_2_a(&mut self) { 
            let a = self.registers.a;
            self.bit_i_n(2, a); 
        }

        fn bit_3_b(&mut self) { 
            let b = self.registers.b;
            self.bit_i_n(3, b); 
        }

        fn bit_3_c(&mut self) { 
            let c = self.registers.c;
            self.bit_i_n(3, c); 
        }

        fn bit_3_d(&mut self) { 
            let d = self.registers.d;
            self.bit_i_n(3, d); 
        }

        fn bit_3_e(&mut self) { 
            let e = self.registers.e;
            self.bit_i_n(3, e); 
        }

        fn bit_3_h(&mut self) { 
            let h = self.registers.h;
            self.bit_i_n(3, h); 
        }

        fn bit_3_l(&mut self) { 
            let l = self.registers.l;
            self.bit_i_n(3, l); 
        }

        fn bit_3_hl(&mut self, memory: &Memory) { 
            let hl = self.registers.get_hl();
            self.bit_i_n(3, memory[hl]); 
        }

        fn bit_3_a(&mut self) { 
            let a = self.registers.a;
            self.bit_i_n(3, a); 
        }

        fn bit_4_b(&mut self) { 
            let b = self.registers.b;
            self.bit_i_n(4, b); 
        }

        fn bit_4_c(&mut self) { 
            let c = self.registers.c;
            self.bit_i_n(4, c); 
        }

        fn bit_4_d(&mut self) { 
            let d = self.registers.d;
            self.bit_i_n(4, d); 
        }

        fn bit_4_e(&mut self) { 
            let e = self.registers.e;
            self.bit_i_n(4, e); 
        }

        fn bit_4_h(&mut self) { 
            let h = self.registers.h;
            self.bit_i_n(4, h); 
        }

        fn bit_4_l(&mut self) { 
            let l = self.registers.l;
            self.bit_i_n(4, l); 
        }

        fn bit_4_hl(&mut self, memory: &Memory) { 
            let hl = self.registers.get_hl();
            self.bit_i_n(4, memory[hl]); 
        }

        fn bit_4_a(&mut self) { 
            let a = self.registers.a;
            self.bit_i_n(4, a); 
        }

        fn bit_5_b(&mut self) { 
            let b = self.registers.b;
            self.bit_i_n(5, b); 
        }

        fn bit_5_c(&mut self) { 
            let c = self.registers.c;
            self.bit_i_n(5, c); 
        }

        fn bit_5_d(&mut self) { 
            let d = self.registers.d;
            self.bit_i_n(5, d); 
        }

        fn bit_5_e(&mut self) { 
            let e = self.registers.e;
            self.bit_i_n(5, e); 
        }

        fn bit_5_h(&mut self) { 
            let h = self.registers.h;
            self.bit_i_n(5, h); 
        }

        fn bit_5_l(&mut self) { 
            let l = self.registers.l;
            self.bit_i_n(5, l); 
        }

        fn bit_5_hl(&mut self, memory: &Memory) { 
            let hl = self.registers.get_hl();
            self.bit_i_n(5, memory[hl]); 
        }

        fn bit_5_a(&mut self) { 
            let a = self.registers.a;
            self.bit_i_n(5, a); 
        }

        fn bit_6_b(&mut self) { 
            let b = self.registers.b;
            self.bit_i_n(6, b); 
        }

        fn bit_6_c(&mut self) { 
            let c = self.registers.c;
            self.bit_i_n(6, c); 
        }

        fn bit_6_d(&mut self) { 
            let d = self.registers.d;
            self.bit_i_n(6, d); 
        }

        fn bit_6_e(&mut self) { 
            let e = self.registers.e;
            self.bit_i_n(6, e);
        }

        fn bit_6_h(&mut self) { 
            let h = self.registers.h;
            self.bit_i_n(6, h); 
        }

        fn bit_6_l(&mut self) { 
            let l = self.registers.l;
            self.bit_i_n(6, l); 
        }

        fn bit_6_hl(&mut self, memory: &Memory) { 
            let hl = self.registers.get_hl();
            self.bit_i_n(6, memory[hl]); 
        }

        fn bit_6_a(&mut self) { 
            let a = self.registers.a;
            self.bit_i_n(6, a); 
        }

        fn bit_7_b(&mut self) { 
            let b = self.registers.b;
            self.bit_i_n(7, b); 
        }

        fn bit_7_c(&mut self) { 
            let c = self.registers.c;
            self.bit_i_n(7, c); 
        }

        fn bit_7_d(&mut self) { 
            let d = self.registers.d;
            self.bit_i_n(7, d);
        }

        fn bit_7_e(&mut self) { 
            let e = self.registers.e;
            self.bit_i_n(7, e); 
        }

        fn bit_7_h(&mut self) { 
            let h = self.registers.h;
            self.bit_i_n(7, h); 
        }

        fn bit_7_l(&mut self) { 
            let l = self.registers.l;
            self.bit_i_n(7, l); 
        }

        fn bit_7_hl(&mut self, memory: &Memory) { 
            let hl = self.registers.get_hl();
            self.bit_i_n(7, memory[hl]); 
        }

        fn bit_7_a(&mut self) { 
            let a = self.registers.a;
            self.bit_i_n(7, a); 
        }
        fn res_0_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.res_i_n(0, b); 
        }

        fn res_0_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.res_i_n(0, c); 
        }
        fn res_0_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.res_i_n(0, d); 
        }
        fn res_0_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.res_i_n(0, e); 
        }
        fn res_0_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.res_i_n(0, h); 
        }
        fn res_0_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.res_i_n(0, l); 
        }
        fn res_0_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.res_i_n(0, memory[hl]);
        }

        fn res_0_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.res_i_n(0, a); 
        }

        fn res_1_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.res_i_n(1, b); 
        }

        fn res_1_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.res_i_n(1, c); 
        }

        fn res_1_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.res_i_n(1, d); 
        }

        fn res_1_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.res_i_n(1, e); 
        }

        fn res_1_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.res_i_n(1, h); 
        }

        fn res_1_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.res_i_n(1, l); 
        }

        fn res_1_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.res_i_n(1, memory[hl]);
        }

        fn res_1_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.res_i_n(1, a); 
        }

        fn res_2_b(&mut self) {
            let b = self.registers.b;
            self.registers.b = self.res_i_n(2, b); 
        }

        fn res_2_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.res_i_n(2, c); 
        }

        fn res_2_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.res_i_n(2, d); 
        }

        fn res_2_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.res_i_n(2, e); 
        }

        fn res_2_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.res_i_n(2, h); 
        }

        fn res_2_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.res_i_n(2, l); 
        }

        fn res_2_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.res_i_n(2, memory[hl]);
        }

        fn res_2_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.res_i_n(2, a); 
        }

        fn res_3_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.res_i_n(3, b); 
        }

        fn res_3_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.res_i_n(3, c); 
        }

        fn res_3_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.res_i_n(3, d); 
        }

        fn res_3_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.res_i_n(3, e); 
        }

        fn res_3_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.res_i_n(3, h); 
        }

        fn res_3_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.res_i_n(3, l); 
        }

        fn res_3_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.res_i_n(3, memory[hl]);
        }

        fn res_3_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.res_i_n(3, a); 
        }

        fn res_4_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.res_i_n(4, b); 
        }

        fn res_4_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.res_i_n(4, c); 
        }

        fn res_4_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.res_i_n(4, d); 
        }

        fn res_4_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.res_i_n(4, e); 
        }

        fn res_4_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.res_i_n(4, h); 
        }

        fn res_4_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.res_i_n(4, l); 
        }

        fn res_4_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.res_i_n(4, memory[hl]);
        }

        fn res_4_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.res_i_n(4, a); 
        }

        fn res_5_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.res_i_n(5, b); 
        }

        fn res_5_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.res_i_n(5, c); 
        }

        fn res_5_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.res_i_n(5, d); 
        }

        fn res_5_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.res_i_n(5, e); 
        }

        fn res_5_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.res_i_n(5, h); 
        }

        fn res_5_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.res_i_n(5, l); 
        }

        fn res_5_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.res_i_n(5, memory[hl]);
        }

        fn res_5_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.res_i_n(5, a); 
        }

        fn res_6_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.res_i_n(6, b); 
        }

        fn res_6_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.res_i_n(6, c); 
        }

        fn res_6_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.res_i_n(6, d); 
        }

        fn res_6_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.res_i_n(6, e); 
        }

        fn res_6_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.res_i_n(6, h); 
        }

        fn res_6_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.res_i_n(6, l); 
        }

        fn res_6_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.res_i_n(6, memory[hl]);
        }

        fn res_6_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.res_i_n(6, a); 
        }

        fn res_7_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.res_i_n(7, b); 
        }

        fn res_7_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.res_i_n(7, c); 
        }

        fn res_7_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.res_i_n(7, d); 
        }

        fn res_7_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.res_i_n(7, e); 
        }

        fn res_7_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.res_i_n(7, h); 
        }

        fn res_7_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.res_i_n(7, l); 
        }

        fn res_7_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.res_i_n(6, memory[hl]);
        }

        fn res_7_a(&mut self) { 
            let a = self.registers.a;
            self.res_i_n(7, a); 
        }

        fn set_0_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.set_i_n(0, b); 
        }

        fn set_0_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.set_i_n(0, c); 
        }

        fn set_0_d(&mut self) {
            let d = self.registers.d;
            self.registers.d = self.set_i_n(0, d);
        }

        fn set_0_e(&mut self) {
            let e = self.registers.e;
            self.registers.e = self.set_i_n(0, e); 
        }

        fn set_0_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.set_i_n(0, h); 
        }

        fn set_0_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.set_i_n(0, l); 
        }

        fn set_0_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.set_i_n(0, memory[hl]);
        }

        fn set_0_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.set_i_n(0, a); 
        }

        fn set_1_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.set_i_n(0, b); 
        }

        fn set_1_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.set_i_n(0, c); 
        }

        fn set_1_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.set_i_n(0, d);
        }

        fn set_1_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.set_i_n(0, e); 
        }

        fn set_1_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.set_i_n(0, h); 
        }

        fn set_1_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.set_i_n(0, l); 
        }

        fn set_1_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.set_i_n(1, memory[hl]);
        }

        fn set_1_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.set_i_n(1, a); 
        }

        fn set_2_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.set_i_n(2, b); 
        }

        fn set_2_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.set_i_n(2, c); 
        }

        fn set_2_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.set_i_n(2, d); 
        }

        fn set_2_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.set_i_n(2, e); 
        }

        fn set_2_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.set_i_n(2, h); 
        }

        fn set_2_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.set_i_n(2, l); 
        }

        fn set_2_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.set_i_n(2, memory[hl]);
        }

        fn set_2_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.set_i_n(2, a); 
        }

        fn set_3_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.set_i_n(3, b); 
        }

        fn set_3_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.set_i_n(3, c); 
        }

        fn set_3_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.set_i_n(3, d); 
        }

        fn set_3_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.set_i_n(3, e); 
        }

        fn set_3_h(&mut self) {
            let h = self.registers.h;
            self.registers.h = self.set_i_n(3, h);
        }

        fn set_3_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.set_i_n(3, l); 
        }

        fn set_3_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.set_i_n(3, memory[hl]);
        }

        fn set_3_a(&mut self) {
            let a = self.registers.a;
            self.registers.a = self.set_i_n(3, a); 
        }

        fn set_4_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.set_i_n(4, b); 
        }

        fn set_4_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.set_i_n(4, c);
        }

        fn set_4_d(&mut self) {
            let d = self.registers.d;
            self.registers.d = self.set_i_n(4, d); 
        }

        fn set_4_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.set_i_n(4, e); 
        }

        fn set_4_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.set_i_n(4, h);
        }

        fn set_4_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.set_i_n(4, l); 
        }

        fn set_4_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.set_i_n(4, memory[hl]);
        }

        fn set_4_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.set_i_n(3, a); 
        }

        fn set_5_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.set_i_n(5, b); 
        }

        fn set_5_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.set_i_n(5, c); 
        }

        fn set_5_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.set_i_n(5, d); 
        }

        fn set_5_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.set_i_n(5, e); 
        }

        fn set_5_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.set_i_n(5, h); 
        }

        fn set_5_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.set_i_n(5, l); 
        }

        fn set_5_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.set_i_n(5, memory[hl]);
        }

        fn set_5_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.set_i_n(5, a); 
        }

        fn set_6_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.set_i_n(6, b); 
        }

        fn set_6_c(&mut self) { 
            let c = self.registers.c;
            self.registers.c = self.set_i_n(6, c); 
        }

        fn set_6_d(&mut self) { 
            let d = self.registers.d;
            self.registers.d = self.set_i_n(6, d); 
        }

        fn set_6_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.set_i_n(6, e); 
        }

        fn set_6_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.set_i_n(6, h); 
        }

        fn set_6_l(&mut self) {
            let l = self.registers.l;
            self.registers.l = self.set_i_n(6, l); 
        }

        fn set_6_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.set_i_n(6, memory[hl]);
        }

        fn set_6_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.set_i_n(6, a); 
        }

        fn set_7_b(&mut self) { 
            let b = self.registers.b;
            self.registers.b = self.set_i_n(7, b); 
        }

        fn set_7_c(&mut self) {
            let c = self.registers.c; 
            self.registers.c = self.set_i_n(7, c); 
        }

        fn set_7_d(&mut self) {
            let d = self.registers.d;
            self.registers.d = self.set_i_n(7, d); 
        }

        fn set_7_e(&mut self) { 
            let e = self.registers.e;
            self.registers.e = self.set_i_n(7, e);
        }

        fn set_7_h(&mut self) { 
            let h = self.registers.h;
            self.registers.h = self.set_i_n(7, h); 
        }

        fn set_7_l(&mut self) { 
            let l = self.registers.l;
            self.registers.l = self.set_i_n(7, l); 
        }

        fn set_7_hl(&mut self, memory: &mut Memory) {
            let hl = self.registers.get_hl();
            memory[hl] = self.set_i_n(7, memory[hl]);
        }

        fn set_7_a(&mut self) { 
            let a = self.registers.a;
            self.registers.a = self.set_i_n(6, a); 
        }

    //interrupts

    pub fn rst_40(&mut self, memory: &mut Memory) { self.rst_n(0x40, memory); }
    pub fn rst_48(&mut self, memory: &mut Memory) { self.rst_n(0x48, memory); }
    pub fn rst_50(&mut self, memory: &mut Memory) { self.rst_n(0x50, memory); }
    pub fn rst_58(&mut self, memory: &mut Memory) { self.rst_n(0x58, memory); }
    pub fn rst_60(&mut self, memory: &mut Memory) { self.rst_n(0x60, memory); }

    //helpers

    fn rst_n(&mut self, n: u8, memory: &mut Memory) {
        let pc = self.registers.pc;
        self.push_nn(pc, memory);
        self.registers.pc = n as u16;
    }

    fn dec_n(&mut self, n: u8) -> u8 {
        // convert 1 into a 2's complement signed value
        // then add it to n, checking for the half carry
        let half_carry = ((n & 0xF) + (0xF)) & 0x10 == 0x10;

        let n = n.wrapping_sub(1);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.insert(Flag::NEGATIVE);

        n
    }

    fn add_hl_ss(&mut self, ss: u16) {
        let mut hl = self.registers.get_hl();

        let half_carry = (((hl & 0x3FFF) + (ss & 0x3FFF)) & 0x4000) == 0x4000;
        let full_carry = ((hl as u32) + (ss as u32) & 0x10000) == 0x10000;

        hl = hl.wrapping_add(ss);

        self.registers.f.remove(Flag::NEGATIVE);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);

        self.registers.set_hl(hl);
    }

    fn inc_n(&mut self, n: u8) -> u8 {
        let half_carry = (((n & 0xF) + 1) & 0x10) == 0x10;

        let n = n.wrapping_add(1);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.remove(Flag::NEGATIVE);

        n
    }

    fn bit_i_n(&mut self, i: u8, n: u8) {
        self.registers.f.insert(Flag::HALF_CARRY);
        self.registers.f.remove(Flag::NEGATIVE);
        self.registers.f.set(Flag::ZERO, n & (1 << i) == 0);
    }

    fn res_i_n(&mut self, i: u8, n: u8) -> u8 {
        n & !((1 << i) as u8)
    }

    fn set_i_n(&mut self, i: u8, n: u8) -> u8 {
        n | (1 << i)
    }

    fn rlc_n(&mut self, n: u8) -> u8 {
        let left_bit = (n & 0x80) == 0x80;

        println!("{:?}", left_bit);
        println!("{:?}", n);

        let n = n.rotate_left(1);

        self.registers.f.set(Flag::FULL_CARRY, left_bit);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        n
    }

    fn rrc_n(&mut self, n: u8) -> u8 {
        let right_bit = (n & 0x1) == 1;
        let n = n.rotate_right(1);

        self.registers.f.set(Flag::FULL_CARRY, right_bit);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        n
    }

    fn rr_n(&mut self, n: u8) -> u8 {
        let right_bit = (n & 0x1) == 1;
        let mut n = n >> 1;

        n |= (self.registers.f.contains(Flag::FULL_CARRY) as u8) << 7;

        self.registers.f.set(Flag::FULL_CARRY, right_bit);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        n
    }

    fn rl_n(&mut self, n: u8) -> u8 {
        let left_bit = (n & 0x80) == 0x80;
        let mut n = n << 1;
        n |= self.registers.f.contains(Flag::FULL_CARRY) as u8;

        self.registers.f.set(Flag::FULL_CARRY, left_bit);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        n
    }

    fn sla_n(&mut self, n: u8) -> u8 {
        let left_bit = (n & 0x80) == 0x80;
        let n = n << 1;

        self.registers.f.set(Flag::FULL_CARRY, left_bit);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        n
    }

    fn sra_n(&mut self, n: u8) -> u8 {
        let left = n & 0x80;
        let right_bit = (n & 0x1) == 1;

        let mut n = n >> 1;
        n |= left;

        self.registers.f.set(Flag::FULL_CARRY, right_bit);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        n
    }

    fn jr_cc_n(&mut self, cc: bool, n: u8) {
        if cc {
            self.jr_n(n);
        }
    }

    fn jp_cc_nn(&mut self, cc: bool, nn: u16) {
        if cc {
            self.jp_nn(nn);
        }
    }

    fn inc_nn(&self, nn: u16) -> u16 { nn + 1 }

    fn dec_nn(&self, nn: u16) -> u16 { nn - 1 }

    fn call_cc_nn(&mut self, cc: bool, nn: u16, memory: &mut Memory) {
        if cc {
            self.call_nn(nn, memory);
        }
    }

    fn ret_cc(&mut self, cc: bool, memory: &Memory) {
        if cc {
            self.ret(memory);
        }
    }

    fn pop(&mut self, memory: &Memory) -> u16 {
        let word = memory.read_word(self.registers.sp);
        self.registers.sp += 2;

        word
    }

    fn push_nn(&mut self, nn: u16, memory: &mut Memory) {
        self.registers.sp -= 2;
        memory.write_word(self.registers.sp, nn);
    }

    fn swap_n(&mut self, n: u8) -> u8 {
        let high = n & 0xF0;
        let low = n & 0xF;

        let n = (low << 4) | (high >> 4);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY);

        n
    }

    fn srl_n(&mut self, n: u8) -> u8 {
        let first = n & 1 == 1;
        let n = n >> 1;

        self.registers.f.set(Flag::FULL_CARRY, first);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::HALF_CARRY | Flag::NEGATIVE);        

        n
    }
}

#[cfg(test)]
mod tests {

    use cpu::registers::flag::Flag;
    use cpu::Cpu;

    #[test]
    fn test_inc_nn() {
        let cpu = Cpu::new();
        let mut n = 1;

        n = cpu.inc_nn(n);

        assert_eq!(n, 2);
    }

    #[test]
    fn test_dec_nn() {
        let cpu = Cpu::new();
        let mut n = 1;

        n = cpu.dec_nn(n);

        assert_eq!(n, 0);
    }

    #[test]
    fn test_inc_n() {

        // cause an 8 bit overflow, triggering a zero
        // and half carry flag
        let mut cpu = Cpu::new();
        let mut n = 0xFF;

        n = cpu.inc_n(n);

        assert_eq!(n, 0);
        assert_eq!(cpu.registers.f, Flag::ZERO | Flag::HALF_CARRY);
    }

    #[test]
    fn test_dec_n() {
        let mut cpu = Cpu::new();
        let mut n = 1;

        n = cpu.dec_n(n);

        assert_eq!(n, 0);
        assert_eq!(cpu.registers.f, Flag::ZERO | Flag::NEGATIVE | Flag::HALF_CARRY);
    }

    #[test]
    fn test_add_a_n() {
        let mut cpu = Cpu::new();
        let n = 0xF1;
        cpu.registers.a = 0xF;

        cpu.add_a_n(n);

        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.f, Flag::ZERO | Flag::HALF_CARRY | Flag::FULL_CARRY);
    }

    #[test]
    fn test_sub_a_n() {
        let mut cpu = Cpu::new();
        let n = 0xF;
        cpu.registers.a = 0xF;

        cpu.sub_a_n(n);

        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.f, Flag::ZERO | Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY);
    }

    #[test]
    fn test_adc_a_n() {
        let mut cpu = Cpu::new();
        let n = 0xF0;
        cpu.registers.a = 0xF;
        cpu.registers.f |= Flag::FULL_CARRY;

        cpu.adc_a_n(n);

        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.f, Flag::ZERO | Flag::HALF_CARRY | Flag::FULL_CARRY);
    }

    #[test]
    fn test_sbc_a_n() {
        let mut cpu = Cpu::new();
        let n = 0xE;
        cpu.registers.a = 0xF;
        cpu.registers.f |= Flag::FULL_CARRY;

        cpu.sbc_a_n(n);

        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.f, Flag::ZERO | Flag::NEGATIVE);
    }

    #[test]
    fn test_add_hl_ss() {
        let mut cpu = Cpu::new();
        let ss = 0x7F7F;
        cpu.registers.set_hl(0x7F7F);

        cpu.add_hl_ss(ss);

        assert_eq!(cpu.registers.get_hl(), 0xFEFE);
        assert_eq!(cpu.registers.f, Flag::HALF_CARRY);
    }

    #[test]
    fn test_bit_i_n() {
        let mut cpu = Cpu::new();

        cpu.bit_i_n(0, 1);

        assert_eq!(cpu.registers.f, Flag::HALF_CARRY);
    }

    #[test]
    fn test_set_i_n() {
        let mut cpu = Cpu::new();

        let n = cpu.set_i_n(0, 0);

        assert_eq!(n, 1);
    }

    #[test]
    fn test_res_i_n() {
        let mut cpu = Cpu::new();

        let n = cpu.res_i_n(0, 1);

        assert_eq!(n, 0);
    }

    #[test]
    fn test_rlc_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0b00000000;
        cpu.registers.f |= Flag::ZERO | Flag::HALF_CARRY | Flag::NEGATIVE | Flag::FULL_CARRY;

        cpu.rlc_a();

        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.f, Flag::ZERO);
    }

    #[test]
    fn test_rlc_n() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0b10101010;
        cpu.registers.f |= Flag::ZERO | Flag::FULL_CARRY | Flag::HALF_CARRY | Flag::NEGATIVE;

        cpu.rlc_a();

        assert_eq!(cpu.registers.a, 0b01010101);
        assert_eq!(cpu.registers.f, Flag::FULL_CARRY);
    }

    #[test]
    fn test_rrc_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0b10101010;
        cpu.registers.f |= Flag::ZERO | Flag::FULL_CARRY | Flag::HALF_CARRY | Flag::NEGATIVE;

        cpu.rrc_a();

        assert_eq!(cpu.registers.a, 0b01010101);
        assert!(cpu.registers.f.is_empty());
    }

    #[test]
    fn test_rrc_n() {
        let mut cpu = Cpu::new();
        cpu.registers.f |= Flag::FULL_CARRY;

        let n = cpu.rrc_n(0);

        assert_eq!(n, 0);
        assert_eq!(cpu.registers.f, Flag::ZERO);
    }

    #[test]
    fn test_rl_n() {
        let mut cpu = Cpu::new();
        cpu.registers.f |= Flag::FULL_CARRY;

        let n = cpu.rl_n(0b00101010);

        assert_eq!(n, 0b01010101);
        assert!(cpu.registers.f.is_empty());
    }

    #[test]
    fn test_rr_n() {
        let mut cpu = Cpu::new();
        cpu.registers.f |= Flag::FULL_CARRY;

        let n = cpu.rr_n(0b10101010);

        assert_eq!(n, 0b11010101);
        assert!(cpu.registers.f.is_empty());
    }

    #[test]
    fn test_sla_n() {
        let mut cpu = Cpu::new();

        let n = cpu.sla_n(0b10000000);

        assert_eq!(n, 0);
        assert_eq!(cpu.registers.f, Flag::FULL_CARRY | Flag::ZERO);
    }

    #[test]
    fn test_sra_n() {
        let mut cpu = Cpu::new();

        let n = cpu.sra_n(0b10011001);

        assert_eq!(n, 0b11001100);
        assert_eq!(cpu.registers.f, Flag::FULL_CARRY);
    }

    #[test]
    fn test_srl_n() {
        let mut cpu = Cpu::new();

        let n = cpu.srl_n(0b11111111);

        assert_eq!(n, 0b01111111);
        assert_eq!(cpu.registers.f, Flag::FULL_CARRY);
    }
}