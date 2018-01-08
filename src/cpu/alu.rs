use cpu::registers::flag::Flag;

pub fn add_a_n(a: u8, n: u8, flag: &mut Flag) -> u8 {
    let half_carry = ((a & 0xF) + (n & 0xF)) & 0x10 == 0x10;
    let full_carry = ((a as u16) + (n as u16)) & 0x100 == 0x100;

    let a = a.wrapping_add(n);

    flag.set(Flag::ZERO, a == 0);
    flag.remove(Flag::NEGATIVE);
    flag.set(Flag::HALF_CARRY, half_carry);
    flag.set(Flag::FULL_CARRY, full_carry);

    a
}

pub fn adc_a_n(a: u8, n: u8, flag: &mut Flag) -> u8 {
    let n = if flag.contains(Flag::FULL_CARRY) {
        n + 1
    } else {
        n
    };

    let half_carry = ((a & 0xF) + (n & 0xF)) & 0x10 == 0x10;
    let full_carry = ((a as u16) + (n as u16)) & 0x100 == 0x100;

    let a = a.wrapping_add(n);

    flag.set(Flag::ZERO, a == 0);
    flag.set(Flag::FULL_CARRY, full_carry);
    flag.set(Flag::HALF_CARRY, half_carry);
    flag.remove(Flag::NEGATIVE);

    a
}

pub fn sub_a_n(a: u8, n: u8, flag: &mut Flag) -> u8 {
    let half_carry = (a & 0xF) < (n & 0xF);
    let full_carry = a < n;
    let zero = a == n;

    let a = a.wrapping_sub(n);

    flag.set(Flag::HALF_CARRY, half_carry);
    flag.set(Flag::FULL_CARRY, full_carry);
    flag.set(Flag::ZERO, zero);
    flag.insert(Flag::NEGATIVE);

    a
}

pub fn sbc_a_n(a: u8, n: u8, flag: &mut Flag) -> u8 {
    let n = if flag.contains(Flag::FULL_CARRY) {
        n + 1
    } else {
        n
    };

    let half_carry = (a & 0xF) < (n & 0xF);
    let full_carry = a < n;
    let zero = a == n;

    let a = a.wrapping_sub(n);

    flag.set(Flag::ZERO, zero);
    flag.set(Flag::FULL_CARRY, full_carry);
    flag.set(Flag::HALF_CARRY, half_carry);
    flag.insert(Flag::NEGATIVE);

    a
}

pub fn and_a_n(a: u8, n: u8, flag: &mut Flag) -> u8 {
    let a = a & n;

    flag.set(Flag::ZERO, a == 0);
    flag.insert(Flag::HALF_CARRY);
    flag.remove(Flag::NEGATIVE | Flag::FULL_CARRY);

    a
}

pub fn or_a_n(a: u8, n: u8, flag: &mut Flag) -> u8 {
    let a = a | n;

    flag.set(Flag::ZERO, a == 0);
    flag.remove(Flag::NEGATIVE | Flag::FULL_CARRY | Flag::HALF_CARRY);

    a
}

pub fn xor_a_n(a: u8, n: u8, flag: &mut Flag) -> u8 {
    let a = a ^ n;

    flag.set(Flag::ZERO, a == 0);
    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY);

    a
}

pub fn cp_a_n(a: u8, n: u8, flag: &mut Flag) {
    let half_carry = (a & 0xF) < (n & 0xF);
    let overflow = a < n;

    flag.set(Flag::FULL_CARRY, overflow);
    flag.set(Flag::ZERO, a == n);
    flag.set(Flag::HALF_CARRY, half_carry);
    flag.insert(Flag::NEGATIVE);
}

pub fn inc_n(n: u8, flag: &mut Flag) -> u8 {
    let half_carry = (((n & 0xF) + 1) & 0x10) == 0x10;

    let n = n.wrapping_add(1);

    flag.set(Flag::ZERO, n == 0);
    flag.set(Flag::HALF_CARRY, half_carry);
    flag.remove(Flag::NEGATIVE);

    n
}

pub fn dec_n(n: u8, flag: &mut Flag) -> u8 {
    // convert 1 into a 2's complement signed value
    // then add it to n, checking for the half carry
    let half_carry = ((n & 0xF) + (0xF)) & 0x10 == 0x10;

    let n = n.wrapping_sub(1);

    flag.set(Flag::ZERO, n == 0);
    flag.set(Flag::HALF_CARRY, half_carry);
    flag.insert(Flag::NEGATIVE);

    n
}

