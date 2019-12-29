mod registers;

use self::registers::flag::Flag;
use self::registers::Registers;
use bit_utils;
use mmu::Memory;

const INSTRUCTION_TIMINGS: [i32; 256] = [
    4, 12, 8, 8, 4, 4, 8, 4, 20, 8, 8, 8, 4, 4, 8, 4, 4, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4,
    8, 4, 8, 12, 8, 8, 4, 4, 8, 4, 8, 8, 8, 8, 4, 4, 8, 4, 8, 12, 8, 8, 12, 12, 12, 4, 8, 8, 8, 8,
    4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4,
    4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 8, 8, 8, 8, 8, 8, 4, 8, 4, 4, 4, 4,
    4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4,
    4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4,
    4, 4, 8, 4, 8, 12, 12, 16, 12, 16, 8, 16, 8, 16, 12, 4, 12, 24, 8, 16, 8, 12, 12, 0, 12, 16, 8,
    16, 8, 16, 12, 0, 12, 0, 8, 16, 12, 12, 8, 0, 0, 16, 8, 16, 16, 4, 16, 0, 0, 0, 8, 16, 12, 12,
    8, 4, 0, 16, 8, 16, 12, 8, 16, 4, 0, 0, 8, 16,
];

const CB_INSTRUCTION_TIMINGS: [i32; 256] = [
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8,
    16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8,
    8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8,
    8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8,
    16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8,
    8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8,
    8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
];

pub struct Cpu {
    registers: Registers,
    halted: bool,
    interrupt_enabled: bool,
    pending_enable_interrupts: i32,
    pending_disable_interrupts: i32,
    instruction_cycle: i32,
    is_cgb: bool,
    cgb_speed: bool,
}

impl Cpu {
    pub fn new(is_cgb: bool) -> Cpu {
        let mut registers: Registers = Default::default();
        if is_cgb {
            registers.set_af(0x11B0);
        } else {
            registers.set_af(0x01B0);
        }
        registers.set_bc(0x0013);
        registers.set_de(0x00D8);
        registers.set_hl(0x014D);
        registers.pc = 0x0100;
        registers.sp = 0xFFFE;

        Cpu {
            registers,
            halted: false,
            interrupt_enabled: false,
            pending_enable_interrupts: -1,
            pending_disable_interrupts: -1,
            instruction_cycle: 0,
            is_cgb,
            cgb_speed: false,
        }
    }

    pub fn unhalt(&mut self) {
        self.halted = false;
    }

    pub fn are_interrupts_enabled(&self) -> bool {
        self.interrupt_enabled
    }

    pub fn disable_interrupts(&mut self) {
        self.interrupt_enabled = false;
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
        self.instruction_cycle = 0;
        if !self.halted {
            if self.pending_enable_interrupts != -1 {
                let pending_enable_interrupts = self.pending_enable_interrupts;
                self.pending_enable_interrupts -= 1;
                if pending_enable_interrupts == 0 {
                    self.pending_enable_interrupts = -1;
                    self.interrupt_enabled = true;
                }
            }

            if self.pending_disable_interrupts != -1 {
                let pending_disable_interrupts = self.pending_disable_interrupts;
                self.pending_disable_interrupts -= 1;
                if pending_disable_interrupts == 0 {
                    self.pending_disable_interrupts = -1;
                    self.interrupt_enabled = false;
                }
            }

            let opcode = self.get_n(memory);
            self.instruction_cycle += INSTRUCTION_TIMINGS[opcode as usize];
            self.execute_opcode(opcode, memory);
        } else {
            self.instruction_cycle = 4;
        }

        if self.halted {
            self.instruction_cycle = 4;
        }
        if self.cgb_speed {
            self.instruction_cycle / 2
        } else {
            self.instruction_cycle
        }
    }

