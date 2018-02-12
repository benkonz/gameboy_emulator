mod registers;
mod alu;
mod function;
mod stack;

use self::registers::Registers;
use self::registers::flag::Flag;
use mmu::Memory;
use std::u8;

pub struct Cpu {
    registers: Registers,
    pub stopped: bool,
    pub halted: bool,
    pub interrupt_enabled: bool,
    pub cycles: u64,
    instruction_cycle: i32,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Default::default(),
            stopped: false,
            halted: false,
            interrupt_enabled: false,
            cycles: 0,
            instruction_cycle: 0,
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

    pub fn step(&mut self, memory: &mut Memory) -> i32 {
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
            }
            0x0F => self.rrc_a(),
            0x10 => self.stop(),
            0x11 => {
                let nn = self.get_nn(memory);
                self.ld_de_nn(nn);
            }
            0x12 => self.ld_de_a(memory),
            0x13 => self.inc_de(),
            0x14 => self.inc_d(),
            0x15 => self.dec_d(),
            0x16 => {
                let n = self.get_n(memory);
                self.ld_d_n(n);
            }
            0x17 => self.rl_a(),
            0x18 => {
                let n = self.get_n(memory);
                self.jr_n(n);
            }
            0x19 => self.add_hl_de(),
            0x1A => self.ld_a_de(memory),
            0x1B => self.dec_de(),
            0x1C => self.inc_e(),
            0x1D => self.dec_e(),
            0x1E => {
                let n = self.get_n(memory);
                self.ld_e_n(n);
            }
            0x1F => self.rr_a(),
            0x20 => {
                let n = self.get_n(memory);
                self.jr_nz_n(n);
            }
            0x21 => {
                let nn = self.get_nn(memory);
                self.ld_hl_nn(nn);
            }
            0x22 => self.ldi_hl_a(memory),
            0x23 => self.inc_hl(),
            0x24 => self.inc_h(),
            0x25 => self.dec_h(),
            0x26 => {
                let n = self.get_n(memory);
                self.ld_h_n(n);
            }
            0x27 => self.daa(),
            0x28 => {
                let n = self.get_n(memory);
                self.jr_z_n(n);
            }
            0x29 => self.add_hl_hl(),
            0x2A => self.ldi_a_hl(memory),
            0x2B => self.dec_hl(),
            0x2C => self.inc_l(),
            0x2D => self.dec_l(),
            0x2E => {
                let n = self.get_n(memory);
                self.ld_l_n(n);
            }
            0x2F => self.cpl(),
            0x30 => {
                let n = self.get_n(memory);
                self.jr_nc_n(n);
            }
            0x31 => {
                let nn = self.get_nn(memory);
                self.ld_sp_nn(nn);
            }
            0x32 => self.ldd_hl_a(memory),
            0x33 => self.inc_sp(),
            0x34 => self.inc_hl_ref(memory),
            0x35 => self.dec_hl_ref(memory),
            0x36 => {
                let n = self.get_n(memory);
                self.ld_hl_n(n, memory);
            }
            0x37 => self.scf(),
            0x38 => {
                let n = self.get_n(memory);
                self.jr_c_n(n);
            }
            0x39 => self.add_hl_sp(),
            0x3A => self.ldd_a_hl(memory),
            0x3B => self.dec_sp(),
            0x3C => self.inc_a(),
            0x3D => self.dec_a(),
            0x3E => {
                let n = self.get_n(memory);
                self.ld_a_n(n);
            }
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
            }
            0xC3 => {
                let nn = self.get_nn(memory);
                self.jp_nn(nn);
            }
            0xC4 => {
                let nn = self.get_nn(memory);
                self.call_nz_nn(memory, nn);
            }
            0xC5 => self.push_bc(memory),
            0xC6 => {
                let n = self.get_n(memory);
                self.add_a_n(n);
            }
            0xC7 => self.rst_0(memory),
            0xC8 => self.ret_z(memory),
            0xC9 => self.ret(memory),
            0xCA => {
                let nn = self.get_nn(memory);
                self.jp_z_nn(nn);
            }
            0xCB => {
                let n = self.get_n(memory);
                self.ext_ops(n, memory);
            }
            0xCC => {
                let nn = self.get_nn(memory);
                self.call_z_nn(nn, memory);
            }
            0xCD => {
                let nn = self.get_nn(memory);
                self.call_nn(nn, memory);
            }
            0xCE => {
                let n = self.get_n(memory);
                self.adc_a_n(n);
            }
            0xCF => self.rst_8(memory),
            0xD0 => self.ret_nc(memory),
            0xD1 => self.pop_de(memory),
            0xD2 => {
                let nn = self.get_nn(memory);
                self.jp_nc_nn(nn);
            }
            0xD3 => self.undefined(),
            0xD4 => {
                let nn = self.get_nn(memory);
                self.call_nc_nn(nn, memory);
            }
            0xD5 => self.push_de(memory),
            0xD6 => {
                let n = self.get_n(memory);
                self.sub_a_n(n);
            }
            0xD7 => self.rst_10(memory),
            0xD8 => self.ret_c(memory),
            0xD9 => self.ret_i(memory),
            0xDA => {
                let nn = self.get_nn(memory);
                self.jp_c_nn(nn);
            }
            0xDB => self.undefined(),
            0xDC => {
                let nn = self.get_nn(memory);
                self.call_c_nn(nn, memory);
            }
            0xDD => self.undefined(),
            0xDE => {
                let n = self.get_n(memory);
                self.sbc_a_n(n);
            }
            0xDF => self.rst_18(memory),
            0xE0 => {
                let n = self.get_n(memory);
                self.ldh_n_a(n, memory);
            }
            0xE1 => self.pop_hl(memory),
            0xE2 => self.ldh_c_a(memory),
            0xE3 => self.undefined(),
            0xE4 => self.undefined(),
            0xE5 => self.push_hl(memory),
            0xE6 => {
                let n = self.get_n(memory);
                self.and_n(n);
            }
            0xE7 => self.rst_20(memory),
            0xE8 => {
                let n = self.get_n(memory);
                self.add_sp_d(n);
            }
            0xE9 => self.jp_hl(memory),
            0xEA => {
                let nn = self.get_nn(memory);
                self.ld_nn_a(nn, memory);
            }
            0xEB => self.undefined(),
            0xEC => self.undefined(),
            0xED => self.undefined(),
            0xEE => {
                let n = self.get_n(memory);
                self.xor_n(n);
            }
            0xEF => self.rst_28(memory),
            0xF0 => {
                let n = self.get_n(memory);
                self.ldh_a_n(n, memory);
            }
            0xF1 => self.pop_af(memory),
            0xF2 => self.undefined(),
            0xF3 => self.di(),
            0xF4 => self.undefined(),
            0xF5 => self.push_af(memory),
            0xF6 => {
                let n = self.get_n(memory);
                self.or_n(n);
            }
            0xF7 => self.rst_30(memory),
            0xF8 => {
                let n = self.get_n(memory);
                self.ldhl_sp_d(n, memory);
            }
            0xF9 => self.ld_sp_hl(),
            0xFA => {
                let n = self.get_nn(memory);
                self.ld_a_nn(n, memory);
            }
            0xFB => self.ei(),
            0xFC => self.undefined(),
            0xFD => self.undefined(),
            0xFE => {
                let n = self.get_n(memory);
                self.cp_n(n);
            }
            0xFF => self.rst_38(memory),
            _ => {}
        }

        self.instruction_cycle
    }

    //opcodes

    fn nop(&mut self) {
        self.instruction_cycle = 1;
    }

    fn ld_bc_nn(&mut self, nn: u16) {
        self.registers.set_bc(nn);
        self.instruction_cycle = 3;
    }

    fn ld_bc_a(&mut self, memory: &mut Memory) {
        let bc = self.registers.get_bc();
        memory.write_byte(bc, self.registers.a);
        self.instruction_cycle = 2;
    }

    fn inc_bc(&mut self) {
        let mut bc = self.registers.get_bc();
        bc = alu::inc_nn(bc);
        self.registers.set_bc(bc);
        self.instruction_cycle = 2;
    }

    fn inc_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = alu::inc_n(b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn dec_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = alu::dec_n(b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ld_b_n(&mut self, n: u8) {
        self.registers.b = n;
        self.instruction_cycle = 2;
    }

    fn rlc_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = alu::rlc_a(a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ld_nn_sp(&mut self, nn: u16, memory: &mut Memory) {
        memory.write_word(nn, self.registers.sp);
        self.instruction_cycle = 5;
    }

    fn add_hl_bc(&mut self) {
        let hl = self.registers.get_hl();
        let mut bc = self.registers.get_bc();
        bc = alu::add_hl_nn(hl, bc, &mut self.registers.f);
        self.registers.set_bc(bc);
        self.instruction_cycle = 2;
    }

    fn ld_a_bc(&mut self, memory: &Memory) {
        self.registers.a = memory.read_byte(self.registers.get_bc());
        self.instruction_cycle = 2;
    }

    fn dec_bc(&mut self) {
        let mut bc = self.registers.get_bc();
        bc = alu::dec_nn(bc);
        self.registers.set_bc(bc);
        self.instruction_cycle = 2;
    }
    fn inc_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = alu::inc_n(c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }
    fn dec_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = alu::dec_n(c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ld_c_n(&mut self, n: u8) {
        self.registers.c = n;
        self.instruction_cycle = 2;
    }

    fn rrc_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = alu::rrc_a(a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn stop(&mut self) {
        self.stopped = true;
        self.instruction_cycle = 1;
    }

    fn ld_de_nn(&mut self, nn: u16) {
        self.registers.set_de(nn);
        self.instruction_cycle = 3;
    }

    fn ld_de_a(&mut self, memory: &mut Memory) {
        let de = self.registers.get_de();
        memory.write_byte(de, self.registers.a);
        self.instruction_cycle = 2;
    }

    fn inc_de(&mut self) {
        let mut de = self.registers.get_de();
        de = alu::inc_nn(de);
        self.registers.set_de(de);
        self.instruction_cycle = 2;
    }

    fn inc_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = alu::inc_n(d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn dec_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = alu::dec_n(d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ld_d_n(&mut self, n: u8) {
        self.registers.d = n;
        self.instruction_cycle = 2;
    }

    fn rl_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = alu::rl_a(a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn jr_n(&mut self, n: u8) {
        let n = n as i8 as u16;
        self.registers.pc = self.registers.pc.wrapping_add(n);
        self.instruction_cycle = 3;
    }

    fn add_hl_de(&mut self) {
        let hl = self.registers.get_hl();
        let mut de = self.registers.get_de();
        de = alu::add_hl_nn(hl, de, &mut self.registers.f);
        self.registers.set_de(de);
        self.instruction_cycle = 2;
    }

    fn ld_a_de(&mut self, memory: &Memory) {
        self.registers.a = memory.read_byte(self.registers.get_de());
        self.instruction_cycle = 2;
    }

    fn dec_de(&mut self) {
        let mut de = self.registers.get_de();
        de = alu::dec_nn(de);
        self.registers.set_de(de);
        self.instruction_cycle = 2;
    }

    fn inc_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = alu::inc_n(e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn dec_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = alu::dec_n(e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ld_e_n(&mut self, n: u8) {
        self.registers.e = n;
        self.instruction_cycle = 2;
    }

    fn rr_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = alu::rr_a(a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn jr_nz_n(&mut self, n: u8) {
        if !self.registers.f.contains(Flag::ZERO) {
            self.registers.pc = self.registers.pc.wrapping_add(n as i8 as u16);
            self.instruction_cycle = 3;
        } else {
            self.instruction_cycle = 2;
        }
    }

    fn ld_hl_nn(&mut self, nn: u16) {
        self.registers.set_hl(nn);
        self.instruction_cycle = 3;
    }

    fn ldi_hl_a(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.a);
        self.registers.set_hl(hl.wrapping_add(1));
        self.instruction_cycle = 2;
    }

    fn inc_hl(&mut self) {
        let mut hl = self.registers.get_hl();
        hl = alu::inc_nn(hl);
        self.registers.set_hl(hl);
        self.instruction_cycle = 2;
    }

    fn inc_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = alu::inc_n(h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn dec_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = alu::dec_n(h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ld_h_n(&mut self, n: u8) {
        self.registers.h = n;
        self.instruction_cycle = 2;
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

        self.instruction_cycle = 1;
    }

    fn jr_z_n(&mut self, n: u8) {
        if self.registers.f.contains(Flag::ZERO) {
            self.registers.pc = self.registers.pc.wrapping_add(n as i8 as u16);
            self.instruction_cycle = 3;
        } else {
            self.instruction_cycle = 2;
        }
    }

    fn add_hl_hl(&mut self) {
        let mut hl = self.registers.get_hl();
        hl = alu::add_hl_nn(hl, hl, &mut self.registers.f);
        self.registers.set_hl(hl);
        self.instruction_cycle = 2;
    }

    fn ldi_a_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.a = memory.read_byte(hl);
        self.registers.set_hl(hl.wrapping_add(1));
        self.instruction_cycle = 2;
    }

    fn dec_hl(&mut self) {
        let mut hl = self.registers.get_hl();
        hl = alu::dec_nn(hl);
        self.registers.set_hl(hl);
        self.instruction_cycle = 2;
    }

    fn inc_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = alu::inc_n(l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn dec_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = alu::dec_n(l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ld_l_n(&mut self, n: u8) {
        self.registers.l = n;
        self.instruction_cycle = 2;
    }

    fn cpl(&mut self) {
        self.registers.a = !self.registers.a;
        self.registers.f.insert(Flag::HALF_CARRY | Flag::NEGATIVE);
        self.instruction_cycle = 1;
    }

    fn jr_nc_n(&mut self, n: u8) {
        if !self.registers.f.contains(Flag::FULL_CARRY) {
            self.registers.pc = self.registers.pc.wrapping_add(n as i8 as u16);
            self.instruction_cycle = 3;
        } else {
            self.instruction_cycle = 2;
        }
    }

    fn ld_sp_nn(&mut self, nn: u16) {
        self.registers.sp = nn;
        self.instruction_cycle = 3;
    }

    fn ldd_hl_a(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.a);
        self.registers.set_hl(hl.wrapping_sub(1));
        self.instruction_cycle = 2;
    }

    fn inc_sp(&mut self) {
        self.registers.sp = alu::inc_nn(self.registers.sp);
        self.instruction_cycle = 2;
    }

    fn inc_hl_ref(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::inc_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 3;
    }

    fn dec_hl_ref(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::dec_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 3;
    }

    fn ld_hl_n(&mut self, n: u8, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, n);
        self.instruction_cycle = 3;
    }

    fn scf(&mut self) {
        self.registers.f.insert(Flag::FULL_CARRY);
        self.instruction_cycle = 1;
    }

    fn jr_c_n(&mut self, n: u8) {
        if self.registers.f.contains(Flag::FULL_CARRY) {
            self.registers.pc = self.registers.pc.wrapping_add(n as i8 as u16);
            self.instruction_cycle = 3;
        } else {
            self.instruction_cycle = 2;
        }
    }

    fn add_hl_sp(&mut self) {
        let hl = self.registers.get_hl();
        let sp = self.registers.sp;
        self.registers.sp = alu::add_hl_nn(sp, hl, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn ldd_a_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.a = memory.read_byte(hl);
        self.registers.set_hl(hl.wrapping_sub(1));
        self.instruction_cycle = 2;
    }

    fn dec_sp(&mut self) {
        let sp = self.registers.sp;
        self.registers.sp = alu::dec_nn(sp);
        self.instruction_cycle = 2;
    }

    fn inc_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = alu::inc_n(a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn dec_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = alu::dec_n(a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ld_a_n(&mut self, n: u8) {
        self.registers.a = n;
        self.instruction_cycle = 2;
    }

    fn ccf(&mut self) {
        self.registers.f.toggle(Flag::FULL_CARRY);
        self.instruction_cycle = 1;
    }

    fn ld_b_b(&mut self) {
        self.registers.b = self.registers.b;
        self.instruction_cycle = 1;
    }
    fn ld_b_c(&mut self) {
        self.registers.b = self.registers.c;
        self.instruction_cycle = 1;
    }
    fn ld_b_d(&mut self) {
        self.registers.b = self.registers.d;
        self.instruction_cycle = 1;
    }
    fn ld_b_e(&mut self) {
        self.registers.b = self.registers.e;
        self.instruction_cycle = 1;
    }
    fn ld_b_h(&mut self) {
        self.registers.b = self.registers.h;
        self.instruction_cycle = 1;
    }
    fn ld_b_l(&mut self) {
        self.registers.b = self.registers.l;
        self.instruction_cycle = 1;
    }
    fn ld_b_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.b = memory.read_byte(hl);
        self.instruction_cycle = 2;
    }
    fn ld_b_a(&mut self) {
        self.registers.b = self.registers.a;
        self.instruction_cycle = 1;
    }

    fn ld_c_b(&mut self) {
        self.registers.c = self.registers.b;
        self.instruction_cycle = 1;
    }
    fn ld_c_c(&mut self) {
        self.registers.c = self.registers.c;
        self.instruction_cycle = 1;
    }
    fn ld_c_d(&mut self) {
        self.registers.c = self.registers.d;
        self.instruction_cycle = 1;
    }
    fn ld_c_e(&mut self) {
        self.registers.c = self.registers.e;
        self.instruction_cycle = 1;
    }
    fn ld_c_h(&mut self) {
        self.registers.c = self.registers.h;
        self.instruction_cycle = 1;
    }
    fn ld_c_l(&mut self) {
        self.registers.c = self.registers.l;
        self.instruction_cycle = 1;
    }
    fn ld_c_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.c = memory.read_byte(hl);
        self.instruction_cycle = 2;
    }
    fn ld_c_a(&mut self) {
        self.registers.c = self.registers.a;
        self.instruction_cycle = 1;
    }

    fn ld_d_b(&mut self) {
        self.registers.d = self.registers.b;
        self.instruction_cycle = 1;
    }
    fn ld_d_c(&mut self) {
        self.registers.d = self.registers.c;
        self.instruction_cycle = 1;
    }
    fn ld_d_d(&mut self) {
        self.registers.d = self.registers.d;
        self.instruction_cycle = 1;
    }
    fn ld_d_e(&mut self) {
        self.registers.d = self.registers.e;
        self.instruction_cycle = 1;
    }
    fn ld_d_h(&mut self) {
        self.registers.d = self.registers.h;
        self.instruction_cycle = 1;
    }
    fn ld_d_l(&mut self) {
        self.registers.d = self.registers.l;
        self.instruction_cycle = 1;
    }
    fn ld_d_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.d = memory.read_byte(hl);
        self.instruction_cycle = 2;
    }
    fn ld_d_a(&mut self) {
        self.registers.d = self.registers.a;
        self.instruction_cycle = 1;
    }

    fn ld_e_b(&mut self) {
        self.registers.e = self.registers.b;
        self.instruction_cycle = 1;
    }
    fn ld_e_c(&mut self) {
        self.registers.e = self.registers.c;
        self.instruction_cycle = 1;
    }
    fn ld_e_d(&mut self) {
        self.registers.e = self.registers.d;
        self.instruction_cycle = 1;
    }
    fn ld_e_e(&mut self) {
        self.registers.e = self.registers.e;
        self.instruction_cycle = 1;
    }
    fn ld_e_h(&mut self) {
        self.registers.e = self.registers.h;
        self.instruction_cycle = 1;
    }
    fn ld_e_l(&mut self) {
        self.registers.e = self.registers.l;
        self.instruction_cycle = 1;
    }
    fn ld_e_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.e = memory.read_byte(hl);
        self.instruction_cycle = 2;
    }
    fn ld_e_a(&mut self) {
        self.registers.e = self.registers.a;
        self.instruction_cycle = 1;
    }

    fn ld_h_b(&mut self) {
        self.registers.h = self.registers.b;
        self.instruction_cycle = 1;
    }
    fn ld_h_c(&mut self) {
        self.registers.h = self.registers.c;
        self.instruction_cycle = 1;
    }
    fn ld_h_d(&mut self) {
        self.registers.h = self.registers.d;
        self.instruction_cycle = 1;
    }
    fn ld_h_e(&mut self) {
        self.registers.h = self.registers.e;
        self.instruction_cycle = 1;
    }
    fn ld_h_h(&mut self) {
        self.registers.h = self.registers.h;
        self.instruction_cycle = 1;
    }
    fn ld_h_l(&mut self) {
        self.registers.h = self.registers.l;
        self.instruction_cycle = 1;
    }
    fn ld_h_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.h = memory.read_byte(hl);
    }
    fn ld_h_a(&mut self) {
        self.registers.h = self.registers.a;
        self.instruction_cycle = 1;
    }

    fn ld_l_b(&mut self) {
        self.registers.l = self.registers.b;
        self.instruction_cycle = 1;
    }
    fn ld_l_c(&mut self) {
        self.registers.l = self.registers.c;
        self.instruction_cycle = 1;
    }
    fn ld_l_d(&mut self) {
        self.registers.l = self.registers.d;
        self.instruction_cycle = 1;
    }
    fn ld_l_e(&mut self) {
        self.registers.l = self.registers.e;
        self.instruction_cycle = 1;
    }
    fn ld_l_h(&mut self) {
        self.registers.l = self.registers.h;
        self.instruction_cycle = 1;
    }
    fn ld_l_l(&mut self) {
        self.registers.l = self.registers.l;
        self.instruction_cycle = 1;
    }
    fn ld_l_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.l = memory.read_byte(hl);
        self.instruction_cycle = 2;
    }
    fn ld_l_a(&mut self) {
        self.registers.l = self.registers.a;
        self.instruction_cycle = 1;
    }

    fn ld_hl_b(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.b);
        self.instruction_cycle = 2;
    }
    fn ld_hl_c(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.c);
        self.instruction_cycle = 2;
    }
    fn ld_hl_d(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.d);
        self.instruction_cycle = 2;
    }
    fn ld_hl_e(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.e);
        self.instruction_cycle = 2;
    }
    fn ld_hl_h(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.h);
        self.instruction_cycle = 2;
    }
    fn ld_hl_l(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.l);
        self.instruction_cycle = 2;
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn ld_hl_a(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.a);
        self.instruction_cycle = 2;
    }

    fn ld_a_b(&mut self) {
        self.registers.a = self.registers.b;
        self.instruction_cycle = 1;
    }

    fn ld_a_c(&mut self) {
        self.registers.a = self.registers.c;
        self.instruction_cycle = 1;
    }

    fn ld_a_d(&mut self) {
        self.registers.a = self.registers.d;
        self.instruction_cycle = 1;
    }

    fn ld_a_e(&mut self) {
        self.registers.a = self.registers.e;
        self.instruction_cycle = 1;
    }

    fn ld_a_h(&mut self) {
        self.registers.a = self.registers.h;
        self.instruction_cycle = 1;
    }

    fn ld_a_l(&mut self) {
        self.registers.a = self.registers.l;
        self.instruction_cycle = 1;
    }

    fn ld_a_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.a = memory.read_byte(hl);
        self.instruction_cycle = 2;
    }

    fn ld_a_a(&mut self) {
        self.registers.a = self.registers.a;
        self.instruction_cycle = 1;
    }

    fn add_a_b(&mut self) {
        self.registers.a = alu::add_a_n(self.registers.a, self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn add_a_c(&mut self) {
        self.registers.a = alu::add_a_n(self.registers.a, self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn add_a_d(&mut self) {
        self.registers.a = alu::add_a_n(self.registers.a, self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn add_a_e(&mut self) {
        self.registers.a = alu::add_a_n(self.registers.a, self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn add_a_h(&mut self) {
        self.registers.a = alu::add_a_n(self.registers.a, self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn add_a_l(&mut self) {
        self.registers.a = alu::add_a_n(self.registers.a, self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn add_a_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        self.registers.a = alu::add_a_n(self.registers.a, hl, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn add_a_a(&mut self) {
        self.registers.a = alu::add_a_n(self.registers.a, self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn adc_a_b(&mut self) {
        self.registers.a = alu::adc_a_n(self.registers.a, self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn adc_a_c(&mut self) {
        self.registers.a = alu::adc_a_n(self.registers.a, self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn adc_a_d(&mut self) {
        self.registers.a = alu::adc_a_n(self.registers.a, self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn adc_a_e(&mut self) {
        self.registers.a = alu::adc_a_n(self.registers.a, self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn adc_a_h(&mut self) {
        self.registers.a = alu::adc_a_n(self.registers.a, self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn adc_a_l(&mut self) {
        self.registers.a = alu::adc_a_n(self.registers.a, self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn adc_a_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        self.registers.a = alu::adc_a_n(self.registers.a, hl, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn adc_a_a(&mut self) {
        self.registers.a = alu::adc_a_n(self.registers.a, self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sub_a_b(&mut self) {
        self.registers.a = alu::sub_a_n(self.registers.a, self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sub_a_c(&mut self) {
        self.registers.a = alu::sub_a_n(self.registers.a, self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sub_a_d(&mut self) {
        self.registers.a = alu::sub_a_n(self.registers.a, self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sub_a_e(&mut self) {
        self.registers.a = alu::sub_a_n(self.registers.a, self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sub_a_h(&mut self) {
        self.registers.a = alu::sub_a_n(self.registers.a, self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sub_a_l(&mut self) {
        self.registers.a = alu::sub_a_n(self.registers.a, self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sub_a_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        self.registers.a = alu::sub_a_n(self.registers.a, hl, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sub_a_a(&mut self) {
        self.registers.a = alu::sub_a_n(self.registers.a, self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sbc_a_b(&mut self) {
        self.registers.a = alu::sbc_a_n(self.registers.a, self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sbc_a_c(&mut self) {
        self.registers.a = alu::sbc_a_n(self.registers.a, self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sbc_a_d(&mut self) {
        self.registers.a = alu::sbc_a_n(self.registers.a, self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sbc_a_e(&mut self) {
        self.registers.a = alu::sbc_a_n(self.registers.a, self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sbc_a_h(&mut self) {
        self.registers.a = alu::sbc_a_n(self.registers.a, self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sbc_a_l(&mut self) {
        self.registers.a = alu::sbc_a_n(self.registers.a, self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn sbc_a_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        self.registers.a = alu::sbc_a_n(self.registers.a, hl, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sbc_a_a(&mut self) {
        self.registers.a = alu::sbc_a_n(self.registers.a, self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn and_b(&mut self) {
        self.registers.a = alu::and_a_n(self.registers.a, self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn and_c(&mut self) {
        self.registers.a = alu::and_a_n(self.registers.a, self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn and_d(&mut self) {
        self.registers.a = alu::and_a_n(self.registers.a, self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn and_e(&mut self) {
        self.registers.a = alu::and_a_n(self.registers.a, self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn and_h(&mut self) {
        self.registers.a = alu::and_a_n(self.registers.a, self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn and_l(&mut self) {
        self.registers.a = alu::and_a_n(self.registers.a, self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn and_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        self.registers.a = alu::and_a_n(self.registers.a, hl, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn and_a(&mut self) {
        self.registers.a = alu::and_a_n(self.registers.a, self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn xor_b(&mut self) {
        self.registers.a = alu::xor_a_n(self.registers.a, self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn xor_c(&mut self) {
        self.registers.a = alu::xor_a_n(self.registers.a, self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn xor_d(&mut self) {
        self.registers.a = alu::xor_a_n(self.registers.a, self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn xor_e(&mut self) {
        self.registers.a = alu::xor_a_n(self.registers.a, self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn xor_h(&mut self) {
        self.registers.a = alu::xor_a_n(self.registers.a, self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn xor_l(&mut self) {
        self.registers.a = alu::xor_a_n(self.registers.a, self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn xor_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        self.registers.a = alu::xor_a_n(self.registers.a, hl, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn xor_a(&mut self) {
        self.registers.a = alu::xor_a_n(self.registers.a, self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn or_b(&mut self) {
        self.registers.a = alu::or_a_n(self.registers.a, self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn or_c(&mut self) {
        self.registers.a = alu::or_a_n(self.registers.a, self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn or_d(&mut self) {
        self.registers.a = alu::or_a_n(self.registers.a, self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn or_e(&mut self) {
        self.registers.a = alu::or_a_n(self.registers.a, self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn or_h(&mut self) {
        self.registers.a = alu::or_a_n(self.registers.a, self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn or_l(&mut self) {
        self.registers.a = alu::or_a_n(self.registers.a, self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn or_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        self.registers.a = alu::or_a_n(self.registers.a, hl, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn or_a(&mut self) {
        self.registers.a = alu::or_a_n(self.registers.a, self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn cp_b(&mut self) {
        alu::cp_a_n(self.registers.a, self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn cp_c(&mut self) {
        alu::cp_a_n(self.registers.a, self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn cp_d(&mut self) {
        alu::cp_a_n(self.registers.a, self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn cp_e(&mut self) {
        alu::cp_a_n(self.registers.a, self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn cp_h(&mut self) {
        alu::cp_a_n(self.registers.a, self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn cp_l(&mut self) {
        alu::cp_a_n(self.registers.a, self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn cp_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::cp_a_n(self.registers.a, hl, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn cp_a(&mut self) {
        alu::cp_a_n(self.registers.a, self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn ret_nz(&mut self, memory: &Memory) {
        if !self.registers.f.contains(Flag::ZERO) {
            function::ret(&mut self.registers, memory);
            self.instruction_cycle = 3;
        } else {
            self.instruction_cycle = 1;
        }
    }

    fn pop_bc(&mut self, memory: &Memory) {
        let bc = stack::pop(&mut self.registers.sp, memory);
        self.registers.set_bc(bc);
        self.instruction_cycle = 3;
    }

    fn jp_nz_nn(&mut self, nn: u16) {
        if !self.registers.f.contains(Flag::ZERO) {
            self.registers.pc = nn;
            self.instruction_cycle = 4;
        } else {
            self.instruction_cycle = 3;
        }
    }

    fn jp_nn(&mut self, nn: u16) {
        self.registers.pc = nn;
        self.instruction_cycle = 4;
    }

    fn call_nz_nn(&mut self, memory: &mut Memory, nn: u16) {
        if !self.registers.f.contains(Flag::ZERO) {
            function::call_nn(&mut self.registers, nn, memory);
            self.instruction_cycle = 5;
        } else {
            self.instruction_cycle = 3;
        }
    }

    fn push_bc(&mut self, memory: &mut Memory) {
        let bc = self.registers.get_bc();
        stack::push(&mut self.registers.sp, bc, memory);
        self.instruction_cycle = 3;
    }

    fn add_a_n(&mut self, n: u8) {
        self.registers.a = alu::add_a_n(self.registers.a, n, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rst_0(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x0, memory);
        self.instruction_cycle = 3;
    }

    fn ret_z(&mut self, memory: &Memory) {
        if self.registers.f.contains(Flag::ZERO) {
            function::ret(&mut self.registers, memory);
            self.instruction_cycle = 3;
        } else {
            self.instruction_cycle = 1;
        }
    }

    fn ret(&mut self, memory: &Memory) {
        function::ret(&mut self.registers, memory);
        self.instruction_cycle = 3;
    }

    fn jp_z_nn(&mut self, nn: u16) {
        if self.registers.f.contains(Flag::ZERO) {
            self.registers.pc = nn;
            self.instruction_cycle = 4;
        } else {
            self.instruction_cycle = 3;
        }
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
            0x07 => self.rlc_a_cb(),
            0x08 => self.rrc_b(),
            0x09 => self.rrc_c(),
            0x0A => self.rrc_d(),
            0x0B => self.rrc_e(),
            0x0C => self.rrc_h(),
            0x0D => self.rrc_l(),
            0x0E => self.rrc_hl(memory),
            0x0F => self.rrc_a_cb(),
            0x10 => self.rl_b(),
            0x11 => self.rl_c(),
            0x12 => self.rl_d(),
            0x13 => self.rl_e(),
            0x14 => self.rl_h(),
            0x15 => self.rl_l(),
            0x16 => self.rl_hl(memory),
            0x17 => self.rl_a_cb(),
            0x18 => self.rr_b(),
            0x19 => self.rr_c(),
            0x1A => self.rr_d(),
            0x1B => self.rr_e(),
            0x1C => self.rr_h(),
            0x1D => self.rr_l(),
            0x1E => self.rr_hl(memory),
            0x1F => self.rr_a_cb(),
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
        if self.registers.f.contains(Flag::ZERO) {
            function::call_nn(&mut self.registers, nn, memory);
            self.instruction_cycle = 5;
        } else {
            self.instruction_cycle = 3;
        }
    }

    fn call_nn(&mut self, nn: u16, memory: &mut Memory) {
        function::call_nn(&mut self.registers, nn, memory);
        self.instruction_cycle = 5;
    }

    fn adc_a_n(&mut self, n: u8) {
        self.registers.a = alu::adc_a_n(self.registers.a, n, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rst_8(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x08, memory);
        self.instruction_cycle = 3;
    }

    fn ret_nc(&mut self, memory: &Memory) {
        if !self.registers.f.contains(Flag::FULL_CARRY) {
            function::ret(&mut self.registers, memory);
            self.instruction_cycle = 3;
        } else {
            self.instruction_cycle = 1;
        }
    }

    fn pop_de(&mut self, memory: &Memory) {
        let de = stack::pop(&mut self.registers.sp, memory);
        self.registers.set_de(de);
        self.instruction_cycle = 3;
    }

    fn jp_nc_nn(&mut self, nn: u16) {
        if !self.registers.f.contains(Flag::FULL_CARRY) {
            self.registers.pc = nn;
            self.instruction_cycle = 4;
        } else {
            self.instruction_cycle = 3;
        }
    }

    fn undefined(&mut self) {
        panic!("Undefined Opcode!");
    }

    fn call_nc_nn(&mut self, nn: u16, memory: &mut Memory) {
        if !self.registers.f.contains(Flag::FULL_CARRY) {
            function::call_nn(&mut self.registers, nn, memory);
            self.instruction_cycle = 5;
        } else {
            self.instruction_cycle = 3;
        }
    }

    fn push_de(&mut self, memory: &mut Memory) {
        let de = self.registers.get_de();
        stack::push(&mut self.registers.sp, de, memory);
        self.instruction_cycle = 4;
    }

    fn sub_a_n(&mut self, n: u8) {
        self.registers.a = alu::sub_a_n(self.registers.a, n, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rst_10(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x10, memory);
        self.instruction_cycle = 3;
    }

    fn ret_c(&mut self, memory: &Memory) {
        if self.registers.f.contains(Flag::FULL_CARRY) {
            function::ret(&mut self.registers, memory);
            self.instruction_cycle = 3;
        } else {
            self.instruction_cycle = 1;
        }
    }

    fn ret_i(&mut self, memory: &Memory) {
        self.ret(memory);
        self.interrupt_enabled = true;
        self.instruction_cycle = 3;
    }

    fn jp_c_nn(&mut self, nn: u16) {
        if self.registers.f.contains(Flag::FULL_CARRY) {
            self.registers.pc = nn;
            self.instruction_cycle = 4;
        } else {
            self.instruction_cycle = 3;
        }
    }

    fn call_c_nn(&mut self, nn: u16, memory: &mut Memory) {
        if self.registers.f.contains(Flag::FULL_CARRY) {
            function::call_nn(&mut self.registers, nn, memory);
            self.instruction_cycle = 5;
        } else {
            self.instruction_cycle = 3;
        }
    }

    fn sbc_a_n(&mut self, n: u8) {
        self.registers.a = alu::sbc_a_n(self.registers.a, n, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rst_18(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x18, memory);
        self.instruction_cycle = 3;
    }

    fn ldh_n_a(&mut self, n: u8, memory: &mut Memory) {
        let index = 0xFF00 + (n as u16);
        memory.write_byte(index, self.registers.a);
        self.instruction_cycle = 3;
    }

    fn pop_hl(&mut self, memory: &Memory) {
        let hl = stack::pop(&mut self.registers.sp, memory);
        self.registers.set_hl(hl);
        self.instruction_cycle = 3;
    }

    fn ldh_c_a(&mut self, memory: &mut Memory) {
        let index = 0xFF00 + (self.registers.c as u16);
        memory.write_byte(index, self.registers.a);
        self.instruction_cycle = 2;
    }

    fn push_hl(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        stack::push(&mut self.registers.sp, hl, memory);
        self.instruction_cycle = 4;
    }

    fn and_n(&mut self, n: u8) {
        self.registers.a = alu::and_a_n(self.registers.a, n, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rst_20(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x20, memory);
        self.instruction_cycle = 3;
    }

    fn add_sp_d(&mut self, d: u8) {
        self.registers.sp = alu::add_sp_e(self.registers.sp, d, &mut self.registers.f);
        self.instruction_cycle = 4;

    }

    fn jp_hl(&mut self, memory: &Memory) {
        let hl = self.registers.get_hl();
        self.registers.pc = memory.read_word(hl);
        self.instruction_cycle = 1;
    }

    fn ld_nn_a(&mut self, nn: u16, memory: &mut Memory) {
        memory.write_byte(nn, self.registers.a);
        self.instruction_cycle = 4;
    }

    fn xor_n(&mut self, n: u8) {
        self.registers.a = alu::xor_a_n(self.registers.a, n, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rst_28(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x28, memory);
        self.instruction_cycle = 3;
    }

    fn ldh_a_n(&mut self, n: u8, memory: &Memory) {
        self.registers.a = memory.read_byte(0xFF00 + (n as u16));
        self.instruction_cycle = 3;
    }

    fn pop_af(&mut self, memory: &Memory) {
        let af = stack::pop(&mut self.registers.sp, memory);
        self.registers.set_af(af);
        self.instruction_cycle = 3;
    }

    fn di(&mut self) {
        self.interrupt_enabled = false;
        self.instruction_cycle = 1;
    }

    fn push_af(&mut self, memory: &mut Memory) {
        let af = self.registers.get_af();
        stack::push(&mut self.registers.sp, af, memory);
        self.instruction_cycle = 3;
    }

    fn or_n(&mut self, n: u8) {
        self.registers.a = alu::or_a_n(self.registers.a, n, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rst_30(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x30, memory);
        self.instruction_cycle = 3;
    }

    fn ldhl_sp_d(&mut self, d: u8, memory: &Memory) {
        let sp = alu::add_sp_e(self.registers.sp, d, &mut self.registers.f);
        let hl = memory.read_word(sp);
        self.registers.set_hl(hl);
        self.instruction_cycle = 3;
    }

    fn ld_sp_hl(&mut self) {
        self.registers.sp = self.registers.get_hl();
        self.instruction_cycle = 1;
    }

    fn ld_a_nn(&mut self, nn: u16, memory: &Memory) {
        self.registers.a = memory.read_byte(nn);
        self.instruction_cycle = 4;
    }

    fn ei(&mut self) {
        self.interrupt_enabled = true;
        self.instruction_cycle = 1;
    }

    fn cp_n(&mut self, n: u8) {
        alu::cp_a_n(self.registers.a, n, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rst_38(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x38, memory);
        self.instruction_cycle = 3;
    }

    //extended opcodes

    fn rlc_b(&mut self) {
        self.registers.b = alu::rlc_n(self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rlc_c(&mut self) {
        self.registers.c = alu::rlc_n(self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rlc_d(&mut self) {
        self.registers.d = alu::rlc_n(self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rlc_e(&mut self) {
        self.registers.e = alu::rlc_n(self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rlc_h(&mut self) {
        self.registers.h = alu::rlc_n(self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rlc_l(&mut self) {
        self.registers.l = alu::rlc_n(self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rlc_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::rlc_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn rlc_a_cb(&mut self) {
        self.registers.a = alu::rlc_n(self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rrc_b(&mut self) {
        self.registers.b = alu::rrc_n(self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rrc_c(&mut self) {
        self.registers.c = alu::rrc_n(self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rrc_d(&mut self) {
        self.registers.d = alu::rrc_n(self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rrc_e(&mut self) {
        self.registers.e = alu::rrc_n(self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rrc_h(&mut self) {
        self.registers.h = alu::rrc_n(self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rrc_l(&mut self) {
        self.registers.l = alu::rrc_n(self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rrc_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::rrc_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn rrc_a_cb(&mut self) {
        self.registers.a = alu::rrc_n(self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rl_b(&mut self) {
        self.registers.b = alu::rl_n(self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rl_c(&mut self) {
        self.registers.c = alu::rl_n(self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rl_d(&mut self) {
        self.registers.d = alu::rl_n(self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rl_e(&mut self) {
        self.registers.e = alu::rl_n(self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rl_h(&mut self) {
        self.registers.h = alu::rl_n(self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rl_l(&mut self) {
        self.registers.l = alu::rl_n(self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rl_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::rl_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn rl_a_cb(&mut self) {
        self.registers.a = alu::rl_n(self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rr_b(&mut self) {
        self.registers.b = alu::rr_n(self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rr_c(&mut self) {
        self.registers.c = alu::rr_n(self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rr_d(&mut self) {
        self.registers.d = alu::rr_n(self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rr_e(&mut self) {
        self.registers.e = alu::rr_n(self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rr_h(&mut self) {
        self.registers.h = alu::rr_n(self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rr_l(&mut self) {
        self.registers.l = alu::rr_n(self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn rr_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::rr_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn rr_a_cb(&mut self) {
        self.registers.a = alu::rr_n(self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sla_b(&mut self) {
        self.registers.b = alu::sla_n(self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sla_c(&mut self) {
        self.registers.c = alu::sla_n(self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sla_d(&mut self) {
        self.registers.d = alu::sla_n(self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sla_e(&mut self) {
        self.registers.e = alu::sla_n(self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sla_h(&mut self) {
        self.registers.h = alu::sla_n(self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sla_l(&mut self) {
        self.registers.l = alu::sla_n(self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sla_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::sla_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn sla_a(&mut self) {
        self.registers.a = alu::sla_n(self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sra_b(&mut self) {
        self.registers.b = alu::sra_n(self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sra_c(&mut self) {
        self.registers.c = alu::sra_n(self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sra_d(&mut self) {
        self.registers.d = alu::sra_n(self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sra_e(&mut self) {
        self.registers.e = alu::sra_n(self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sra_h(&mut self) {
        self.registers.h = alu::sra_n(self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sra_l(&mut self) {
        self.registers.l = alu::sra_n(self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn sra_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::sra_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn sra_a(&mut self) {
        self.registers.a = alu::sra_n(self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn swap_b(&mut self) {
        self.registers.b = alu::swap_n(self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn swap_c(&mut self) {
        self.registers.c = alu::swap_n(self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn swap_d(&mut self) {
        self.registers.h = alu::swap_n(self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn swap_e(&mut self) {
        self.registers.e = alu::swap_n(self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn swap_h(&mut self) {
        self.registers.h = alu::swap_n(self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn swap_l(&mut self) {
        self.registers.l = alu::swap_n(self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn swap_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::swap_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 2;
    }

    fn swap_a(&mut self) {
        self.registers.a = alu::swap_n(self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 1;
    }

    fn srl_b(&mut self) {
        self.registers.b = alu::srl_n(self.registers.b, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn srl_c(&mut self) {
        self.registers.c = alu::srl_n(self.registers.c, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn srl_d(&mut self) {
        self.registers.d = alu::srl_n(self.registers.d, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn srl_e(&mut self) {
        self.registers.e = alu::srl_n(self.registers.e, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn srl_h(&mut self) {
        self.registers.h = alu::srl_n(self.registers.h, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn srl_l(&mut self) {
        self.registers.l = alu::srl_n(self.registers.l, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn srl_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::srl_n(hl, &mut self.registers.f);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn srl_a(&mut self) {
        self.registers.a = alu::srl_n(self.registers.a, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_0_b(&mut self) {
        alu::bit_n_i(self.registers.b, 0, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_0_c(&mut self) {
        alu::bit_n_i(self.registers.c, 0, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_0_d(&mut self) {
        alu::bit_n_i(self.registers.d, 0, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_0_e(&mut self) {
        alu::bit_n_i(self.registers.e, 0, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_0_h(&mut self) {
        alu::bit_n_i(self.registers.h, 0, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_0_l(&mut self) {
        alu::bit_n_i(self.registers.l, 0, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_0_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::bit_n_i(hl, 0, &mut self.registers.f);
        self.instruction_cycle = 3;
    }

    fn bit_0_a(&mut self) {
        alu::bit_n_i(self.registers.a, 0, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_1_b(&mut self) {
        alu::bit_n_i(self.registers.b, 1, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_1_c(&mut self) {
        alu::bit_n_i(self.registers.c, 1, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_1_d(&mut self) {
        alu::bit_n_i(self.registers.d, 1, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_1_e(&mut self) {
        alu::bit_n_i(self.registers.e, 1, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_1_h(&mut self) {
        alu::bit_n_i(self.registers.h, 1, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_1_l(&mut self) {
        alu::bit_n_i(self.registers.l, 1, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_1_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::bit_n_i(hl, 0, &mut self.registers.f);
        self.instruction_cycle = 3;
    }

    fn bit_1_a(&mut self) {
        alu::bit_n_i(self.registers.a, 1, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_2_b(&mut self) {
        alu::bit_n_i(self.registers.b, 2, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_2_c(&mut self) {
        alu::bit_n_i(self.registers.c, 2, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_2_d(&mut self) {
        alu::bit_n_i(self.registers.d, 2, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_2_e(&mut self) {
        alu::bit_n_i(self.registers.e, 2, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_2_h(&mut self) {
        alu::bit_n_i(self.registers.h, 2, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_2_l(&mut self) {
        alu::bit_n_i(self.registers.l, 2, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_2_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::bit_n_i(hl, 2, &mut self.registers.f);
        self.instruction_cycle = 3;
    }

    fn bit_2_a(&mut self) {
        alu::bit_n_i(self.registers.a, 2, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_3_b(&mut self) {
        alu::bit_n_i(self.registers.b, 3, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_3_c(&mut self) {
        alu::bit_n_i(self.registers.c, 3, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_3_d(&mut self) {
        alu::bit_n_i(self.registers.d, 3, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_3_e(&mut self) {
        alu::bit_n_i(self.registers.e, 3, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_3_h(&mut self) {
        alu::bit_n_i(self.registers.h, 3, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_3_l(&mut self) {
        alu::bit_n_i(self.registers.l, 3, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_3_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::bit_n_i(hl, 3, &mut self.registers.f);
        self.instruction_cycle = 3;
    }

    fn bit_3_a(&mut self) {
        alu::bit_n_i(self.registers.a, 3, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_4_b(&mut self) {
        alu::bit_n_i(self.registers.b, 4, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_4_c(&mut self) {
        alu::bit_n_i(self.registers.c, 4, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_4_d(&mut self) {
        alu::bit_n_i(self.registers.d, 4, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_4_e(&mut self) {
        alu::bit_n_i(self.registers.e, 4, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_4_h(&mut self) {
        alu::bit_n_i(self.registers.h, 4, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_4_l(&mut self) {
        alu::bit_n_i(self.registers.l, 4, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_4_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::bit_n_i(hl, 4, &mut self.registers.f);
        self.instruction_cycle = 3;
    }

    fn bit_4_a(&mut self) {
        alu::bit_n_i(self.registers.a, 4, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_5_b(&mut self) {
        alu::bit_n_i(self.registers.b, 5, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_5_c(&mut self) {
        alu::bit_n_i(self.registers.c, 5, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_5_d(&mut self) {
        alu::bit_n_i(self.registers.d, 5, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_5_e(&mut self) {
        alu::bit_n_i(self.registers.e, 5, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_5_h(&mut self) {
        alu::bit_n_i(self.registers.h, 5, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_5_l(&mut self) {
        alu::bit_n_i(self.registers.l, 5, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_5_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::bit_n_i(hl, 5, &mut self.registers.f);
        self.instruction_cycle = 3;
    }

    fn bit_5_a(&mut self) {
        alu::bit_n_i(self.registers.a, 5, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_6_b(&mut self) {
        alu::bit_n_i(self.registers.b, 6, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_6_c(&mut self) {
        alu::bit_n_i(self.registers.c, 6, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_6_d(&mut self) {
        alu::bit_n_i(self.registers.d, 6, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_6_e(&mut self) {
        alu::bit_n_i(self.registers.e, 6, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_6_h(&mut self) {
        alu::bit_n_i(self.registers.h, 6, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_6_l(&mut self) {
        alu::bit_n_i(self.registers.l, 6, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_6_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::bit_n_i(hl, 6, &mut self.registers.f);
        self.instruction_cycle = 3;
    }

    fn bit_6_a(&mut self) {
        alu::bit_n_i(self.registers.a, 6, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_7_b(&mut self) {
        alu::bit_n_i(self.registers.b, 7, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_7_c(&mut self) {
        alu::bit_n_i(self.registers.c, 7, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_7_d(&mut self) {
        alu::bit_n_i(self.registers.d, 7, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_7_e(&mut self) {
        alu::bit_n_i(self.registers.e, 7, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_7_h(&mut self) {
        alu::bit_n_i(self.registers.h, 7, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_7_l(&mut self) {
        alu::bit_n_i(self.registers.l, 7, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn bit_7_hl(&mut self, memory: &Memory) {
        let hl = memory.read_byte(self.registers.get_hl());
        alu::bit_n_i(hl, 7, &mut self.registers.f);
        self.instruction_cycle = 3;
    }

    fn bit_7_a(&mut self) {
        alu::bit_n_i(self.registers.a, 7, &mut self.registers.f);
        self.instruction_cycle = 2;
    }

    fn res_0_b(&mut self) {
        self.registers.b = alu::res_n_i(self.registers.b, 0);
        self.instruction_cycle = 2;
    }

    fn res_0_c(&mut self) {
        self.registers.c = alu::res_n_i(self.registers.c, 0);
        self.instruction_cycle = 2;
    }

    fn res_0_d(&mut self) {
        self.registers.d = alu::res_n_i(self.registers.d, 0);
        self.instruction_cycle = 2;
    }

    fn res_0_e(&mut self) {
        self.registers.e = alu::res_n_i(self.registers.e, 0);
        self.instruction_cycle = 2;
    }

    fn res_0_h(&mut self) {
        self.registers.h = alu::res_n_i(self.registers.h, 0);
        self.instruction_cycle = 2;
    }

    fn res_0_l(&mut self) {
        self.registers.l = alu::res_n_i(self.registers.l, 0);
        self.instruction_cycle = 2;
    }

    fn res_0_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::res_n_i(hl, 0);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn res_0_a(&mut self) {
        self.registers.a = alu::res_n_i(self.registers.a, 0);
        self.instruction_cycle = 2;
    }

    fn res_1_b(&mut self) {
        self.registers.b = alu::res_n_i(self.registers.b, 1);
        self.instruction_cycle = 2;
    }

    fn res_1_c(&mut self) {
        self.registers.c = alu::res_n_i(self.registers.c, 1);
        self.instruction_cycle = 2;
    }

    fn res_1_d(&mut self) {
        self.registers.d = alu::res_n_i(self.registers.d, 1);
        self.instruction_cycle = 2;
    }

    fn res_1_e(&mut self) {
        self.registers.e = alu::res_n_i(self.registers.e, 1);
        self.instruction_cycle = 2;
    }

    fn res_1_h(&mut self) {
        self.registers.h = alu::res_n_i(self.registers.h, 1);
        self.instruction_cycle = 2;
    }

    fn res_1_l(&mut self) {
        self.registers.l = alu::res_n_i(self.registers.l, 1);
        self.instruction_cycle = 2;
    }

    fn res_1_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::res_n_i(hl, 1);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn res_1_a(&mut self) {
        self.registers.a = alu::res_n_i(self.registers.a, 1);
        self.instruction_cycle = 2;
    }

    fn res_2_b(&mut self) {
        self.registers.b = alu::res_n_i(self.registers.b, 2);
        self.instruction_cycle = 2;
    }

    fn res_2_c(&mut self) {
        self.registers.c = alu::res_n_i(self.registers.c, 2);
        self.instruction_cycle = 2;
    }

    fn res_2_d(&mut self) {
        self.registers.d = alu::res_n_i(self.registers.d, 2);
        self.instruction_cycle = 2;
    }

    fn res_2_e(&mut self) {
        self.registers.e = alu::res_n_i(self.registers.e, 2);
        self.instruction_cycle = 2;
    }

    fn res_2_h(&mut self) {
        self.registers.h = alu::res_n_i(self.registers.h, 2);
        self.instruction_cycle = 2;
    }

    fn res_2_l(&mut self) {
        self.registers.l = alu::res_n_i(self.registers.l, 2);
        self.instruction_cycle = 2;
    }

    fn res_2_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::res_n_i(hl, 2);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn res_2_a(&mut self) {
        self.registers.a = alu::res_n_i(self.registers.a, 2);
        self.instruction_cycle = 2;
    }

    fn res_3_b(&mut self) {
        self.registers.b = alu::res_n_i(self.registers.b, 3);
        self.instruction_cycle = 2;
    }

    fn res_3_c(&mut self) {
        self.registers.c = alu::res_n_i(self.registers.c, 3);
        self.instruction_cycle = 2;
    }

    fn res_3_d(&mut self) {
        self.registers.d = alu::res_n_i(self.registers.d, 3);
        self.instruction_cycle = 2;
    }

    fn res_3_e(&mut self) {
        self.registers.e = alu::res_n_i(self.registers.e, 3);
        self.instruction_cycle = 2;
    }

    fn res_3_h(&mut self) {
        self.registers.h = alu::res_n_i(self.registers.h, 3);
        self.instruction_cycle = 2;
    }

    fn res_3_l(&mut self) {
        self.registers.l = alu::res_n_i(self.registers.l, 3);
        self.instruction_cycle = 2;
    }

    fn res_3_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::res_n_i(hl, 3);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn res_3_a(&mut self) {
        self.registers.a = alu::res_n_i(self.registers.a, 3);
        self.instruction_cycle = 2;
    }

    fn res_4_b(&mut self) {
        self.registers.b = alu::res_n_i(self.registers.b, 4);
        self.instruction_cycle = 2;
    }

    fn res_4_c(&mut self) {
        self.registers.c = alu::res_n_i(self.registers.c, 4);
        self.instruction_cycle = 2;
    }

    fn res_4_d(&mut self) {
        self.registers.d = alu::res_n_i(self.registers.d, 4);
        self.instruction_cycle = 2;
    }

    fn res_4_e(&mut self) {
        self.registers.e = alu::res_n_i(self.registers.e, 4);
        self.instruction_cycle = 2;
    }

    fn res_4_h(&mut self) {
        self.registers.h = alu::res_n_i(self.registers.h, 4);
        self.instruction_cycle = 2;
    }

    fn res_4_l(&mut self) {
        self.registers.l = alu::res_n_i(self.registers.l, 4);
        self.instruction_cycle = 2;
    }

    fn res_4_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::res_n_i(hl, 4);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn res_4_a(&mut self) {
        self.registers.a = alu::res_n_i(self.registers.a, 4);
        self.instruction_cycle = 2;
    }

    fn res_5_b(&mut self) {
        self.registers.b = alu::res_n_i(self.registers.b, 5);
        self.instruction_cycle = 2;
    }

    fn res_5_c(&mut self) {
        self.registers.c = alu::res_n_i(self.registers.c, 5);
        self.instruction_cycle = 2;
    }

    fn res_5_d(&mut self) {
        self.registers.d = alu::res_n_i(self.registers.d, 5);
        self.instruction_cycle = 2;
    }

    fn res_5_e(&mut self) {
        self.registers.e = alu::res_n_i(self.registers.e, 5);
        self.instruction_cycle = 2;
    }

    fn res_5_h(&mut self) {
        self.registers.h = alu::res_n_i(self.registers.h, 5);
        self.instruction_cycle = 2;
    }

    fn res_5_l(&mut self) {
        self.registers.l = alu::res_n_i(self.registers.l, 5);
        self.instruction_cycle = 2;
    }

    fn res_5_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::res_n_i(hl, 5);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn res_5_a(&mut self) {
        self.registers.a = alu::res_n_i(self.registers.a, 5);
        self.instruction_cycle = 2;
    }

    fn res_6_b(&mut self) {
        self.registers.b = alu::res_n_i(self.registers.b, 6);
        self.instruction_cycle = 2;
    }

    fn res_6_c(&mut self) {
        self.registers.c = alu::res_n_i(self.registers.c, 6);
        self.instruction_cycle = 2;
    }

    fn res_6_d(&mut self) {
        self.registers.d = alu::res_n_i(self.registers.d, 6);
        self.instruction_cycle = 2;
    }

    fn res_6_e(&mut self) {
        self.registers.e = alu::res_n_i(self.registers.e, 6);
        self.instruction_cycle = 2;
    }

    fn res_6_h(&mut self) {
        self.registers.h = alu::res_n_i(self.registers.h, 6);
        self.instruction_cycle = 2;
    }

    fn res_6_l(&mut self) {
        self.registers.l = alu::res_n_i(self.registers.l, 6);
        self.instruction_cycle = 2;
    }

    fn res_6_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::res_n_i(hl, 6);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn res_6_a(&mut self) {
        self.registers.a = alu::res_n_i(self.registers.a, 6);
        self.instruction_cycle = 2;
    }

    fn res_7_b(&mut self) {
        self.registers.b = alu::res_n_i(self.registers.b, 7);
        self.instruction_cycle = 2;
    }

    fn res_7_c(&mut self) {
        self.registers.c = alu::res_n_i(self.registers.c, 7);
        self.instruction_cycle = 2;
    }

    fn res_7_d(&mut self) {
        self.registers.d = alu::res_n_i(self.registers.d, 7);
        self.instruction_cycle = 2;
    }

    fn res_7_e(&mut self) {
        self.registers.e = alu::res_n_i(self.registers.e, 7);
        self.instruction_cycle = 2;
    }

    fn res_7_h(&mut self) {
        self.registers.h = alu::res_n_i(self.registers.h, 7);
        self.instruction_cycle = 2;
    }

    fn res_7_l(&mut self) {
        self.registers.l = alu::res_n_i(self.registers.l, 7);
        self.instruction_cycle = 2;
    }

    fn res_7_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::res_n_i(hl, 7);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn res_7_a(&mut self) {
        self.registers.a = alu::res_n_i(self.registers.a, 7);
        self.instruction_cycle = 2;
    }

    fn set_0_b(&mut self) {
        self.registers.b = alu::set_n_i(self.registers.b, 0);
        self.instruction_cycle = 2;
    }

    fn set_0_c(&mut self) {
        self.registers.c = alu::set_n_i(self.registers.c, 0);
        self.instruction_cycle = 2;
    }

    fn set_0_d(&mut self) {
        self.registers.d = alu::set_n_i(self.registers.d, 0);
        self.instruction_cycle = 2;
    }

    fn set_0_e(&mut self) {
        self.registers.e = alu::set_n_i(self.registers.e, 0);
        self.instruction_cycle = 2;
    }

    fn set_0_h(&mut self) {
        self.registers.h = alu::set_n_i(self.registers.h, 0);
        self.instruction_cycle = 2;
    }

    fn set_0_l(&mut self) {
        self.registers.l = alu::set_n_i(self.registers.l, 0);
        self.instruction_cycle = 2;
    }

    fn set_0_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::set_n_i(hl, 0);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn set_0_a(&mut self) {
        self.registers.a = alu::set_n_i(self.registers.a, 0);
        self.instruction_cycle = 2;
    }

    fn set_1_b(&mut self) {
        self.registers.b = alu::set_n_i(self.registers.b, 1);
        self.instruction_cycle = 2;
    }

    fn set_1_c(&mut self) {
        self.registers.c = alu::set_n_i(self.registers.c, 1);
        self.instruction_cycle = 2;
    }

    fn set_1_d(&mut self) {
        self.registers.d = alu::set_n_i(self.registers.d, 1);
        self.instruction_cycle = 2;
    }

    fn set_1_e(&mut self) {
        self.registers.e = alu::set_n_i(self.registers.e, 1);
        self.instruction_cycle = 2;
    }

    fn set_1_h(&mut self) {
        self.registers.h = alu::set_n_i(self.registers.h, 1);
        self.instruction_cycle = 2;
    }

    fn set_1_l(&mut self) {
        self.registers.l = alu::set_n_i(self.registers.l, 1);
        self.instruction_cycle = 2;
    }

    fn set_1_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::set_n_i(hl, 1);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn set_1_a(&mut self) {
        self.registers.a = alu::set_n_i(self.registers.a, 1);
        self.instruction_cycle = 2;
    }

    fn set_2_b(&mut self) {
        self.registers.b = alu::set_n_i(self.registers.b, 1);
        self.instruction_cycle = 2;
    }

    fn set_2_c(&mut self) {
        self.registers.c = alu::set_n_i(self.registers.c, 1);
        self.instruction_cycle = 2;
    }

    fn set_2_d(&mut self) {
        self.registers.d = alu::set_n_i(self.registers.d, 2);
        self.instruction_cycle = 2;
    }

    fn set_2_e(&mut self) {
        self.registers.e = alu::set_n_i(self.registers.e, 2);
        self.instruction_cycle = 2;
    }

    fn set_2_h(&mut self) {
        self.registers.h = alu::set_n_i(self.registers.h, 2);
        self.instruction_cycle = 2;
    }

    fn set_2_l(&mut self) {
        self.registers.l = alu::set_n_i(self.registers.l, 2);
        self.instruction_cycle = 2;
    }

    fn set_2_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::set_n_i(hl, 2);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn set_2_a(&mut self) {
        self.registers.a = alu::set_n_i(self.registers.a, 2);
        self.instruction_cycle = 2;
    }

    fn set_3_b(&mut self) {
        self.registers.b = alu::set_n_i(self.registers.b, 3);
        self.instruction_cycle = 2;
    }

    fn set_3_c(&mut self) {
        self.registers.c = alu::set_n_i(self.registers.c, 3);
        self.instruction_cycle = 2;
    }

    fn set_3_d(&mut self) {
        self.registers.d = alu::set_n_i(self.registers.d, 3);
        self.instruction_cycle = 2;
    }

    fn set_3_e(&mut self) {
        self.registers.e = alu::set_n_i(self.registers.e, 3);
        self.instruction_cycle = 2;
    }

    fn set_3_h(&mut self) {
        self.registers.h = alu::set_n_i(self.registers.h, 3);
        self.instruction_cycle = 2;
    }

    fn set_3_l(&mut self) {
        self.registers.l = alu::set_n_i(self.registers.l, 3);
        self.instruction_cycle = 2;
    }

    fn set_3_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::set_n_i(hl, 3);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn set_3_a(&mut self) {
        self.registers.a = alu::set_n_i(self.registers.a, 3);
        self.instruction_cycle = 2;
    }

    fn set_4_b(&mut self) {
        self.registers.b = alu::set_n_i(self.registers.b, 4);
        self.instruction_cycle = 2;
    }

    fn set_4_c(&mut self) {
        self.registers.c = alu::set_n_i(self.registers.c, 4);
        self.instruction_cycle = 2;
    }

    fn set_4_d(&mut self) {
        self.registers.d = alu::set_n_i(self.registers.d, 4);
        self.instruction_cycle = 2;
    }

    fn set_4_e(&mut self) {
        self.registers.e = alu::set_n_i(self.registers.e, 4);
        self.instruction_cycle = 2;
    }

    fn set_4_h(&mut self) {
        self.registers.h = alu::set_n_i(self.registers.h, 4);
        self.instruction_cycle = 2;
    }

    fn set_4_l(&mut self) {
        self.registers.l = alu::set_n_i(self.registers.l, 4);
        self.instruction_cycle = 2;
    }

    fn set_4_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::set_n_i(hl, 4);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn set_4_a(&mut self) {
        self.registers.a = alu::set_n_i(self.registers.a, 4);
        self.instruction_cycle = 2;
    }

    fn set_5_b(&mut self) {
        self.registers.b = alu::set_n_i(self.registers.b, 5);
        self.instruction_cycle = 2;
    }

    fn set_5_c(&mut self) {
        self.registers.c = alu::set_n_i(self.registers.c, 5);
        self.instruction_cycle = 2;
    }

    fn set_5_d(&mut self) {
        self.registers.d = alu::set_n_i(self.registers.d, 5);
        self.instruction_cycle = 2;
    }

    fn set_5_e(&mut self) {
        self.registers.e = alu::set_n_i(self.registers.e, 5);
        self.instruction_cycle = 2;
    }

    fn set_5_h(&mut self) {
        self.registers.h = alu::set_n_i(self.registers.h, 5);
        self.instruction_cycle = 2;
    }

    fn set_5_l(&mut self) {
        self.registers.l = alu::set_n_i(self.registers.l, 5);
        self.instruction_cycle = 2;
    }

    fn set_5_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::set_n_i(hl, 5);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn set_5_a(&mut self) {
        self.registers.a = alu::set_n_i(self.registers.a, 5);
        self.instruction_cycle = 2;
    }

    fn set_6_b(&mut self) {
        self.registers.b = alu::set_n_i(self.registers.b, 6);
        self.instruction_cycle = 2;
    }

    fn set_6_c(&mut self) {
        self.registers.c = alu::set_n_i(self.registers.c, 6);
        self.instruction_cycle = 2;
    }

    fn set_6_d(&mut self) {
        self.registers.d = alu::set_n_i(self.registers.d, 6);
        self.instruction_cycle = 2;
    }

    fn set_6_e(&mut self) {
        self.registers.e = alu::set_n_i(self.registers.e, 6);
        self.instruction_cycle = 2;
    }

    fn set_6_h(&mut self) {
        self.registers.h = alu::set_n_i(self.registers.h, 6);
        self.instruction_cycle = 2;
    }

    fn set_6_l(&mut self) {
        self.registers.l = alu::set_n_i(self.registers.l, 6);
        self.instruction_cycle = 2;
    }

    fn set_6_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::set_n_i(hl, 6);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn set_6_a(&mut self) {
        self.registers.a = alu::set_n_i(self.registers.a, 6);
        self.instruction_cycle = 2;
    }

    fn set_7_b(&mut self) {
        self.registers.b = alu::set_n_i(self.registers.b, 7);
        self.instruction_cycle = 2;
    }

    fn set_7_c(&mut self) {
        self.registers.c = alu::set_n_i(self.registers.c, 7);
        self.instruction_cycle = 2;
    }

    fn set_7_d(&mut self) {
        self.registers.d = alu::set_n_i(self.registers.d, 7);
        self.instruction_cycle = 2;
    }

    fn set_7_e(&mut self) {
        self.registers.e = alu::set_n_i(self.registers.e, 7);
        self.instruction_cycle = 2;
    }

    fn set_7_h(&mut self) {
        self.registers.h = alu::set_n_i(self.registers.h, 7);
        self.instruction_cycle = 2;
    }

    fn set_7_l(&mut self) {
        self.registers.l = alu::set_n_i(self.registers.l, 7);
        self.instruction_cycle = 2;
    }

    fn set_7_hl(&mut self, memory: &mut Memory) {
        let mut hl = memory.read_byte(self.registers.get_hl());
        hl = alu::set_n_i(hl, 7);
        memory.write_byte(self.registers.get_hl(), hl);
        self.instruction_cycle = 4;
    }

    fn set_7_a(&mut self) {
        self.registers.a = alu::set_n_i(self.registers.a, 7);
        self.instruction_cycle = 2;
    }

    //interrupts

    pub fn rst_40(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x40, memory);
        self.instruction_cycle = 3;
    }

    pub fn rst_48(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x48, memory);
        self.instruction_cycle = 3;
    }

    pub fn rst_50(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x50, memory);
        self.instruction_cycle = 3;
    }

    pub fn rst_58(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x58, memory);
        self.instruction_cycle = 3;
    }

    pub fn rst_60(&mut self, memory: &mut Memory) {
        function::call_nn(&mut self.registers, 0x60, memory);
        self.instruction_cycle = 3;
    }
}

#[cfg(test)]
mod tests {
    //    use cpu::Cpu;
    //    use mmu::Memory;
    //
    //    #[test]
    //    fn test_opcode_0x01() {}
}