pub fn inc_nn(nn: u16) -> u16 { nn + 1 }

pub fn dec_nn(nn: u16) -> u16 { nn - 1 }

pub fn add_hl_nn(hl: u16, nn: u16, flag: &mut Flag) -> u16 {
    let half_carry = (((hl & 0x3FFF) + (nn & 0x3FFF)) & 0x4000) == 0x4000;
    let full_carry = ((hl as u32) + (nn as u32) & 0x10000) == 0x10000;

    let hl = hl.wrapping_add(nn);

    flag.remove(Flag::NEGATIVE);
    flag.set(Flag::HALF_CARRY, half_carry);
    flag.set(Flag::FULL_CARRY, full_carry);

    hl
}

pub fn add_sp_e(sp: u16, e: u8, flag: &mut Flag) -> u16 {
    let d = e as i8 as u16;

    let half_carry = ((sp & 0x3FFF) + (d & 0x3FFF)) & 0x4000 == 0x4000;
    let full_carry = ((sp as u32) + (d as u32)) & 0x10000 == 0x10000;

    let sp = sp.wrapping_add(d);

    flag.remove(Flag::ZERO | Flag::NEGATIVE);
    flag.set(Flag::HALF_CARRY, half_carry);
    flag.set(Flag::FULL_CARRY, full_carry);

    sp
}

pub fn bit_n_i(n: u8, i: u8, flag: &mut Flag) {
    flag.insert(Flag::HALF_CARRY);
    flag.remove(Flag::NEGATIVE);
    flag.set(Flag::ZERO, n & (1 << i) == 0);
}

pub fn set_n_i(n: u8, i: u8) -> u8 { n | (1 << i) }

pub fn res_n_i(n: u8, i: u8) -> u8 { n & !((1 << i) as u8) }

pub fn swap_n(n: u8, flag: &mut Flag) -> u8 {
    let high = n & 0xF0;
    let low = n & 0xF;

    let n = (low << 4) | (high >> 4);

    flag.set(Flag::ZERO, n == 0);
    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY);

    n
}

pub fn rlc_a(a: u8, flag: &mut Flag) -> u8 {
    let a = rlc(a, flag);

    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::ZERO);

    a
}

pub fn rlc_n(n: u8, flag: &mut Flag) -> u8 {
    let n = rlc(n, flag);

    flag.set(Flag::ZERO, n == 0);
    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

    n
}

fn rlc(n: u8, flag: &mut Flag) -> u8 {
    let left_bit = (n & 0x80) == 0x80;

    let n = n.rotate_left(1);
    flag.set(Flag::FULL_CARRY, left_bit);

    n
}

pub fn rrc_a(a: u8, flag: &mut Flag) -> u8 {
    let a = rrc(a, flag);

    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::ZERO);

    a
}

pub fn rrc_n(n: u8, flag: &mut Flag) -> u8 {
    let n = rrc(n, flag);

    flag.set(Flag::ZERO, n == 0);
    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

    n
}

pub fn rrc(n: u8, flag: &mut Flag) -> u8 {
    let right_bit = (n & 0x1) == 1;
    let n = n.rotate_right(1);

    flag.set(Flag::FULL_CARRY, right_bit);

    n
}

pub fn rl_a(a: u8, flag: &mut Flag) -> u8 {
    let a = rl(a, flag);

    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::ZERO);

    a
}

pub fn rl_n(n: u8, flag: &mut Flag) -> u8 {
    let n = rl(n, flag);

    flag.set(Flag::ZERO, n == 0);
    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

    n
}

fn rl(n: u8, flag: &mut Flag) -> u8 {
    let left_bit = (n & 0x80) == 0x80;
    let mut n = n << 1;
    n |= flag.contains(Flag::FULL_CARRY) as u8;

    flag.set(Flag::FULL_CARRY, left_bit);

    n
}

pub fn rr_a(a: u8, flag: &mut Flag) -> u8 {
    let a = rr(a, flag);

    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY | Flag::ZERO);

    a
}

pub fn rr_n(n: u8, flag: &mut Flag) -> u8 {
    let n = rr(n, flag);

    flag.set(Flag::ZERO, n == 0);
    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

    n
}

fn rr(n: u8, flag: &mut Flag) -> u8 {
    let right_bit = (n & 0x1) == 1;
    let mut n = n >> 1;

    n |= (flag.contains(Flag::FULL_CARRY) as u8) << 7;

    flag.set(Flag::FULL_CARRY, right_bit);

    n
}