    fn execute_opcode(&mut self, opcode: u8, memory: &mut Memory) {
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
            0x10 => self.stop(memory),
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
            0x86 => self.add_hl(memory),
            0x87 => self.add_a_a(),
            0x88 => self.adc_a_b(),
            0x89 => self.adc_a_c(),
            0x8A => self.adc_a_d(),
            0x8B => self.adc_a_e(),
            0x8C => self.adc_a_h(),
            0x8D => self.adc_a_l(),
            0x8E => self.adc_hl(memory),
            0x8F => self.adc_a_a(),
            0x90 => self.sub_a_b(),
            0x91 => self.sub_a_c(),
            0x92 => self.sub_a_d(),
            0x93 => self.sub_a_e(),
            0x94 => self.sub_a_h(),
            0x95 => self.sub_a_l(),
            0x96 => self.sub_hl(memory),
            0x97 => self.sub_a_a(),
            0x98 => self.sbc_a_b(),
            0x99 => self.sbc_a_c(),
            0x9A => self.sbc_a_d(),
            0x9B => self.sbc_a_e(),
            0x9C => self.sbc_a_h(),
            0x9D => self.sbc_a_l(),
            0x9E => self.sbc_hl(memory),
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
            0xD3 => self.undefined(opcode),
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
            0xDB => self.undefined(opcode),
            0xDC => {
                let nn = self.get_nn(memory);
                self.call_c_nn(nn, memory);
            }
            0xDD => self.undefined(opcode),
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
            0xE2 => self.ldh_a_c(memory),
            0xE3 => self.undefined(opcode),
            0xE4 => self.undefined(opcode),
            0xE5 => self.push_hl(memory),
            0xE6 => {
                let n = self.get_n(memory);
                self.and_n(n);
            }
            0xE7 => self.rst_20(memory),
            0xE8 => {
                let n = self.get_n(memory);
                self.add_sp_s(n);
            }
            0xE9 => self.jp_hl(),
            0xEA => {
                let nn = self.get_nn(memory);
                self.ld_nn_a(nn, memory);
            }
            0xEB => self.undefined(opcode),
            0xEC => self.undefined(opcode),
            0xED => self.undefined(opcode),
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
            0xF2 => self.ldh_c_a(memory),
            0xF3 => self.di(),
            0xF4 => self.undefined(opcode),
            0xF5 => self.push_af(memory),
            0xF6 => {
                let n = self.get_n(memory);
                self.or_n(n);
            }
            0xF7 => self.rst_30(memory),
            0xF8 => {
                let n = self.get_n(memory);
                self.ld_hl_sp_e(n);
            }
            0xF9 => self.ld_sp_hl(),
            0xFA => {
                let n = self.get_nn(memory);
                self.ld_a_nn(n, memory);
            }
            0xFB => self.ei(),
            0xFC => self.undefined(opcode),
            0xFD => self.undefined(opcode),
            0xFE => {
                let n = self.get_n(memory);
                self.cp_n(n);
            }
            _ => self.rst_38(memory),
        }
    }

    //opcodes

    fn nop(&mut self) {}

    fn ld_bc_nn(&mut self, nn: u16) {
        let bc = self.ld_rr_nn(nn);
        self.registers.set_bc(bc);
    }

    fn ld_bc_a(&mut self, memory: &mut Memory) {
        let bc = self.registers.get_bc();
        self.ld_rr_r(bc, self.registers.a, memory);
    }

    fn inc_bc(&mut self) {
        let bc = self.inc_rr(self.registers.get_bc());
        self.registers.set_bc(bc);
    }

    fn inc_b(&mut self) {
        self.registers.b = self.inc_r(self.registers.b);
    }

    fn dec_b(&mut self) {
        self.registers.b = self.dec_r(self.registers.b);
    }

    fn ld_b_n(&mut self, n: u8) {
        self.registers.b = self.ld_r_n(n);
    }

    fn add_hl_bc(&mut self) {
        self.add_hl_rr(self.registers.get_bc());
    }

    fn ld_a_bc(&mut self, memory: &Memory) {
        self.registers.a = self.ld_r_rr(self.registers.get_bc(), memory);
    }

    fn dec_bc(&mut self) {
        let bc = self.dec_rr(self.registers.get_bc());
        self.registers.set_bc(bc);
    }

    fn inc_c(&mut self) {
        self.registers.c = self.inc_r(self.registers.c);
    }

    fn dec_c(&mut self) {
        self.registers.c = self.dec_r(self.registers.c);
    }

    fn ld_c_n(&mut self, n: u8) {
        self.registers.c = self.ld_r_n(n);
    }

    fn stop(&mut self, memory: &mut Memory) {
        if self.is_cgb {
            let current_key1 = memory.load(0xFF4D);

            if current_key1 & 1 == 1 {
                self.cgb_speed = !self.cgb_speed;

                if self.cgb_speed {
                    memory.store(0xFF4D, 0x80);
                } else {
                    memory.store(0xFF4D, 0x00);
                }
            }
        }
    }

    fn ld_de_nn(&mut self, nn: u16) {
        let de = self.ld_rr_nn(nn);
        self.registers.set_de(de);
    }

    fn ld_de_a(&mut self, memory: &mut Memory) {
        let de = self.registers.get_de();
        self.ld_rr_r(de, self.registers.a, memory);
    }

    fn inc_de(&mut self) {
        let de = self.inc_rr(self.registers.get_de());
        self.registers.set_de(de);
    }

    fn inc_d(&mut self) {
        self.registers.d = self.inc_r(self.registers.d);
    }

    fn dec_d(&mut self) {
        self.registers.d = self.dec_r(self.registers.d);
    }

    fn ld_d_n(&mut self, n: u8) {
        self.registers.d = self.ld_r_n(n);
    }

    fn jr_n(&mut self, n: u8) {
        self.registers.pc = self.registers.pc.wrapping_add(n as i8 as u16);
    }

    fn add_hl_de(&mut self) {
        self.add_hl_rr(self.registers.get_de());
    }

    fn ld_a_de(&mut self, memory: &Memory) {
        self.registers.a = self.ld_r_rr(self.registers.get_de(), memory);
    }

    fn dec_de(&mut self) {
        let de = self.dec_rr(self.registers.get_de());
        self.registers.set_de(de);
    }

    fn inc_e(&mut self) {
        self.registers.e = self.inc_r(self.registers.e);
    }

    fn dec_e(&mut self) {
        self.registers.e = self.dec_r(self.registers.e);
    }

    fn ld_e_n(&mut self, n: u8) {
        self.registers.e = self.ld_r_n(n);
    }

    fn jr_nz_n(&mut self, n: u8) {
        self.jr_cc_n(!self.registers.f.contains(Flag::ZERO), n);
    }

    fn ld_hl_nn(&mut self, nn: u16) {
        let hl = self.ld_rr_nn(nn);
        self.registers.set_hl(hl);
    }

    fn inc_h(&mut self) {
        self.registers.h = self.inc_r(self.registers.h);
    }

    fn dec_h(&mut self) {
        self.registers.h = self.dec_r(self.registers.h);
    }

    fn ld_h_n(&mut self, n: u8) {
        self.registers.h = self.ld_r_n(n);
    }

    fn daa(&mut self) {
        let mut result = i32::from(self.registers.a);

        if !self.registers.f.contains(Flag::NEGATIVE) {
            if self.registers.f.contains(Flag::HALF_CARRY) || ((result & 0xF) > 9) {
                result += 0x06;
            }

            if self.registers.f.contains(Flag::FULL_CARRY) || result > 0x9F {
                result += 0x60;
            }
        } else {
            if self.registers.f.contains(Flag::HALF_CARRY) {
                result = (result - 6) & 0xFF;
            }

            if self.registers.f.contains(Flag::FULL_CARRY) {
                result -= 0x60;
            }
        }

        self.registers.f.remove(Flag::HALF_CARRY);
        self.registers.f.remove(Flag::ZERO);

        if (result & 0x100) == 0x100 {
            self.registers.f.insert(Flag::FULL_CARRY);
        }

        result &= 0xFF;
        if result == 0 {
            self.registers.f.insert(Flag::ZERO);
        }

        self.registers.a = result as u8;
    }

    fn jr_z_n(&mut self, n: u8) {
        let cc = self.registers.f.contains(Flag::ZERO);
        self.jr_cc_n(cc, n);
    }

    fn add_hl_hl(&mut self) {
        let hl = self.registers.get_hl();
        self.add_hl_rr(hl);
    }

    fn inc_l(&mut self) {
        self.registers.l = self.inc_r(self.registers.l);
    }

    fn dec_l(&mut self) {
        self.registers.l = self.dec_r(self.registers.l);
    }

    fn ld_l_n(&mut self, n: u8) {
        self.registers.l = self.ld_r_n(n);
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
        self.registers.sp = self.ld_rr_nn(nn);
    }

    fn inc_sp(&mut self) {
        self.registers.sp = self.inc_rr(self.registers.sp);
    }

    fn scf(&mut self) {
        self.registers.f.insert(Flag::FULL_CARRY);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);
    }

    fn jr_c_n(&mut self, n: u8) {
        let cc = self.registers.f.contains(Flag::FULL_CARRY);
        self.jr_cc_n(cc, n);
    }

    fn add_hl_sp(&mut self) {
        self.add_hl_rr(self.registers.sp);
    }

    fn dec_sp(&mut self) {
        self.registers.sp = self.dec_rr(self.registers.sp);
    }

    fn inc_a(&mut self) {
        self.registers.a = self.inc_r(self.registers.a);
    }

    fn dec_a(&mut self) {
        self.registers.a = self.dec_r(self.registers.a);
    }

    fn ld_a_n(&mut self, n: u8) {
        self.registers.a = self.ld_r_n(n);
    }

    fn ccf(&mut self) {
        self.registers.f.toggle(Flag::FULL_CARRY);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);
    }

    fn ld_b_b(&mut self) {
        self.registers.b = self.ld_r_r(self.registers.b);
    }

    fn ld_b_c(&mut self) {
        self.registers.b = self.ld_r_r(self.registers.c);
    }

    fn ld_b_d(&mut self) {
        self.registers.b = self.ld_r_r(self.registers.d);
    }

    fn ld_b_e(&mut self) {
        self.registers.b = self.ld_r_r(self.registers.e);
    }

    fn ld_b_h(&mut self) {
        self.registers.b = self.ld_r_r(self.registers.h);
    }

    fn ld_b_l(&mut self) {
        self.registers.b = self.ld_r_r(self.registers.l);
    }

    fn ld_b_hl(&mut self, memory: &Memory) {
        self.registers.b = self.ld_r_hl(memory);
    }

    fn ld_b_a(&mut self) {
        self.registers.b = self.ld_r_r(self.registers.a);
    }

    fn ld_c_b(&mut self) {
        self.registers.c = self.ld_r_r(self.registers.b);
    }

    fn ld_c_c(&mut self) {
        self.registers.c = self.ld_r_r(self.registers.c);
    }

    fn ld_c_d(&mut self) {
        self.registers.c = self.ld_r_r(self.registers.d);
    }

    fn ld_c_e(&mut self) {
        self.registers.c = self.ld_r_r(self.registers.e);
    }

    fn ld_c_h(&mut self) {
        self.registers.c = self.ld_r_r(self.registers.h);
    }

    fn ld_c_l(&mut self) {
        self.registers.c = self.ld_r_r(self.registers.l);
    }

    fn ld_c_hl(&mut self, memory: &Memory) {
        self.registers.c = self.ld_r_hl(memory);
    }

    fn ld_c_a(&mut self) {
        self.registers.c = self.ld_r_r(self.registers.a);
    }

    fn ld_d_b(&mut self) {
        self.registers.d = self.ld_r_r(self.registers.b);
    }

    fn ld_d_c(&mut self) {
        self.registers.d = self.ld_r_r(self.registers.c);
    }

    fn ld_d_d(&mut self) {
        self.registers.d = self.ld_r_r(self.registers.d);
    }

    fn ld_d_e(&mut self) {
        self.registers.d = self.ld_r_r(self.registers.e);
    }

    fn ld_d_h(&mut self) {
        self.registers.d = self.ld_r_r(self.registers.h);
    }

    fn ld_d_l(&mut self) {
        self.registers.d = self.ld_r_r(self.registers.l);
    }

    fn ld_d_hl(&mut self, memory: &Memory) {
        self.registers.d = self.ld_r_hl(memory);
    }

    fn ld_d_a(&mut self) {
        self.registers.d = self.ld_r_r(self.registers.a);
    }

    fn ld_e_b(&mut self) {
        self.registers.e = self.ld_r_r(self.registers.b);
    }

    fn ld_e_c(&mut self) {
        self.registers.e = self.ld_r_r(self.registers.c);
    }

    fn ld_e_d(&mut self) {
        self.registers.e = self.ld_r_r(self.registers.d);
    }

    fn ld_e_e(&mut self) {
        self.registers.e = self.ld_r_r(self.registers.e);
    }

    fn ld_e_h(&mut self) {
        self.registers.e = self.ld_r_r(self.registers.h);
    }

    fn ld_e_l(&mut self) {
        self.registers.e = self.ld_r_r(self.registers.l);
    }

    fn ld_e_hl(&mut self, memory: &Memory) {
        self.registers.e = self.ld_r_hl(memory);
    }

    fn ld_e_a(&mut self) {
        self.registers.e = self.ld_r_r(self.registers.a);
    }

    fn ld_h_b(&mut self) {
        self.registers.h = self.ld_r_r(self.registers.b);
    }

    fn ld_h_c(&mut self) {
        self.registers.h = self.ld_r_r(self.registers.c);
    }

    fn ld_h_d(&mut self) {
        self.registers.h = self.ld_r_r(self.registers.d);
    }

    fn ld_h_e(&mut self) {
        self.registers.h = self.ld_r_r(self.registers.e);
    }

    fn ld_h_h(&mut self) {
        self.registers.h = self.ld_r_r(self.registers.h);
    }

    fn ld_h_l(&mut self) {
        self.registers.h = self.ld_r_r(self.registers.l);
    }

    fn ld_h_hl(&mut self, memory: &Memory) {
        self.registers.h = self.ld_r_hl(memory);
    }

    fn ld_h_a(&mut self) {
        self.registers.h = self.ld_r_r(self.registers.a);
    }

    fn ld_l_b(&mut self) {
        self.registers.l = self.ld_r_r(self.registers.b);
    }

    fn ld_l_c(&mut self) {
        self.registers.l = self.ld_r_r(self.registers.c);
    }

    fn ld_l_d(&mut self) {
        self.registers.l = self.ld_r_r(self.registers.d);
    }

    fn ld_l_e(&mut self) {
        self.registers.l = self.ld_r_r(self.registers.e);
    }

    fn ld_l_h(&mut self) {
        self.registers.l = self.ld_r_r(self.registers.h);
    }

    fn ld_l_l(&mut self) {
        self.registers.l = self.ld_r_r(self.registers.l);
    }

    fn ld_l_hl(&mut self, memory: &Memory) {
        self.registers.l = self.ld_r_hl(memory);
    }

    fn ld_l_a(&mut self) {
        self.registers.l = self.ld_r_r(self.registers.a);
    }

    fn ld_hl_b(&mut self, memory: &mut Memory) {
        self.ld_hl_r(self.registers.b, memory);
    }

    fn ld_hl_c(&mut self, memory: &mut Memory) {
        self.ld_hl_r(self.registers.c, memory);
    }

    fn ld_hl_d(&mut self, memory: &mut Memory) {
        self.ld_hl_r(self.registers.d, memory);
    }

    fn ld_hl_e(&mut self, memory: &mut Memory) {
        self.ld_hl_r(self.registers.e, memory);
    }

    fn ld_hl_h(&mut self, memory: &mut Memory) {
        self.ld_hl_r(self.registers.h, memory);
    }

    fn ld_hl_l(&mut self, memory: &mut Memory) {
        self.ld_hl_r(self.registers.l, memory);
    }

    fn halt(&mut self) {
        // if interrupt_enabled is about to be set, set it and repeat the halt instruction
        if self.pending_enable_interrupts != -1 {
            self.interrupt_enabled = true;
            self.pending_enable_interrupts = -1;
            self.registers.pc -= 1;
        } else if self.pending_disable_interrupts != -1 {
            self.interrupt_enabled = false;
            self.pending_enable_interrupts = -1;
            self.registers.pc -= 1;
        } else {
            self.halted = true;
        }
    }

    fn ld_hl_a(&mut self, memory: &mut Memory) {
        self.ld_hl_r(self.registers.a, memory);
    }

    fn ld_a_b(&mut self) {
        self.registers.a = self.ld_r_r(self.registers.b);
    }

    fn ld_a_c(&mut self) {
        self.registers.a = self.ld_r_r(self.registers.c);
    }

    fn ld_a_d(&mut self) {
        self.registers.a = self.ld_r_r(self.registers.d);
    }

    fn ld_a_e(&mut self) {
        self.registers.a = self.ld_r_r(self.registers.e);
    }

    fn ld_a_h(&mut self) {
        self.registers.a = self.ld_r_r(self.registers.h);
    }

    fn ld_a_l(&mut self) {
        self.registers.a = self.ld_r_r(self.registers.l);
    }

    fn ld_a_hl(&mut self, memory: &Memory) {
        self.registers.a = self.ld_r_hl(memory);
    }

    fn ld_a_a(&mut self) {
        self.registers.a = self.ld_r_r(self.registers.a);
    }

    fn add_a_b(&mut self) {
        self.add_r(self.registers.b);
    }

    fn add_a_c(&mut self) {
        self.add_r(self.registers.c);
    }

    fn add_a_d(&mut self) {
        self.add_r(self.registers.d);
    }

    fn add_a_e(&mut self) {
        self.add_r(self.registers.e);
    }

    fn add_a_h(&mut self) {
        self.add_r(self.registers.h);
    }

    fn add_a_l(&mut self) {
        self.add_r(self.registers.l);
    }

    fn add_a_a(&mut self) {
        self.add_r(self.registers.a);
    }

    fn adc_a_b(&mut self) {
        self.adc_r(self.registers.b);
    }

    fn adc_a_c(&mut self) {
        self.adc_r(self.registers.c);
    }

    fn adc_a_d(&mut self) {
        self.adc_r(self.registers.d);
    }

    fn adc_a_e(&mut self) {
        self.adc_r(self.registers.e);
    }

    fn adc_a_h(&mut self) {
        self.adc_r(self.registers.h);
    }

    fn adc_a_l(&mut self) {
        self.adc_r(self.registers.l);
    }

    fn adc_a_a(&mut self) {
        self.adc_r(self.registers.a);
    }

    fn sub_a_b(&mut self) {
        self.sub_r(self.registers.b);
    }

    fn sub_a_c(&mut self) {
        self.sub_r(self.registers.c);
    }

    fn sub_a_d(&mut self) {
        self.sub_r(self.registers.d);
    }

    fn sub_a_e(&mut self) {
        self.sub_r(self.registers.e);
    }

    fn sub_a_h(&mut self) {
        self.sub_r(self.registers.h);
    }

    fn sub_a_l(&mut self) {
        self.sub_r(self.registers.l);
    }

    fn sub_a_a(&mut self) {
        self.sub_r(self.registers.a);
    }

    fn sbc_a_b(&mut self) {
        self.sbc_r(self.registers.b);
    }

    fn sbc_a_c(&mut self) {
        self.sbc_r(self.registers.c);
    }

    fn sbc_a_d(&mut self) {
        self.sbc_r(self.registers.d);
    }

    fn sbc_a_e(&mut self) {
        self.sbc_r(self.registers.e);
    }

    fn sbc_a_h(&mut self) {
        self.sbc_r(self.registers.h);
    }

    fn sbc_a_l(&mut self) {
        self.sbc_r(self.registers.l);
    }

    fn sbc_a_a(&mut self) {
        self.sbc_r(self.registers.a);
    }

    fn and_b(&mut self) {
        self.and_r(self.registers.b);
    }

    fn and_c(&mut self) {
        self.and_r(self.registers.c);
    }

    fn and_d(&mut self) {
        self.and_r(self.registers.d);
    }

    fn and_e(&mut self) {
        self.and_r(self.registers.e);
    }

    fn and_h(&mut self) {
        self.and_r(self.registers.h);
    }

    fn and_l(&mut self) {
        self.and_r(self.registers.l);
    }

    fn and_a(&mut self) {
        self.and_r(self.registers.a);
    }

    fn xor_b(&mut self) {
        self.xor_r(self.registers.b);
    }

    fn xor_c(&mut self) {
        self.xor_r(self.registers.c);
    }

    fn xor_d(&mut self) {
        self.xor_r(self.registers.d);
    }

    fn xor_e(&mut self) {
        self.xor_r(self.registers.e);
    }

    fn xor_h(&mut self) {
        self.xor_r(self.registers.h);
    }

    fn xor_l(&mut self) {
        self.xor_r(self.registers.l);
    }

    fn xor_a(&mut self) {
        self.xor_r(self.registers.a);
    }

    fn or_b(&mut self) {
        self.or_r(self.registers.b);
    }

    fn or_c(&mut self) {
        self.or_r(self.registers.c);
    }

    fn or_d(&mut self) {
        self.or_r(self.registers.d);
    }

    fn or_e(&mut self) {
        self.or_r(self.registers.e);
    }

    fn or_h(&mut self) {
        self.or_r(self.registers.h);
    }

    fn or_l(&mut self) {
        self.or_r(self.registers.l);
    }

    fn or_a(&mut self) {
        self.or_r(self.registers.a);
    }

    fn cp_b(&mut self) {
        self.cp_r(self.registers.b);
    }

    fn cp_c(&mut self) {
        self.cp_r(self.registers.c);
    }

    fn cp_d(&mut self) {
        self.cp_r(self.registers.d);
    }

    fn cp_e(&mut self) {
        self.cp_r(self.registers.e);
    }

    fn cp_h(&mut self) {
        self.cp_r(self.registers.h);
    }

    fn cp_l(&mut self) {
        self.cp_r(self.registers.l);
    }

    fn cp_a(&mut self) {
        self.cp_r(self.registers.a);
    }

    fn ret_nz(&mut self, memory: &Memory) {
        let cc = !self.registers.f.contains(Flag::ZERO);
        self.ret_cc(cc, memory);
    }

    fn pop_bc(&mut self, memory: &Memory) {
        let bc = self.pop_nn(memory);
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
        self.add(n);
    }

    fn rst_0(&mut self, memory: &mut Memory) {
        self.rst_n(0x0, memory);
    }

    fn ret_z(&mut self, memory: &Memory) {
        let cc = self.registers.f.contains(Flag::ZERO);
        self.ret_cc(cc, memory);
    }

    fn jp_z_nn(&mut self, nn: u16) {
        let cc = self.registers.f.contains(Flag::ZERO);
        self.jp_cc_nn(cc, nn);
    }

    fn ext_ops(&mut self, opcode: u8, memory: &mut Memory) {
        self.instruction_cycle = CB_INSTRUCTION_TIMINGS[opcode as usize];

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
            _ => self.set_7_a(),
        }
    }

    fn call_z_nn(&mut self, nn: u16, memory: &mut Memory) {
        let cc = self.registers.f.contains(Flag::ZERO);
        self.call_cc_nn(cc, nn, memory);
    }

    fn adc_a_n(&mut self, n: u8) {
        self.adc(n);
    }

    fn rst_8(&mut self, memory: &mut Memory) {
        self.rst_n(0x8, memory);
    }

    fn ret_nc(&mut self, memory: &Memory) {
        let cc = !self.registers.f.contains(Flag::FULL_CARRY);
        self.ret_cc(cc, memory);
    }

    fn pop_de(&mut self, memory: &Memory) {
        let de = self.pop_nn(memory);
        self.registers.set_de(de);
    }

    fn jp_nc_nn(&mut self, nn: u16) {
        let cc = !self.registers.f.contains(Flag::FULL_CARRY);
        self.jp_cc_nn(cc, nn);
    }

    fn undefined(&mut self, opcode: u8) {
        println!("Undefined Opcode: {:02X}!", opcode);
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
        self.sub(n);
    }

    fn rst_10(&mut self, memory: &mut Memory) {
        self.rst_n(0x10, memory);
    }

    fn ret_c(&mut self, memory: &Memory) {
        self.ret_cc(self.registers.f.contains(Flag::FULL_CARRY), memory);
    }

    fn ret_i(&mut self, memory: &Memory) {
        self.registers.pc = self.pop(memory);
        self.pending_disable_interrupts = -1;
        self.pending_enable_interrupts = -1;
        self.interrupt_enabled = true;
        self.instruction_cycle = 16;
    }

    fn jp_c_nn(&mut self, nn: u16) {
        self.jp_cc_nn(self.registers.f.contains(Flag::FULL_CARRY), nn);
    }

    fn call_c_nn(&mut self, nn: u16, memory: &mut Memory) {
        self.call_cc_nn(self.registers.f.contains(Flag::FULL_CARRY), nn, memory);
    }

    fn sbc_a_n(&mut self, n: u8) {
        self.sbc(n);
    }

    fn rst_18(&mut self, memory: &mut Memory) {
        self.rst_n(0x18, memory);
    }

    fn pop_hl(&mut self, memory: &Memory) {
        let hl = self.pop_nn(memory);
        self.registers.set_hl(hl);
    }

    fn push_hl(&mut self, memory: &mut Memory) {
        let hl = self.registers.get_hl();
        self.push_nn(hl, memory);
    }

    fn rst_20(&mut self, memory: &mut Memory) {
        self.rst_n(0x20, memory);
    }

    fn jp_hl(&mut self) {
        let hl = self.registers.get_hl();
        self.registers.pc = hl;
    }

    fn rst_28(&mut self, memory: &mut Memory) {
        self.rst_n(0x28, memory);
    }

    fn di(&mut self) {
        self.pending_enable_interrupts = -1;
        if self.pending_disable_interrupts == -1 {
            self.pending_disable_interrupts = 1;
        }
    }

    fn push_af(&mut self, memory: &mut Memory) {
        let af = self.registers.get_af();
        self.push_nn(af, memory);
    }

    fn rst_30(&mut self, memory: &mut Memory) {
        self.rst_n(0x30, memory);
    }

    fn ei(&mut self) {
        self.pending_disable_interrupts = -1;
        if self.pending_enable_interrupts == -1 {
            self.pending_enable_interrupts = 1;
        }
    }

    fn rst_38(&mut self, memory: &mut Memory) {
        self.rst_n(0x38, memory);
    }

    //extended opcodes

    fn rlc_b(&mut self) {
        self.registers.b = self.rlc_r(self.registers.b);
    }

    fn rlc_c(&mut self) {
        self.registers.c = self.rlc_r(self.registers.c);
    }

    fn rlc_d(&mut self) {
        self.registers.d = self.rlc_r(self.registers.d);
    }

    fn rlc_e(&mut self) {
        self.registers.e = self.rlc_r(self.registers.e);
    }

    fn rlc_h(&mut self) {
        self.registers.h = self.rlc_r(self.registers.h);
    }

    fn rlc_l(&mut self) {
        self.registers.l = self.rlc_r(self.registers.l);
    }

    fn rlc_a_cb(&mut self) {
        self.registers.a = self.rlc_r(self.registers.a);
    }

    fn rrc_b(&mut self) {
        self.registers.b = self.rrc_r(self.registers.b);
    }

    fn rrc_c(&mut self) {
        self.registers.c = self.rrc_r(self.registers.c);
    }

    fn rrc_d(&mut self) {
        self.registers.d = self.rrc_r(self.registers.d);
    }

    fn rrc_e(&mut self) {
        self.registers.e = self.rrc_r(self.registers.e);
    }

    fn rrc_h(&mut self) {
        self.registers.h = self.rrc_r(self.registers.h);
    }

    fn rrc_l(&mut self) {
        self.registers.l = self.rrc_r(self.registers.l);
    }

    fn rrc_a_cb(&mut self) {
        self.registers.a = self.rrc_r(self.registers.a);
    }

    fn rl_b(&mut self) {
        self.registers.b = self.rl_r(self.registers.b);
    }

    fn rl_c(&mut self) {
        self.registers.c = self.rl_r(self.registers.c);
    }

    fn rl_d(&mut self) {
        self.registers.d = self.rl_r(self.registers.d);
    }

    fn rl_e(&mut self) {
        self.registers.e = self.rl_r(self.registers.e);
    }

    fn rl_h(&mut self) {
        self.registers.h = self.rl_r(self.registers.h);
    }

    fn rl_l(&mut self) {
        self.registers.l = self.rl_r(self.registers.l);
    }

    fn rl_a_cb(&mut self) {
        self.registers.a = self.rl_r(self.registers.a);
    }

    fn rr_b(&mut self) {
        self.registers.b = self.rr_r(self.registers.b);
    }

    fn rr_c(&mut self) {
        self.registers.c = self.rr_r(self.registers.c);
    }

    fn rr_d(&mut self) {
        self.registers.d = self.rr_r(self.registers.d);
    }

    fn rr_e(&mut self) {
        self.registers.e = self.rr_r(self.registers.e);
    }

    fn rr_h(&mut self) {
        self.registers.h = self.rr_r(self.registers.h);
    }

    fn rr_l(&mut self) {
        self.registers.l = self.rr_r(self.registers.l);
    }

    fn rr_a_cb(&mut self) {
        self.registers.a = self.rr_r(self.registers.a);
    }

    fn sla_b(&mut self) {
        self.registers.b = self.sla_r(self.registers.b);
    }

    fn sla_c(&mut self) {
        self.registers.c = self.sla_r(self.registers.c);
    }

    fn sla_d(&mut self) {
        self.registers.d = self.sla_r(self.registers.d);
    }

    fn sla_e(&mut self) {
        self.registers.e = self.sla_r(self.registers.e);
    }

    fn sla_h(&mut self) {
        self.registers.h = self.sla_r(self.registers.h);
    }

    fn sla_l(&mut self) {
        self.registers.l = self.sla_r(self.registers.l);
    }

    fn sla_a(&mut self) {
        self.registers.a = self.sla_r(self.registers.a);
    }

    fn sra_b(&mut self) {
        self.registers.b = self.sra_r(self.registers.b);
    }

    fn sra_c(&mut self) {
        self.registers.c = self.sra_r(self.registers.c);
    }

    fn sra_d(&mut self) {
        self.registers.d = self.sra_r(self.registers.d);
    }

    fn sra_e(&mut self) {
        self.registers.e = self.sra_r(self.registers.e);
    }

    fn sra_h(&mut self) {
        self.registers.h = self.sra_r(self.registers.h);
    }

    fn sra_l(&mut self) {
        self.registers.l = self.sra_r(self.registers.l);
    }

    fn sra_a(&mut self) {
        self.registers.a = self.sra_r(self.registers.a);
    }

    fn swap_b(&mut self) {
        self.registers.b = self.swap_r(self.registers.b);
    }

    fn swap_c(&mut self) {
        self.registers.c = self.swap_r(self.registers.c);
    }

    fn swap_d(&mut self) {
        self.registers.d = self.swap_r(self.registers.d);
    }

    fn swap_e(&mut self) {
        self.registers.e = self.swap_r(self.registers.e);
    }

    fn swap_h(&mut self) {
        self.registers.h = self.swap_r(self.registers.h);
    }

    fn swap_l(&mut self) {
        self.registers.l = self.swap_r(self.registers.l);
    }

    fn swap_a(&mut self) {
        self.registers.a = self.swap_r(self.registers.a);
    }

    fn srl_b(&mut self) {
        self.registers.b = self.srl_r(self.registers.b);
    }

    fn srl_c(&mut self) {
        self.registers.c = self.srl_r(self.registers.c);
    }

    fn srl_d(&mut self) {
        self.registers.d = self.srl_r(self.registers.d);
    }

    fn srl_e(&mut self) {
        self.registers.e = self.srl_r(self.registers.e);
    }

    fn srl_h(&mut self) {
        self.registers.h = self.srl_r(self.registers.h);
    }

    fn srl_l(&mut self) {
        self.registers.l = self.srl_r(self.registers.l);
    }

    fn srl_a(&mut self) {
        self.registers.a = self.srl_r(self.registers.a);
    }

    fn bit_0_b(&mut self) {
        self.bit_i_r(self.registers.b, 0);
    }

    fn bit_0_c(&mut self) {
        self.bit_i_r(self.registers.c, 0);
    }

    fn bit_0_d(&mut self) {
        self.bit_i_r(self.registers.d, 0);
    }

    fn bit_0_e(&mut self) {
        self.bit_i_r(self.registers.e, 0);
    }

    fn bit_0_h(&mut self) {
        self.bit_i_r(self.registers.h, 0);
    }

    fn bit_0_l(&mut self) {
        self.bit_i_r(self.registers.l, 0);
    }

    fn bit_0_hl(&mut self, memory: &Memory) {
        self.bit_i_hl(0, memory);
    }

    fn bit_0_a(&mut self) {
        self.bit_i_r(self.registers.a, 0);
    }

    fn bit_1_b(&mut self) {
        self.bit_i_r(self.registers.b, 1);
    }

    fn bit_1_c(&mut self) {
        self.bit_i_r(self.registers.c, 1);
    }

    fn bit_1_d(&mut self) {
        self.bit_i_r(self.registers.d, 1);
    }

    fn bit_1_e(&mut self) {
        self.bit_i_r(self.registers.e, 1);
    }

    fn bit_1_h(&mut self) {
        self.bit_i_r(self.registers.h, 1);
    }

    fn bit_1_l(&mut self) {
        self.bit_i_r(self.registers.l, 1);
    }

    fn bit_1_hl(&mut self, memory: &Memory) {
        self.bit_i_hl(1, memory);
    }

    fn bit_1_a(&mut self) {
        self.bit_i_r(self.registers.a, 1);
    }

    fn bit_2_b(&mut self) {
        self.bit_i_r(self.registers.b, 2);
    }

    fn bit_2_c(&mut self) {
        self.bit_i_r(self.registers.c, 2);
    }

    fn bit_2_d(&mut self) {
        self.bit_i_r(self.registers.d, 2);
    }

    fn bit_2_e(&mut self) {
        self.bit_i_r(self.registers.e, 2);
    }

    fn bit_2_h(&mut self) {
        self.bit_i_r(self.registers.h, 2);
    }

    fn bit_2_l(&mut self) {
        self.bit_i_r(self.registers.l, 2);
    }

    fn bit_2_hl(&mut self, memory: &Memory) {
        self.bit_i_hl(2, memory);
    }

    fn bit_2_a(&mut self) {
        self.bit_i_r(self.registers.a, 2);
    }

    fn bit_3_b(&mut self) {
        self.bit_i_r(self.registers.b, 3);
    }

    fn bit_3_c(&mut self) {
        self.bit_i_r(self.registers.c, 3);
    }

    fn bit_3_d(&mut self) {
        self.bit_i_r(self.registers.d, 3);
    }

    fn bit_3_e(&mut self) {
        self.bit_i_r(self.registers.e, 3);
    }

    fn bit_3_h(&mut self) {
        self.bit_i_r(self.registers.h, 3);
    }

    fn bit_3_l(&mut self) {
        self.bit_i_r(self.registers.l, 3);
    }

    fn bit_3_hl(&mut self, memory: &Memory) {
        self.bit_i_hl(3, memory);
    }

    fn bit_3_a(&mut self) {
        self.bit_i_r(self.registers.a, 3);
    }

    fn bit_4_b(&mut self) {
        self.bit_i_r(self.registers.b, 4);
    }

    fn bit_4_c(&mut self) {
        self.bit_i_r(self.registers.c, 4);
    }

    fn bit_4_d(&mut self) {
        self.bit_i_r(self.registers.d, 4);
    }

    fn bit_4_e(&mut self) {
        self.bit_i_r(self.registers.e, 4);
    }

    fn bit_4_h(&mut self) {
        self.bit_i_r(self.registers.h, 4);
    }

    fn bit_4_l(&mut self) {
        self.bit_i_r(self.registers.l, 4);
    }

    fn bit_4_hl(&mut self, memory: &Memory) {
        self.bit_i_hl(4, memory);
    }

    fn bit_4_a(&mut self) {
        self.bit_i_r(self.registers.a, 4);
    }

    fn bit_5_b(&mut self) {
        self.bit_i_r(self.registers.b, 5);
    }

    fn bit_5_c(&mut self) {
        self.bit_i_r(self.registers.c, 5);
    }

    fn bit_5_d(&mut self) {
        self.bit_i_r(self.registers.d, 5);
    }

    fn bit_5_e(&mut self) {
        self.bit_i_r(self.registers.e, 5);
    }

    fn bit_5_h(&mut self) {
        self.bit_i_r(self.registers.h, 5);
    }

    fn bit_5_l(&mut self) {
        self.bit_i_r(self.registers.l, 5);
    }

    fn bit_5_hl(&mut self, memory: &Memory) {
        self.bit_i_hl(5, memory);
    }

    fn bit_5_a(&mut self) {
        self.bit_i_r(self.registers.a, 5);
    }

    fn bit_6_b(&mut self) {
        self.bit_i_r(self.registers.b, 6);
    }

    fn bit_6_c(&mut self) {
        self.bit_i_r(self.registers.c, 6);
    }

    fn bit_6_d(&mut self) {
        self.bit_i_r(self.registers.d, 6);
    }

    fn bit_6_e(&mut self) {
        self.bit_i_r(self.registers.e, 6);
    }

    fn bit_6_h(&mut self) {
        self.bit_i_r(self.registers.h, 6);
    }

    fn bit_6_l(&mut self) {
        self.bit_i_r(self.registers.l, 6);
    }

    fn bit_6_hl(&mut self, memory: &Memory) {
        self.bit_i_hl(6, memory);
    }

    fn bit_6_a(&mut self) {
        self.bit_i_r(self.registers.a, 6);
    }

    fn bit_7_b(&mut self) {
        self.bit_i_r(self.registers.b, 7);
    }

    fn bit_7_c(&mut self) {
        self.bit_i_r(self.registers.c, 7);
    }

    fn bit_7_d(&mut self) {
        self.bit_i_r(self.registers.d, 7);
    }

    fn bit_7_e(&mut self) {
        self.bit_i_r(self.registers.e, 7);
    }

    fn bit_7_h(&mut self) {
        self.bit_i_r(self.registers.h, 7);
    }

    fn bit_7_l(&mut self) {
        self.bit_i_r(self.registers.l, 7);
    }

    fn bit_7_hl(&mut self, memory: &Memory) {
        self.bit_i_hl(7, memory);
    }

    fn bit_7_a(&mut self) {
        self.bit_i_r(self.registers.a, 7);
    }

    fn res_0_b(&mut self) {
        self.registers.b = self.res_i_r(self.registers.b, 0);
    }

    fn res_0_c(&mut self) {
        self.registers.c = self.res_i_r(self.registers.c, 0);
    }

    fn res_0_d(&mut self) {
        self.registers.d = self.res_i_r(self.registers.d, 0);
    }

    fn res_0_e(&mut self) {
        self.registers.e = self.res_i_r(self.registers.e, 0);
    }

    fn res_0_h(&mut self) {
        self.registers.h = self.res_i_r(self.registers.h, 0);
    }

    fn res_0_l(&mut self) {
        self.registers.l = self.res_i_r(self.registers.l, 0);
    }

    fn res_0_hl(&mut self, memory: &mut Memory) {
        self.res_i_hl(0, memory);
    }

    fn res_0_a(&mut self) {
        self.registers.a = self.res_i_r(self.registers.a, 0);
    }

    fn res_1_b(&mut self) {
        self.registers.b = self.res_i_r(self.registers.b, 1);
    }

    fn res_1_c(&mut self) {
        self.registers.c = self.res_i_r(self.registers.c, 1);
    }

    fn res_1_d(&mut self) {
        self.registers.d = self.res_i_r(self.registers.d, 1);
    }

    fn res_1_e(&mut self) {
        self.registers.e = self.res_i_r(self.registers.e, 1);
    }

    fn res_1_h(&mut self) {
        self.registers.h = self.res_i_r(self.registers.h, 1);
    }

    fn res_1_l(&mut self) {
        self.registers.l = self.res_i_r(self.registers.l, 1);
    }

    fn res_1_hl(&mut self, memory: &mut Memory) {
        self.res_i_hl(1, memory);
    }

    fn res_1_a(&mut self) {
        self.registers.a = self.res_i_r(self.registers.a, 1);
    }

    fn res_2_b(&mut self) {
        self.registers.b = self.res_i_r(self.registers.b, 2);
    }

    fn res_2_c(&mut self) {
        self.registers.c = self.res_i_r(self.registers.c, 2);
    }

    fn res_2_d(&mut self) {
        self.registers.d = self.res_i_r(self.registers.d, 2);
    }

    fn res_2_e(&mut self) {
        self.registers.e = self.res_i_r(self.registers.e, 2);
    }

    fn res_2_h(&mut self) {
        self.registers.h = self.res_i_r(self.registers.h, 2);
    }

    fn res_2_l(&mut self) {
        self.registers.l = self.res_i_r(self.registers.l, 2);
    }

    fn res_2_hl(&mut self, memory: &mut Memory) {
        self.res_i_hl(2, memory);
    }

    fn res_2_a(&mut self) {
        self.registers.a = self.res_i_r(self.registers.a, 2);
    }

    fn res_3_b(&mut self) {
        self.registers.b = self.res_i_r(self.registers.b, 3);
    }

    fn res_3_c(&mut self) {
        self.registers.c = self.res_i_r(self.registers.c, 3);
    }

    fn res_3_d(&mut self) {
        self.registers.d = self.res_i_r(self.registers.d, 3);
    }

    fn res_3_e(&mut self) {
        self.registers.e = self.res_i_r(self.registers.e, 3);
    }

    fn res_3_h(&mut self) {
        self.registers.h = self.res_i_r(self.registers.h, 3);
    }

    fn res_3_l(&mut self) {
        self.registers.l = self.res_i_r(self.registers.l, 3);
    }

    fn res_3_hl(&mut self, memory: &mut Memory) {
        self.res_i_hl(3, memory);
    }

    fn res_3_a(&mut self) {
        self.registers.a = self.res_i_r(self.registers.a, 3);
    }

    fn res_4_b(&mut self) {
        self.registers.b = self.res_i_r(self.registers.b, 4);
    }

    fn res_4_c(&mut self) {
        self.registers.c = self.res_i_r(self.registers.c, 4);
    }

    fn res_4_d(&mut self) {
        self.registers.d = self.res_i_r(self.registers.d, 4);
    }

    fn res_4_e(&mut self) {
        self.registers.e = self.res_i_r(self.registers.e, 4);
    }

    fn res_4_h(&mut self) {
        self.registers.h = self.res_i_r(self.registers.h, 4);
    }

    fn res_4_l(&mut self) {
        self.registers.l = self.res_i_r(self.registers.l, 4);
    }

    fn res_4_hl(&mut self, memory: &mut Memory) {
        self.res_i_hl(4, memory);
    }

    fn res_4_a(&mut self) {
        self.registers.a = self.res_i_r(self.registers.a, 4);
    }

    fn res_5_b(&mut self) {
        self.registers.b = self.res_i_r(self.registers.b, 5);
    }

    fn res_5_c(&mut self) {
        self.registers.c = self.res_i_r(self.registers.c, 5);
    }

    fn res_5_d(&mut self) {
        self.registers.d = self.res_i_r(self.registers.d, 5);
    }

    fn res_5_e(&mut self) {
        self.registers.e = self.res_i_r(self.registers.e, 5);
    }

    fn res_5_h(&mut self) {
        self.registers.h = self.res_i_r(self.registers.h, 5);
    }

    fn res_5_l(&mut self) {
        self.registers.l = self.res_i_r(self.registers.l, 5);
    }

    fn res_5_hl(&mut self, memory: &mut Memory) {
        self.res_i_hl(5, memory);
    }

    fn res_5_a(&mut self) {
        self.registers.a = self.res_i_r(self.registers.a, 5);
    }

    fn res_6_b(&mut self) {
        self.registers.b = self.res_i_r(self.registers.b, 6);
    }

    fn res_6_c(&mut self) {
        self.registers.c = self.res_i_r(self.registers.c, 6);
    }

    fn res_6_d(&mut self) {
        self.registers.d = self.res_i_r(self.registers.d, 6);
    }

    fn res_6_e(&mut self) {
        self.registers.e = self.res_i_r(self.registers.e, 6);
    }

    fn res_6_h(&mut self) {
        self.registers.h = self.res_i_r(self.registers.h, 6);
    }

    fn res_6_l(&mut self) {
        self.registers.l = self.res_i_r(self.registers.l, 6);
    }

    fn res_6_hl(&mut self, memory: &mut Memory) {
        self.res_i_hl(6, memory);
    }

    fn res_6_a(&mut self) {
        self.registers.a = self.res_i_r(self.registers.a, 6);
    }

    fn res_7_b(&mut self) {
        self.registers.b = self.res_i_r(self.registers.b, 7);
    }

    fn res_7_c(&mut self) {
        self.registers.c = self.res_i_r(self.registers.c, 7);
    }

    fn res_7_d(&mut self) {
        self.registers.d = self.res_i_r(self.registers.d, 7);
    }

    fn res_7_e(&mut self) {
        self.registers.e = self.res_i_r(self.registers.e, 7);
    }

    fn res_7_h(&mut self) {
        self.registers.h = self.res_i_r(self.registers.h, 7);
    }

    fn res_7_l(&mut self) {
        self.registers.l = self.res_i_r(self.registers.l, 7);
    }

    fn res_7_hl(&mut self, memory: &mut Memory) {
        self.res_i_hl(7, memory);
    }

    fn res_7_a(&mut self) {
        self.registers.a = self.res_i_r(self.registers.a, 7);
    }

    fn set_0_b(&mut self) {
        self.registers.b = self.set_i_r(self.registers.b, 0);
    }

    fn set_0_c(&mut self) {
        self.registers.c = self.set_i_r(self.registers.c, 0);
    }

    fn set_0_d(&mut self) {
        self.registers.d = self.set_i_r(self.registers.d, 0);
    }

    fn set_0_e(&mut self) {
        self.registers.e = self.set_i_r(self.registers.e, 0);
    }

    fn set_0_h(&mut self) {
        self.registers.h = self.set_i_r(self.registers.h, 0);
    }

    fn set_0_l(&mut self) {
        self.registers.l = self.set_i_r(self.registers.l, 0);
    }

    fn set_0_hl(&mut self, memory: &mut Memory) {
        self.set_i_hl(0, memory);
    }

    fn set_0_a(&mut self) {
        self.registers.a = self.set_i_r(self.registers.a, 0);
    }

    fn set_1_b(&mut self) {
        self.registers.b = self.set_i_r(self.registers.b, 1);
    }

    fn set_1_c(&mut self) {
        self.registers.c = self.set_i_r(self.registers.c, 1);
    }

    fn set_1_d(&mut self) {
        self.registers.d = self.set_i_r(self.registers.d, 1);
    }

    fn set_1_e(&mut self) {
        self.registers.e = self.set_i_r(self.registers.e, 1);
    }

    fn set_1_h(&mut self) {
        self.registers.h = self.set_i_r(self.registers.h, 1);
    }

    fn set_1_l(&mut self) {
        self.registers.l = self.set_i_r(self.registers.l, 1);
    }

    fn set_1_hl(&mut self, memory: &mut Memory) {
        self.set_i_hl(1, memory);
    }

    fn set_1_a(&mut self) {
        self.registers.a = self.set_i_r(self.registers.a, 1);
    }

    fn set_2_b(&mut self) {
        self.registers.b = self.set_i_r(self.registers.b, 2);
    }

    fn set_2_c(&mut self) {
        self.registers.c = self.set_i_r(self.registers.c, 2);
    }

    fn set_2_d(&mut self) {
        self.registers.d = self.set_i_r(self.registers.d, 2);
    }

    fn set_2_e(&mut self) {
        self.registers.e = self.set_i_r(self.registers.e, 2);
    }

    fn set_2_h(&mut self) {
        self.registers.h = self.set_i_r(self.registers.h, 2);
    }

    fn set_2_l(&mut self) {
        self.registers.l = self.set_i_r(self.registers.l, 2);
    }

    fn set_2_hl(&mut self, memory: &mut Memory) {
        self.set_i_hl(2, memory);
    }

    fn set_2_a(&mut self) {
        self.registers.a = self.set_i_r(self.registers.a, 2);
    }

    fn set_3_b(&mut self) {
        self.registers.b = self.set_i_r(self.registers.b, 3);
    }

    fn set_3_c(&mut self) {
        self.registers.c = self.set_i_r(self.registers.c, 3);
    }

    fn set_3_d(&mut self) {
        self.registers.d = self.set_i_r(self.registers.d, 3);
    }

    fn set_3_e(&mut self) {
        self.registers.e = self.set_i_r(self.registers.e, 3);
    }

    fn set_3_h(&mut self) {
        self.registers.h = self.set_i_r(self.registers.h, 3);
    }

    fn set_3_l(&mut self) {
        self.registers.l = self.set_i_r(self.registers.l, 3);
    }

    fn set_3_hl(&mut self, memory: &mut Memory) {
        self.set_i_hl(3, memory);
    }

    fn set_3_a(&mut self) {
        self.registers.a = self.set_i_r(self.registers.a, 3);
    }

    fn set_4_b(&mut self) {
        self.registers.b = self.set_i_r(self.registers.b, 4);
    }

    fn set_4_c(&mut self) {
        self.registers.c = self.set_i_r(self.registers.c, 4);
    }

    fn set_4_d(&mut self) {
        self.registers.d = self.set_i_r(self.registers.d, 4);
    }

    fn set_4_e(&mut self) {
        self.registers.e = self.set_i_r(self.registers.e, 4);
    }

    fn set_4_h(&mut self) {
        self.registers.h = self.set_i_r(self.registers.h, 4);
    }

    fn set_4_l(&mut self) {
        self.registers.l = self.set_i_r(self.registers.l, 4);
    }

    fn set_4_hl(&mut self, memory: &mut Memory) {
        self.set_i_hl(4, memory);
    }

    fn set_4_a(&mut self) {
        self.registers.a = self.set_i_r(self.registers.a, 4);
    }

    fn set_5_b(&mut self) {
        self.registers.b = self.set_i_r(self.registers.b, 5);
    }

    fn set_5_c(&mut self) {
        self.registers.c = self.set_i_r(self.registers.c, 5);
    }

    fn set_5_d(&mut self) {
        self.registers.d = self.set_i_r(self.registers.d, 5);
    }

    fn set_5_e(&mut self) {
        self.registers.e = self.set_i_r(self.registers.e, 5);
    }

    fn set_5_h(&mut self) {
        self.registers.h = self.set_i_r(self.registers.h, 5);
    }

    fn set_5_l(&mut self) {
        self.registers.l = self.set_i_r(self.registers.l, 5);
    }

    fn set_5_hl(&mut self, memory: &mut Memory) {
        self.set_i_hl(5, memory);
    }

    fn set_5_a(&mut self) {
        self.registers.a = self.set_i_r(self.registers.a, 5);
    }

    fn set_6_b(&mut self) {
        self.registers.b = self.set_i_r(self.registers.b, 6);
    }

    fn set_6_c(&mut self) {
        self.registers.c = self.set_i_r(self.registers.c, 6);
    }

    fn set_6_d(&mut self) {
        self.registers.d = self.set_i_r(self.registers.d, 6);
    }

    fn set_6_e(&mut self) {
        self.registers.e = self.set_i_r(self.registers.e, 6);
    }

    fn set_6_h(&mut self) {
        self.registers.h = self.set_i_r(self.registers.h, 6);
    }

    fn set_6_l(&mut self) {
        self.registers.l = self.set_i_r(self.registers.l, 6);
    }

    fn set_6_hl(&mut self, memory: &mut Memory) {
        self.set_i_hl(6, memory);
    }

    fn set_6_a(&mut self) {
        self.registers.a = self.set_i_r(self.registers.a, 6);
    }

    fn set_7_b(&mut self) {
        self.registers.b = self.set_i_r(self.registers.b, 7);
    }

    fn set_7_c(&mut self) {
        self.registers.c = self.set_i_r(self.registers.c, 7);
    }

    fn set_7_d(&mut self) {
        self.registers.d = self.set_i_r(self.registers.d, 7);
    }

    fn set_7_e(&mut self) {
        self.registers.e = self.set_i_r(self.registers.e, 7);
    }

    fn set_7_h(&mut self) {
        self.registers.h = self.set_i_r(self.registers.h, 7);
    }

    fn set_7_l(&mut self) {
        self.registers.l = self.set_i_r(self.registers.l, 7);
    }

    fn set_7_hl(&mut self, memory: &mut Memory) {
        self.set_i_hl(7, memory);
    }

    fn set_7_a(&mut self) {
        self.registers.a = self.set_i_r(self.registers.a, 7);
    }

    //interrupts

    pub fn rst_40(&mut self, memory: &mut Memory) {
        self.rst_n(0x40, memory);
    }

    pub fn rst_48(&mut self, memory: &mut Memory) {
        self.rst_n(0x48, memory);
    }

    pub fn rst_50(&mut self, memory: &mut Memory) {
        self.rst_n(0x50, memory);
    }

    pub fn rst_58(&mut self, memory: &mut Memory) {
        self.rst_n(0x58, memory);
    }

    pub fn rst_60(&mut self, memory: &mut Memory) {
        self.rst_n(0x60, memory);
    }

    // LOADS
    fn ld_r_r(&mut self, r: u8) -> u8 {
        r
    }

    fn ld_r_n(&mut self, n: u8) -> u8 {
        n
    }

    fn ld_rr_r(&mut self, rr: u16, r: u8, memory: &mut Memory) {
        memory.write_byte(rr, r);
    }

    fn ld_a_nn(&mut self, nn: u16, memory: &Memory) {
        self.registers.a = memory.read_byte(nn);
    }

    fn ld_r_rr(&mut self, rr: u16, memory: &Memory) -> u8 {
        memory.read_byte(rr)
    }

    fn ld_hl_r(&mut self, r: u8, memory: &mut Memory) {
        memory.write_byte(self.registers.get_hl(), r);
    }

    fn ld_hl_n(&mut self, n: u8, memory: &mut Memory) {
        memory.write_byte(self.registers.get_hl(), n);
    }

    fn ld_r_hl(&mut self, memory: &Memory) -> u8 {
        memory.read_byte(self.registers.get_hl())
    }

    fn ld_nn_a(&mut self, nn: u16, memory: &mut Memory) {
        memory.write_byte(nn, self.registers.a);
    }

    fn ldi_hl_a(&mut self, memory: &mut Memory) {
        memory.write_byte(self.registers.get_hl(), self.registers.a);
        let hl = self.registers.get_hl().wrapping_add(1);
        self.registers.set_hl(hl);
    }

    fn inc_hl(&mut self) {
        let hl = self.inc_rr(self.registers.get_hl());
        self.registers.set_hl(hl);
    }

    fn ldi_a_hl(&mut self, memory: &Memory) {
        self.registers.a = memory.read_byte(self.registers.get_hl());
        let hl = self.registers.get_hl().wrapping_add(1);
        self.registers.set_hl(hl);
    }

    fn dec_hl(&mut self) {
        let hl = self.dec_rr(self.registers.get_hl());
        self.registers.set_hl(hl);
    }

    fn ldd_hl_a(&mut self, memory: &mut Memory) {
        memory.write_byte(self.registers.get_hl(), self.registers.a);
        let hl = self.registers.get_hl().wrapping_sub(1);
        self.registers.set_hl(hl);
    }

    fn ldd_a_hl(&mut self, memory: &Memory) {
        self.registers.a = memory.read_byte(self.registers.get_hl());
        let hl = self.registers.get_hl().wrapping_sub(1);
        self.registers.set_hl(hl);
    }

    fn ldh_a_n(&mut self, n: u8, memory: &Memory) {
        self.registers.a = memory.read_byte(0xFF00 + u16::from(n));
    }

    fn ldh_n_a(&mut self, n: u8, memory: &mut Memory) {
        memory.write_byte(0xFF00 + u16::from(n), self.registers.a);
    }

    fn ldh_a_c(&mut self, memory: &mut Memory) {
        memory.write_byte(0xFF00 + u16::from(self.registers.c), self.registers.a);
    }

    fn ldh_c_a(&mut self, memory: &mut Memory) {
        self.registers.a = memory.read_byte(0xFF00 + u16::from(self.registers.c));
    }

    fn ld_rr_nn(&mut self, nn: u16) -> u16 {
        nn
    }

    fn ld_sp_hl(&mut self) {
        self.registers.sp = self.registers.get_hl();
    }

    fn ld_hl_sp_e(&mut self, e: u8) {
        let hl = self.add_sp(e);
        self.registers.set_hl(hl);
    }

    fn ld_nn_sp(&mut self, nn: u16, memory: &mut Memory) {
        memory.write_word(nn, self.registers.sp);
    }

    fn push(&mut self, nn: u16, memory: &mut Memory) {
        self.registers.sp -= 2;
        memory.write_word(self.registers.sp, nn);
    }

    fn push_nn(&mut self, rr: u16, memory: &mut Memory) {
        self.push(rr, memory);
    }

    fn pop(&mut self, memory: &Memory) -> u16 {
        let word = memory.read_word(self.registers.sp);
        self.registers.sp += 2;

        word
    }

    fn pop_nn(&mut self, memory: &Memory) -> u16 {
        self.pop(memory)
    }

    fn pop_af(&mut self, memory: &Memory) {
        let nn = self.pop(memory);
        self.registers.set_af(nn);
    }

    //alu

    fn add(&mut self, n: u8) {
        let half_carry = ((self.registers.a & 0xF) + (n & 0xF)) & 0x10 == 0x10;
        let full_carry = (u16::from(self.registers.a) + u16::from(n)) & 0x100 == 0x100;

        self.registers.a = self.registers.a.wrapping_add(n);

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.remove(Flag::NEGATIVE);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
    }

    fn add_r(&mut self, r: u8) {
        self.add(r);
    }

    fn add_hl(&mut self, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.add(n);
    }

    fn sub(&mut self, n: u8) {
        let half_carry = (self.registers.a & 0xF) < (n & 0xF);
        let full_carry = self.registers.a < n;
        let zero = self.registers.a == n;

        self.registers.a = self.registers.a.wrapping_sub(n);

        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
        self.registers.f.set(Flag::ZERO, zero);
        self.registers.f.insert(Flag::NEGATIVE);
    }

    fn sub_r(&mut self, r: u8) {
        self.sub(r);
    }

    fn sub_hl(&mut self, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.sub(n);
    }

    fn sbc(&mut self, n: u8) {
        let carry = if self.registers.f.contains(Flag::FULL_CARRY) {
            1u8
        } else {
            0u8
        };

        let result = i16::from(self.registers.a) - i16::from(n) - i16::from(carry);

        let half_carry =
            i16::from(self.registers.a & 0x0F) - i16::from(n & 0x0F) - (i16::from(carry)) < 0;
        let full_carry = result < 0;

        self.registers.a = self.registers.a.wrapping_sub(n).wrapping_sub(carry);

        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.insert(Flag::NEGATIVE);
    }

    fn sbc_r(&mut self, r: u8) {
        self.sbc(r);
    }

    fn sbc_hl(&mut self, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.sbc(n);
    }

    fn adc(&mut self, n: u8) {
        let carry = if self.registers.f.contains(Flag::FULL_CARRY) {
            1u8
        } else {
            0u8
        };

        let half_carry = ((self.registers.a & 0xF) + (n & 0xF) + carry) & 0x10 == 0x10;
        let full_carry =
            (u16::from(self.registers.a) + u16::from(n) + u16::from(carry)) & 0x100 == 0x100;

        self.registers.a = self.registers.a.wrapping_add(n).wrapping_add(carry);

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.remove(Flag::NEGATIVE);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
    }

    fn adc_r(&mut self, r: u8) {
        self.adc(r);
    }

    fn adc_hl(&mut self, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.adc(n);
    }

    fn and(&mut self, n: u8) {
        self.registers.a &= n;

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers.f.insert(Flag::HALF_CARRY);
        self.registers.f.remove(Flag::NEGATIVE | Flag::FULL_CARRY);
    }

    fn and_r(&mut self, r: u8) {
        self.and(r);
    }

    fn and_n(&mut self, n: u8) {
        self.and(n);
    }

    fn and_hl(&mut self, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.and(n);
    }

    fn or(&mut self, n: u8) {
        self.registers.a |= n;

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers
            .f
            .remove(Flag::NEGATIVE | Flag::FULL_CARRY | Flag::HALF_CARRY);
    }

    fn or_r(&mut self, r: u8) {
        self.or(r);
    }

    fn or_n(&mut self, n: u8) {
        self.or(n);
    }

    fn or_hl(&mut self, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.or(n);
    }

    fn xor(&mut self, n: u8) {
        self.registers.a ^= n;

        self.registers.f.set(Flag::ZERO, self.registers.a == 0);
        self.registers
            .f
            .remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY);
    }

    fn xor_r(&mut self, r: u8) {
        self.xor(r);
    }

    fn xor_n(&mut self, n: u8) {
        self.xor(n);
    }

    fn xor_hl(&mut self, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.xor(n);
    }

    fn cp(&mut self, n: u8) {
        let half_carry = (self.registers.a & 0xF) < (n & 0xF);
        let overflow = self.registers.a < n;

        self.registers.f.set(Flag::FULL_CARRY, overflow);
        self.registers.f.set(Flag::ZERO, self.registers.a == n);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.insert(Flag::NEGATIVE);
    }

    fn cp_r(&mut self, r: u8) {
        self.cp(r);
    }

    fn cp_n(&mut self, n: u8) {
        self.cp(n);
    }

    fn cp_hl(&mut self, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.cp(n);
    }

    fn inc(&mut self, n: u8) -> u8 {
        let half_carry = (((n & 0xF) + 1) & 0x10) == 0x10;

        let n = n.wrapping_add(1);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.remove(Flag::NEGATIVE);

        n
    }

    fn inc_r(&mut self, r: u8) -> u8 {
        self.inc(r)
    }

    fn inc_hl_ref(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.inc(n);
        memory.write_byte(self.registers.get_hl(), n);
    }

    fn inc_rr(&mut self, rr: u16) -> u16 {
        rr.wrapping_add(1)
    }

    fn dec(&mut self, n: u8) -> u8 {
        let half_carry = (n & 0xF) < 1;

        let n = n.wrapping_sub(1);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.insert(Flag::NEGATIVE);

        n
    }

    fn dec_r(&mut self, r: u8) -> u8 {
        self.dec(r)
    }

    fn dec_hl_ref(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.dec(n);
        memory.write_byte(self.registers.get_hl(), n);
    }

    fn dec_rr(&mut self, rr: u16) -> u16 {
        rr.wrapping_sub(1)
    }

    fn add_hl_rr(&mut self, rr: u16) {
        let hl = self.registers.get_hl();
        let result = u32::from(hl) + u32::from(rr);

        let half_carry = (hl & 0xFFF) + (rr & 0xFFF) > 0xFFF;
        let full_carry = result > 0xFFFF;

        self.registers.set_hl(hl.wrapping_add(rr));

        self.registers.f.remove(Flag::NEGATIVE);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);
    }

    fn add_sp_s(&mut self, e: u8) {
        self.registers.sp = self.add_sp(e);
    }

    fn add_sp(&mut self, n: u8) -> u16 {
        let s = i16::from(n as i8) as u16;

        let half_carry = (self.registers.sp & 0x000F) + (n as i8 as u16 & 0x000F) > 0x000F;
        let full_carry = (self.registers.sp & 0x00FF) + (s & 0x00FF) > 0x00FF;

        self.registers.f.remove(Flag::ZERO | Flag::NEGATIVE);
        self.registers.f.set(Flag::HALF_CARRY, half_carry);
        self.registers.f.set(Flag::FULL_CARRY, full_carry);

        self.registers.sp.wrapping_add(s)
    }

    // rotates

    fn rlc(&mut self, n: u8) -> u8 {
        let left_bit = (n & 0x80) == 0x80;

        let n = n.rotate_left(1);
        self.registers.f.set(Flag::FULL_CARRY, left_bit);

        n
    }

    fn rlc_a(&mut self) {
        self.registers.a = self.rlc(self.registers.a);

        self.registers
            .f
            .remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::ZERO);
    }

    fn rlc_r(&mut self, r: u8) -> u8 {
        let r = self.rlc(r);

        self.registers.f.set(Flag::ZERO, r == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        r
    }

    fn rlc_hl(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.rlc(n);
        memory.write_byte(self.registers.get_hl(), n);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);
    }

    fn rrc(&mut self, n: u8) -> u8 {
        let right_bit = (n & 0x1) == 1;
        let n = n.rotate_right(1);

        self.registers.f.set(Flag::FULL_CARRY, right_bit);

        n
    }

    fn rrc_a(&mut self) {
        self.registers.a = self.rrc(self.registers.a);
        self.registers
            .f
            .remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::ZERO);
    }

    fn rrc_r(&mut self, r: u8) -> u8 {
        let r = self.rrc(r);

        self.registers.f.set(Flag::ZERO, r == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        r
    }

    fn rrc_hl(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.rrc(n);
        memory.write_byte(self.registers.get_hl(), n);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);
    }

    fn rr(&mut self, n: u8) -> u8 {
        let right_bit = (n & 0x1) == 1;
        let mut n = n >> 1;

        n |= (self.registers.f.contains(Flag::FULL_CARRY) as u8) << 7;

        self.registers.f.set(Flag::FULL_CARRY, right_bit);

        n
    }

    fn rr_a(&mut self) {
        self.registers.a = self.rr(self.registers.a);

        self.registers
            .f
            .remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::ZERO);
    }

    fn rr_r(&mut self, r: u8) -> u8 {
        let r = self.rr(r);

        self.registers.f.set(Flag::ZERO, r == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        r
    }

    fn rr_hl(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.rr(n);
        memory.write_byte(self.registers.get_hl(), n);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);
    }

    fn rl(&mut self, n: u8) -> u8 {
        let left_bit = (n & 0x80) == 0x80;
        let mut n = n << 1;
        n |= self.registers.f.contains(Flag::FULL_CARRY) as u8;

        self.registers.f.set(Flag::FULL_CARRY, left_bit);

        n
    }

    fn rl_a(&mut self) {
        self.registers.a = self.rl(self.registers.a);

        self.registers
            .f
            .remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::ZERO);
    }

    fn rl_r(&mut self, r: u8) -> u8 {
        let r = self.rl(r);

        self.registers.f.set(Flag::ZERO, r == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        r
    }

    fn rl_hl(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.rl(n);
        memory.write_byte(self.registers.get_hl(), n);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);
    }

    fn sla(&mut self, n: u8) -> u8 {
        let left_bit = (n & 0x80) == 0x80;
        let n = n << 1;

        self.registers.f.set(Flag::FULL_CARRY, left_bit);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        n
    }

    fn sla_r(&mut self, r: u8) -> u8 {
        self.sla(r)
    }

    fn sla_hl(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.sla(n);
        memory.write_byte(self.registers.get_hl(), n);
    }

    fn sra(&mut self, n: u8) -> u8 {
        let left = n & 0x80;
        let right_bit = (n & 0x1) == 1;

        let mut n = n >> 1;
        n |= left;

        self.registers.f.set(Flag::FULL_CARRY, right_bit);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

        n
    }

    fn sra_r(&mut self, r: u8) -> u8 {
        self.sra(r)
    }

    fn sra_hl(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.sra(n);
        memory.write_byte(self.registers.get_hl(), n);
    }

    fn srl(&mut self, n: u8) -> u8 {
        let first = n & 1 == 1;
        let n = n >> 1;

        self.registers.f.set(Flag::FULL_CARRY, first);
        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers.f.remove(Flag::HALF_CARRY | Flag::NEGATIVE);

        n
    }

    fn srl_r(&mut self, r: u8) -> u8 {
        self.srl(r)
    }

    fn srl_hl(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.srl(n);
        memory.write_byte(self.registers.get_hl(), n);
    }

    // bit opcodes

    fn bit_i(&mut self, i: u8, n: u8) {
        self.registers.f.insert(Flag::HALF_CARRY);
        self.registers.f.remove(Flag::NEGATIVE);
        self.registers.f.set(Flag::ZERO, !bit_utils::is_set(n, i));
    }

    fn bit_i_r(&mut self, r: u8, i: u8) {
        self.bit_i(i, r);
    }

    fn bit_i_hl(&mut self, i: u8, memory: &Memory) {
        let n = memory.read_byte(self.registers.get_hl());
        self.bit_i(i, n);
    }

    fn set_i(&mut self, i: u8, n: u8) -> u8 {
        bit_utils::set_bit(n, i)
    }

    fn set_i_r(&mut self, r: u8, i: u8) -> u8 {
        self.set_i(i, r)
    }

    fn set_i_hl(&mut self, i: u8, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.set_i(i, n);
        memory.write_byte(self.registers.get_hl(), n);
    }

    fn res_i(&mut self, i: u8, n: u8) -> u8 {
        bit_utils::unset_bit(n, i)
    }

    fn res_i_r(&mut self, r: u8, i: u8) -> u8 {
        self.res_i(i, r)
    }

    fn res_i_hl(&mut self, i: u8, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.res_i(i, n);
        memory.write_byte(self.registers.get_hl(), n);
    }

    // functions

    fn call_nn(&mut self, nn: u16, memory: &mut Memory) {
        self.push(self.registers.pc, memory);
        self.registers.pc = nn;
    }

    fn call_cc_nn(&mut self, cc: bool, nn: u16, memory: &mut Memory) {
        if cc {
            self.push(self.registers.pc, memory);
            self.registers.pc = nn;
            self.instruction_cycle = 24;
        }
    }

    fn rst_n(&mut self, n: u8, memory: &mut Memory) {
        self.push(self.registers.pc, memory);
        self.registers.pc = u16::from(n);
    }

    fn ret(&mut self, memory: &Memory) {
        self.registers.pc = self.pop(memory);
        self.instruction_cycle = 16;
    }

    fn ret_cc(&mut self, cc: bool, memory: &Memory) {
        if cc {
            self.registers.pc = self.pop(memory);
            self.instruction_cycle = 20;
        }
    }

    fn jp_cc_nn(&mut self, cc: bool, nn: u16) {
        if cc {
            self.registers.pc = nn;
            self.instruction_cycle = 16;
        }
    }

    fn jr_cc_n(&mut self, cc: bool, n: u8) {
        if cc {
            self.registers.pc = self.registers.pc.wrapping_add(i16::from(n as i8) as u16);
            self.instruction_cycle = 12;
        }
    }

    // misc

    fn swap(&mut self, n: u8) -> u8 {
        let high = n & 0xF0;
        let low = n & 0xF;

        let n = (low << 4) | (high >> 4);

        self.registers.f.set(Flag::ZERO, n == 0);
        self.registers
            .f
            .remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY);

        n
    }

    fn swap_r(&mut self, r: u8) -> u8 {
        self.swap(r)
    }

    fn swap_hl(&mut self, memory: &mut Memory) {
        let mut n = memory.read_byte(self.registers.get_hl());
        n = self.swap(n);
        memory.write_byte(self.registers.get_hl(), n);
    }
}