pub fn sla_n(n: u8, flag: &mut Flag) -> u8 {
    let left_bit = (n & 0x80) == 0x80;
    let n = n << 1;

    flag.set(Flag::FULL_CARRY, left_bit);
    flag.set(Flag::ZERO, n == 0);
    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

    n
}

pub fn sra_n(n: u8, flag: &mut Flag) -> u8 {
    let left = n & 0x80;
    let right_bit = (n & 0x1) == 1;

    let mut n = n >> 1;
    n |= left;

    flag.set(Flag::FULL_CARRY, right_bit);
    flag.set(Flag::ZERO, n == 0);
    flag.remove(Flag::NEGATIVE | Flag::HALF_CARRY);

    n
}

pub fn srl_n(n: u8, flag: &mut Flag) -> u8 {
    let first = n & 1 == 1;
    let n = n >> 1;

    flag.set(Flag::FULL_CARRY, first);
    flag.set(Flag::ZERO, n == 0);
    flag.remove(Flag::HALF_CARRY | Flag::NEGATIVE);

    n
}

#[cfg(test)]
mod tests {
    use cpu::alu;
    use cpu::registers::flag::Flag;

    #[test]
    fn test_inc_nn() {
        let mut n = 1;

        n = alu::inc_nn(n);

        assert_eq!(n, 2);
    }

    #[test]
    fn test_dec_nn() {
        let mut n = 1;

        n = alu::dec_nn(n);

        assert_eq!(n, 0);
    }

    #[test]
    fn test_inc_n() {
        // cause an 8 bit overflow, triggering a zero
        // and half carry flag
        let mut n = 0xFF;
        let mut flag = Flag::empty();

        n = alu::inc_n(n, &mut flag);

        assert_eq!(n, 0);
        assert_eq!(flag, Flag::ZERO | Flag::HALF_CARRY);
    }

    #[test]
    fn test_dec_n() {
        let mut n = 1;
        let mut flag = Flag::empty();

        n = alu::dec_n(n, &mut flag);

        assert_eq!(n, 0);
        assert_eq!(flag, Flag::ZERO | Flag::NEGATIVE | Flag::HALF_CARRY);
    }

    #[test]
    fn test_add_a_n() {
        let a = 0xF;
        let n = 0xF1;
        let mut flag = Flag::empty();

        let a = alu::add_a_n(a, n, &mut flag);

        assert_eq!(a, 0);
        assert_eq!(flag, Flag::ZERO | Flag::HALF_CARRY | Flag::FULL_CARRY);
    }

    #[test]
    fn test_sub_a_n() {
        let mut a = 0xF;
        let n = 0xF;
        let mut flag = Flag::empty();

        a = alu::sub_a_n(a, n, &mut flag);

        assert_eq!(a, 0);
        assert_eq!(flag, Flag::ZERO | Flag::NEGATIVE);
    }

    #[test]
    fn test_adc_a_n() {
        let mut a = 0xF;
        let n = 0xF0;
        let mut flag = Flag::FULL_CARRY;

        a = alu::adc_a_n(a, n, &mut flag);

        assert_eq!(a, 0);
        assert_eq!(flag, Flag::ZERO | Flag::HALF_CARRY | Flag::FULL_CARRY);
    }

    #[test]
    fn test_sbc_a_n() {
        let mut a = 0xF;
        let n = 0xE;
        let mut flag = Flag::FULL_CARRY;

        a = alu::sbc_a_n(a, n, &mut flag);

        assert_eq!(a, 0);
        assert_eq!(flag, Flag::ZERO | Flag::NEGATIVE);
    }

    #[test]
    fn test_add_hl_ss() {
        let ss = 0x7F7F;
        let mut hl = 0x7F7F;
        let mut flag = Flag::empty();

        hl = alu::add_hl_nn(hl, ss, &mut flag);

        assert_eq!(hl, 0xFEFE);
        assert_eq!(flag, Flag::HALF_CARRY);
    }

    #[test]
    fn test_bit_i_n() {
        let mut flag = Flag::empty();

        alu::bit_n_i(0, 0, &mut flag);

        assert_eq!(flag, Flag::HALF_CARRY | Flag::ZERO);
    }

    #[test]
    fn test_set_i_n() {
        let n = alu::set_n_i(0, 0);

        assert_eq!(n, 1);
    }

    #[test]
    fn test_res_i_n() {
        let n = alu::res_n_i(0, 1);

        assert_eq!(n, 0);
    }

    #[test]
    fn test_rlc_a() {
        let mut a = 0b00000000;
        let mut flag = Flag::ZERO | Flag::HALF_CARRY | Flag::NEGATIVE | Flag::FULL_CARRY;

        a = alu::rlc_a(a, &mut flag);

        assert_eq!(a, 0);
        assert!(flag.is_empty());
    }

    #[test]
    fn test_rlc_n() {
        let mut n = 0b10101010;
        let mut flag = Flag::ZERO | Flag::FULL_CARRY | Flag::HALF_CARRY | Flag::NEGATIVE;

        n = alu::rlc_n(n, &mut flag);

        assert_eq!(n, 0b01010101);
        assert_eq!(flag, Flag::FULL_CARRY);
    }

    #[test]
    fn test_rrc_a() {
        let mut a = 0b10101010;
        let mut flag = Flag::ZERO | Flag::FULL_CARRY | Flag::HALF_CARRY | Flag::NEGATIVE;

        a = alu::rrc_a(a, &mut flag);

        assert_eq!(a, 0b01010101);
        assert!(flag.is_empty());
    }

    #[test]
    fn test_rrc_n() {
        let mut flag = Flag::FULL_CARRY;

        let n = alu::rrc_n(0, &mut flag);

        assert_eq!(n, 0);
        assert_eq!(flag, Flag::ZERO);
    }

    #[test]
    fn test_rl_n() {
        let mut flag = Flag::FULL_CARRY;

        let n = alu::rl_n(0b00101010, &mut flag);

        assert_eq!(n, 0b01010101);
        assert!(flag.is_empty());
    }

    #[test]
    fn test_rr_n() {
        let mut flag = Flag::FULL_CARRY;

        let n = alu::rr_n(0b10101010, &mut flag);

        assert_eq!(n, 0b11010101);
        assert!(flag.is_empty());
    }

    #[test]
    fn test_sla_n() {
        let mut flag = Flag::empty();

        let n = alu::sla_n(0b10000000, &mut flag);

        assert_eq!(n, 0);
        assert_eq!(flag, Flag::FULL_CARRY | Flag::ZERO);
    }

    #[test]
    fn test_sra_n() {
        let mut flag = Flag::empty();

        let n = alu::sra_n(0b10011001, &mut flag);

        assert_eq!(n, 0b11001100);
        assert_eq!(flag, Flag::FULL_CARRY);
    }

    #[test]
    fn test_srl_n() {
        let mut flag = Flag::empty();

        let n = alu::srl_n(0b11111111, &mut flag);

        assert_eq!(n, 0b01111111);
        assert_eq!(flag, Flag::FULL_CARRY);
    }

    #[test]
    fn test_swap_n() {
        let mut flag = Flag::ZERO | Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY;

        let n = alu::swap_n(0b11110000, &mut flag);

        assert_eq!(n, 0b00001111);
        assert!(flag.is_empty());
    }

    #[test]
    fn test_add_sp_e() {
        let mut flag = Flag::empty();
        let mut sp = 0xFFF0;
        let d = 0b11111111;

        sp = alu::add_sp_e(sp, d, &mut flag);

        assert_eq!(flag, Flag::HALF_CARRY | Flag::FULL_CARRY);
        assert_eq!(sp, 0xFFEF);
    }

    #[test]
    fn test_and_a_n() {
        let mut a = 0xFF;
        let mut flag = Flag::FULL_CARRY | Flag::NEGATIVE;

        a = alu::and_a_n(a, 0, &mut flag);

        assert_eq!(a, 0);
        assert_eq!(flag, Flag::ZERO | Flag::HALF_CARRY);
    }

    #[test]
    fn test_or_a_n() {
        let mut a = 0;
        let mut flag = Flag::ZERO | Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY;

        a = alu::or_a_n(a, 0xFF, &mut flag);

        assert_eq!(a, 0xFF);
        assert!(flag.is_empty());
    }

    #[test]
    fn test_xor_a_n() {
        let mut a = 0b11110000;
        let mut flag = Flag::ZERO | Flag::NEGATIVE | Flag::HALF_CARRY | Flag::FULL_CARRY;

        a = alu::xor_a_n(a, 0xFF, &mut flag);

        assert_eq!(a, 0b00001111);
        assert!(flag.is_empty());
    }

    #[test]
    fn test_cp_a_n() {
        let mut flag = Flag::empty();
        let a = 0xFF;

        alu::cp_a_n(a, 0xFF, &mut flag);

        assert_eq!(flag, Flag::ZERO | Flag::NEGATIVE);
    }
}