#[cfg(test)]
mod tests {
    use cpu::registers::flag::Flag;
    use cpu::Cpu;
    use mmu::Memory;

    #[test]
    fn opcode_01() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_word(0xC000, 0x1234);

        cpu.execute_opcode(0x01, &mut memory);

        assert_eq!(cpu.registers.get_bc(), 0x1234);
    }

    #[test]
    fn opcode_02() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_bc(0xC000);
        cpu.registers.a = 0xFF;

        cpu.execute_opcode(0x02, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xFF);
    }

    #[test]
    fn opcode_03() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x03, &mut memory);

        assert_eq!(cpu.registers.get_bc(), 0x0001);
    }

    #[test]
    fn opcode_04() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x04, &mut memory);

        assert_eq!(cpu.registers.b, 0x01);
    }

    #[test]
    fn opcode_05() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.b = 1;

        cpu.execute_opcode(0x05, &mut memory);

        assert_eq!(cpu.registers.b, 0x00);
    }

    #[test]
    fn opcode_06() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_word(0xC000, 0xFF);

        cpu.execute_opcode(0x06, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_07() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b10000000;

        cpu.execute_opcode(0x07, &mut memory);

        assert_eq!(cpu.registers.a, 0b00000001);
        assert!(cpu.registers.f.contains(Flag::FULL_CARRY));
    }

    #[test]
    fn opcode_08() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0x1234;
        memory.write_word(0xC000, 0xD000);
        cpu.registers.pc = 0xC000;

        cpu.execute_opcode(0x08, &mut memory);

        assert_eq!(memory.read_word(0xD000), 0x1234);
    }

    #[test]
    fn opcode_09() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_bc(0xFF00);
        cpu.registers.set_hl(0x00FF);

        cpu.execute_opcode(0x09, &mut memory);

        assert_eq!(cpu.registers.get_hl(), 0xFFFF);
    }

    #[test]
    fn opcode_0a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_bc(0xC000);
        memory.write_byte(0xC000, 0xFF);

        cpu.execute_opcode(0x0A, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_0b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_bc(1);

        cpu.execute_opcode(0x0B, &mut memory);

        assert_eq!(cpu.registers.get_bc(), 0x00);
    }

    #[test]
    fn opcode_0c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x0C, &mut memory);

        assert_eq!(cpu.registers.c, 0x01);
    }

    #[test]
    fn opcode_0d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.c = 0x1;
        cpu.execute_opcode(0x0D, &mut memory);

        assert_eq!(cpu.registers.c, 0x00);
    }

    #[test]
    fn opcode_0e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_byte(0xC000, 0xFF);

        cpu.execute_opcode(0xE, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_0f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00000001;
        cpu.execute_opcode(0xF, &mut memory);

        assert_eq!(cpu.registers.a, 0b10000000);
        assert!(cpu.registers.f.contains(Flag::FULL_CARRY));
    }

    #[test]
    fn opcode_10() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x10, &mut memory);

        assert!(cpu.stopped);
    }

    #[test]
    fn opcode_11() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        memory.write_word(0xC000, 0x1234);
        cpu.registers.pc = 0xC000;

        cpu.execute_opcode(0x11, &mut memory);

        assert_eq!(cpu.registers.get_de(), 0x1234);
    }

    #[test]
    fn opcode_12() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_de(0xC000);
        cpu.registers.a = 0xFF;

        cpu.execute_opcode(0x12, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xFF);
    }

    #[test]
    fn opcode_13() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x13, &mut memory);

        assert_eq!(cpu.registers.get_de(), 0x1);
    }

    #[test]
    fn opcode_14() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x14, &mut memory);

        assert_eq!(cpu.registers.d, 0x1);
    }

    #[test]
    fn opcode_15() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.d = 1;
        cpu.execute_opcode(0x15, &mut memory);

        assert_eq!(cpu.registers.d, 0);
    }

    #[test]
    fn opcode_16() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        memory.write_byte(0xC000, 0xFF);
        cpu.registers.pc = 0xC000;

        cpu.execute_opcode(0x16, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_17() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.f.insert(Flag::FULL_CARRY);

        cpu.execute_opcode(0x17, &mut memory);

        assert_eq!(cpu.registers.a, 1);
        assert!(!cpu.registers.f.contains(Flag::FULL_CARRY));
    }

    #[test]
    fn opcode_18() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        memory.write_byte(0xC000, 0xFF);
        cpu.registers.pc = 0xC000;
        cpu.execute_opcode(0x18, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC000);
    }

    #[test]
    fn opcode_19() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_de(0xFFFF);
        cpu.execute_opcode(0x19, &mut memory);

        assert_eq!(cpu.registers.get_hl(), 0xFFFF);
    }

    #[test]
    fn opcode_1a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_de(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x1A, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_1b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_de(1);
        cpu.execute_opcode(0x1B, &mut memory);

        assert_eq!(cpu.registers.get_de(), 0);
    }

    #[test]
    fn opcode_1c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x1C, &mut memory);

        assert_eq!(cpu.registers.e, 1);
    }

    #[test]
    fn opcode_1d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.e = 1;
        cpu.execute_opcode(0x1D, &mut memory);

        assert_eq!(cpu.registers.e, 0);
    }

    #[test]
    fn opcode_1e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        memory.write_byte(0xC000, 0xFF);
        cpu.registers.pc = 0xC000;
        cpu.execute_opcode(0x1E, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_1f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x1F, &mut memory);

        assert_eq!(cpu.registers.a, 0b10000000);
        assert!(!cpu.registers.f.contains(Flag::FULL_CARRY));
    }

    #[test]
    fn opcode_20() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x20, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC000);
    }

    #[test]
    fn opcode_21() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_word(0xC000, 0x1234);
        cpu.execute_opcode(0x21, &mut memory);

        assert_eq!(cpu.registers.get_hl(), 0x1234);
    }

    #[test]
    fn opcode_22() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x22, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xFF);
        assert_eq!(cpu.registers.get_hl(), 0xC001);
    }

    #[test]
    fn opcode_23() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x23, &mut memory);

        assert_eq!(cpu.registers.get_hl(), 0x0001);
    }

    #[test]
    fn opcode_24() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x24, &mut memory);

        assert_eq!(cpu.registers.h, 0x1);
    }

    #[test]
    fn opcode_25() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.h = 1;
        cpu.execute_opcode(0x25, &mut memory);

        assert_eq!(cpu.registers.h, 0);
    }

    #[test]
    fn opcode_26() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x26, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_27() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x2E;
        cpu.execute_opcode(0x27, &mut memory);

        assert_eq!(cpu.registers.a, 0x34);
    }

    #[test]
    fn opcode_28() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.f.insert(Flag::ZERO);
        memory.write_byte(0xC000, 0xFF);
        cpu.registers.pc = 0xC000;
        cpu.execute_opcode(0x28, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC000);
    }

    #[test]
    fn opcode_29() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0x2000);
        cpu.execute_opcode(0x29, &mut memory);

        assert_eq!(cpu.registers.get_hl(), 0x4000);
    }

    #[test]
    fn opcode_2a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x2A, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
        assert_eq!(cpu.registers.get_hl(), 0xC001);
    }

    #[test]
    fn opcode_2b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(1);
        cpu.execute_opcode(0x2b, &mut memory);

        assert_eq!(cpu.registers.get_hl(), 0);
    }

    #[test]
    fn opcode_2c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x2C, &mut memory);

        assert_eq!(cpu.registers.l, 1);
    }

    #[test]
    fn opcode_2d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.l = 1;
        cpu.execute_opcode(0x2D, &mut memory);

        assert_eq!(cpu.registers.l, 0);
    }

    #[test]
    fn opcode_2e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        memory.write_byte(0xC000, 0xFF);
        cpu.registers.pc = 0xC000;
        cpu.execute_opcode(0x2E, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_2f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x2F, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_30() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x30, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC000);
    }

    #[test]
    fn opcode_31() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        memory.write_word(0xC000, 0x1234);
        cpu.registers.pc = 0xC000;
        cpu.execute_opcode(0x31, &mut memory);

        assert_eq!(cpu.registers.sp, 0x1234);
    }

    #[test]
    fn opcode_32() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC001);
        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x32, &mut memory);

        assert_eq!(memory.read_byte(0xC001), 0xFF);
        assert_eq!(cpu.registers.get_hl(), 0xC000);
    }

    #[test]
    fn opcode_33() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x33, &mut memory);

        assert_eq!(cpu.registers.sp, 1);
    }

    #[test]
    fn opcode_34() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        cpu.execute_opcode(0x34, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 1);
    }

    #[test]
    fn opcode_35() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 1);
        cpu.execute_opcode(0x35, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0);
    }

    #[test]
    fn opcode_36() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.set_hl(0xC002);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x36, &mut memory);

        assert_eq!(memory.read_byte(0xC002), 0xFF);
    }

    #[test]
    fn opcode_37() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x37, &mut memory);

        assert!(cpu.registers.f.contains(Flag::FULL_CARRY));
    }

    #[test]
    fn opcode_38() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.registers.pc = 0xC000;
        memory.write_byte(0xC000, 0xFF);

        assert_eq!(cpu.registers.pc, 0xC000);
    }

    #[test]
    fn opcode_39() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0x00FF;
        cpu.registers.set_hl(0xFF00);
        cpu.execute_opcode(0x39, &mut memory);

        assert_eq!(cpu.registers.get_hl(), 0xFFFF);
    }

    #[test]
    fn opcode_3a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x3a, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
        assert_eq!(cpu.registers.get_hl(), 0xBFFF);
    }

    #[test]
    fn opcode_3b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0x0001;
        cpu.execute_opcode(0x3b, &mut memory);

        assert_eq!(cpu.registers.sp, 0);
    }

    #[test]
    fn opcode_3c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x3c, &mut memory);

        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn opcode_3d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 1;
        cpu.execute_opcode(0x3d, &mut memory);

        assert_eq!(cpu.registers.a, 0);
    }

    #[test]
    fn opcode_3e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x3e, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_3f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x3f, &mut memory);

        assert!(cpu.registers.f.contains(Flag::FULL_CARRY));
    }

    #[test]
    fn opcode_40() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.b = 0xFF;
        cpu.execute_opcode(0x40, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_41() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.c = 0xFF;
        cpu.execute_opcode(0x41, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_42() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.d = 0xFF;
        cpu.execute_opcode(0x42, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_43() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.e = 0xFF;
        cpu.execute_opcode(0x43, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_44() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.h = 0xFF;
        cpu.execute_opcode(0x44, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_45() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.l = 0xFF;
        cpu.execute_opcode(0x45, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_46() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x46, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_47() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x47, &mut memory);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn opcode_48() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.b = 0xFF;
        cpu.execute_opcode(0x48, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_49() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.c = 0xFF;
        cpu.execute_opcode(0x49, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_4a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.d = 0xFF;
        cpu.execute_opcode(0x4a, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_4b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.e = 0xFF;
        cpu.execute_opcode(0x4b, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_4c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.h = 0xFF;
        cpu.execute_opcode(0x4c, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_4d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.l = 0xFF;
        cpu.execute_opcode(0x4d, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_4e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x4e, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_4f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x4f, &mut memory);

        assert_eq!(cpu.registers.c, 0xFF);
    }

    #[test]
    fn opcode_50() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.b = 0xFF;
        cpu.execute_opcode(0x50, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_51() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.c = 0xFF;
        cpu.execute_opcode(0x51, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_52() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.d = 0xFF;
        cpu.execute_opcode(0x52, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_53() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.e = 0xFF;
        cpu.execute_opcode(0x53, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_54() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.h = 0xFF;
        cpu.execute_opcode(0x54, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_55() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.l = 0xFF;
        cpu.execute_opcode(0x55, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_56() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x56, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_57() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x57, &mut memory);

        assert_eq!(cpu.registers.d, 0xFF);
    }

    #[test]
    fn opcode_58() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.b = 0xFF;
        cpu.execute_opcode(0x58, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_59() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.c = 0xFF;
        cpu.execute_opcode(0x59, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_5a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.d = 0xFF;
        cpu.execute_opcode(0x5a, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_5b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.e = 0xFF;
        cpu.execute_opcode(0x5b, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_5c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.h = 0xFF;
        cpu.execute_opcode(0x5c, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_5d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.l = 0xFF;
        cpu.execute_opcode(0x5d, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_5e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x5e, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_5f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x5f, &mut memory);

        assert_eq!(cpu.registers.e, 0xFF);
    }

    #[test]
    fn opcode_60() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.b = 0xFF;
        cpu.execute_opcode(0x60, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_61() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.c = 0xFF;
        cpu.execute_opcode(0x61, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_62() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.d = 0xFF;
        cpu.execute_opcode(0x62, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_63() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.e = 0xFF;
        cpu.execute_opcode(0x63, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_64() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.h = 0xFF;
        cpu.execute_opcode(0x64, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_65() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.l = 0xFF;
        cpu.execute_opcode(0x65, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_66() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x66, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_67() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x67, &mut memory);

        assert_eq!(cpu.registers.h, 0xFF);
    }

    #[test]
    fn opcode_68() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.b = 0xFF;
        cpu.execute_opcode(0x68, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_69() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.c = 0xFF;
        cpu.execute_opcode(0x69, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_6a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.d = 0xFF;
        cpu.execute_opcode(0x6a, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_6b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.e = 0xFF;
        cpu.execute_opcode(0x6b, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_6c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.h = 0xFF;
        cpu.execute_opcode(0x6c, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_6d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.l = 0xFF;
        cpu.execute_opcode(0x6d, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_6e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x6e, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_6f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x6f, &mut memory);

        assert_eq!(cpu.registers.l, 0xFF);
    }

    #[test]
    fn opcode_70() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        cpu.registers.b = 0xFF;
        cpu.execute_opcode(0x70, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xFF);
    }

    #[test]
    fn opcode_71() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        cpu.registers.c = 0xFF;
        cpu.execute_opcode(0x71, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xFF);
    }

    #[test]
    fn opcode_72() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        cpu.registers.d = 0xFF;
        cpu.execute_opcode(0x72, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xFF);
    }

    #[test]
    fn opcode_73() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        cpu.registers.e = 0xFF;
        cpu.execute_opcode(0x73, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xFF);
    }

    #[test]
    fn opcode_74() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        cpu.execute_opcode(0x74, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xC0);
    }

    #[test]
    fn opcode_75() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC0FF);
        cpu.execute_opcode(0x75, &mut memory);

        assert_eq!(memory.read_byte(0xC0FF), 0xFF);
    }

    #[test]
    fn opcode_76() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0x76, &mut memory);

        assert!(cpu.halted);
    }

    #[test]
    fn opcode_77() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x77, &mut memory);

        assert_eq!(memory.read_byte(0xC000), 0xFF);
    }

    #[test]
    fn opcode_78() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.b = 0xFF;
        cpu.execute_opcode(0x78, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_79() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.c = 0xFF;
        cpu.execute_opcode(0x79, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_7a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.d = 0xFF;
        cpu.execute_opcode(0x7a, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_7b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.e = 0xFF;
        cpu.execute_opcode(0x7b, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_7c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.h = 0xFF;
        cpu.execute_opcode(0x7c, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_7d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.l = 0xFF;
        cpu.execute_opcode(0x7d, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_7e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xFF);
        cpu.execute_opcode(0x7e, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_7f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0x7f, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_80() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.b = 0x20;
        cpu.execute_opcode(0x80, &mut memory);

        assert_eq!(cpu.registers.a, 0x30);
    }

    #[test]
    fn opcode_81() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.c = 0x20;
        cpu.execute_opcode(0x81, &mut memory);

        assert_eq!(cpu.registers.a, 0x30);
    }

    #[test]
    fn opcode_82() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.d = 0x20;
        cpu.execute_opcode(0x82, &mut memory);

        assert_eq!(cpu.registers.a, 0x30);
    }

    #[test]
    fn opcode_83() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.e = 0x20;
        cpu.execute_opcode(0x83, &mut memory);

        assert_eq!(cpu.registers.a, 0x30);
    }

    #[test]
    fn opcode_84() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.h = 0x20;
        cpu.execute_opcode(0x84, &mut memory);

        assert_eq!(cpu.registers.a, 0x30);
    }

    #[test]
    fn opcode_85() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.l = 0x20;
        cpu.execute_opcode(0x85, &mut memory);

        assert_eq!(cpu.registers.a, 0x30);
    }

    #[test]
    fn opcode_86() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.set_hl(0xC000);
        memory.write_word(0xC000, 0x20);
        cpu.execute_opcode(0x86, &mut memory);

        assert_eq!(cpu.registers.a, 0x30);
    }

    #[test]
    fn opcode_87() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.execute_opcode(0x87, &mut memory);

        assert_eq!(cpu.registers.a, 0x20);
    }

    #[test]
    fn opcode_88() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.b = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x88, &mut memory);

        assert_eq!(cpu.registers.a, 0x31);
    }

    #[test]
    fn opcode_89() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.c = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x89, &mut memory);

        assert_eq!(cpu.registers.a, 0x31);
    }

    #[test]
    fn opcode_8a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.d = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x8a, &mut memory);

        assert_eq!(cpu.registers.a, 0x31);
    }

    #[test]
    fn opcode_8b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.e = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x8b, &mut memory);

        assert_eq!(cpu.registers.a, 0x31);
    }

    #[test]
    fn opcode_8c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.h = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x8c, &mut memory);

        assert_eq!(cpu.registers.a, 0x31);
    }

    #[test]
    fn opcode_8d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.l = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x8d, &mut memory);

        assert_eq!(cpu.registers.a, 0x31);
    }

    #[test]
    fn opcode_8e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0x20);
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x8e, &mut memory);

        assert_eq!(cpu.registers.a, 0x31);
    }

    #[test]
    fn opcode_8f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x10;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x8f, &mut memory);

        assert_eq!(cpu.registers.a, 0x21);
    }

    #[test]
    fn opcode_90() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.b = 0x20;
        cpu.execute_opcode(0x90, &mut memory);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn opcode_91() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.c = 0x20;
        cpu.execute_opcode(0x91, &mut memory);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn opcode_92() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.d = 0x20;
        cpu.execute_opcode(0x92, &mut memory);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn opcode_93() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.e = 0x20;
        cpu.execute_opcode(0x93, &mut memory);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn opcode_94() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.h = 0x20;
        cpu.execute_opcode(0x94, &mut memory);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn opcode_95() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.l = 0x20;
        cpu.execute_opcode(0x95, &mut memory);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn opcode_96() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0x20);
        cpu.execute_opcode(0x96, &mut memory);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn opcode_97() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.execute_opcode(0x97, &mut memory);

        assert_eq!(cpu.registers.a, 0);
    }

    #[test]
    fn opcode_98() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.b = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x98, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
    }

    #[test]
    fn opcode_99() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.c = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x99, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
    }

    #[test]
    fn opcode_9a() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.d = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x9a, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
    }

    #[test]
    fn opcode_9b() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.e = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x9b, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
    }

    #[test]
    fn opcode_9c() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.h = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x9c, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
    }

    #[test]
    fn opcode_9d() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.l = 0x20;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x9d, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
    }

    #[test]
    fn opcode_9e() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0x20);
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x9e, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
    }

    #[test]
    fn opcode_9f() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x30;
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0x9f, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_a0() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.registers.b = 1;
        cpu.execute_opcode(0xa0, &mut memory);

        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn opcode_a1() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.registers.c = 1;
        cpu.execute_opcode(0xa1, &mut memory);

        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn opcode_a2() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.registers.d = 1;
        cpu.execute_opcode(0xa2, &mut memory);

        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn opcode_a3() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.registers.e = 1;
        cpu.execute_opcode(0xa3, &mut memory);

        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn opcode_a4() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.registers.h = 1;
        cpu.execute_opcode(0xa4, &mut memory);

        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn opcode_a5() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.registers.l = 1;
        cpu.execute_opcode(0xa5, &mut memory);

        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn opcode_a6() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 1);
        cpu.execute_opcode(0xa6, &mut memory);

        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn opcode_a7() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0xFF;
        cpu.execute_opcode(0xa7, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_a8() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00111100;
        cpu.registers.b = 0b11110000;
        cpu.execute_opcode(0xa8, &mut memory);

        assert_eq!(cpu.registers.a, 0b11001100);
    }

    #[test]
    fn opcode_a9() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00111100;
        cpu.registers.c = 0b11110000;
        cpu.execute_opcode(0xa9, &mut memory);

        assert_eq!(cpu.registers.a, 0b11001100);
    }

    #[test]
    fn opcode_aa() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00111100;
        cpu.registers.d = 0b11110000;
        cpu.execute_opcode(0xaa, &mut memory);

        assert_eq!(cpu.registers.a, 0b11001100);
    }

    #[test]
    fn opcode_ab() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00111100;
        cpu.registers.e = 0b11110000;
        cpu.execute_opcode(0xab, &mut memory);

        assert_eq!(cpu.registers.a, 0b11001100);
    }

    #[test]
    fn opcode_ac() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00111100;
        cpu.registers.h = 0b11110000;
        cpu.execute_opcode(0xac, &mut memory);

        assert_eq!(cpu.registers.a, 0b11001100);
    }

    #[test]
    fn opcode_ad() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00111100;
        cpu.registers.l = 0b11110000;
        cpu.execute_opcode(0xad, &mut memory);

        assert_eq!(cpu.registers.a, 0b11001100);
    }

    #[test]
    fn opcode_ae() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00111100;
        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0b11110000);
        cpu.execute_opcode(0xae, &mut memory);

        assert_eq!(cpu.registers.a, 0b11001100);
    }

    #[test]
    fn opcode_af() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0b00111100;
        cpu.execute_opcode(0xaf, &mut memory);

        assert_eq!(cpu.registers.a, 0);
    }

    #[test]
    fn opcode_b0() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.b = 0xF0;
        cpu.execute_opcode(0xb0, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_b1() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.c = 0xF0;
        cpu.execute_opcode(0xb1, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_b2() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.d = 0xF0;
        cpu.execute_opcode(0xb2, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_b3() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.e = 0xF0;
        cpu.execute_opcode(0xb3, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_b4() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.h = 0xF0;
        cpu.execute_opcode(0xb4, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_b5() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.l = 0xF0;
        cpu.execute_opcode(0xb5, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_b6() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0xF0);
        cpu.execute_opcode(0xb6, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_b7() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.execute_opcode(0xb7, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
    }

    #[test]
    fn opcode_b8() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.b = 0x0F;
        cpu.execute_opcode(0xb8, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.f.contains(Flag::ZERO));
    }

    #[test]
    fn opcode_b9() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.c = 0x0F;
        cpu.execute_opcode(0xb9, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.f.contains(Flag::ZERO));
    }

    #[test]
    fn opcode_ba() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.d = 0x0F;
        cpu.execute_opcode(0xba, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.f.contains(Flag::ZERO));
    }

    #[test]
    fn opcode_bb() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.e = 0x0F;
        cpu.execute_opcode(0xbb, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.f.contains(Flag::ZERO));
    }

    #[test]
    fn opcode_bc() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.h = 0x0F;
        cpu.execute_opcode(0xbc, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.f.contains(Flag::ZERO));
    }

    #[test]
    fn opcode_bd() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.l = 0x0F;
        cpu.execute_opcode(0xbd, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.f.contains(Flag::ZERO));
    }

    #[test]
    fn opcode_be() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.registers.set_hl(0xC000);
        memory.write_byte(0xC000, 0x0F);
        cpu.execute_opcode(0xbe, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.f.contains(Flag::ZERO));
    }

    #[test]
    fn opcode_bf() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.a = 0x0F;
        cpu.execute_opcode(0xbf, &mut memory);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.f.contains(Flag::ZERO));
    }

    #[test]
    fn opcode_c0() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0xD000;
        memory.write_word(cpu.registers.sp, 0xC000);
        cpu.execute_opcode(0xc0, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC000);
        assert_eq!(cpu.registers.sp, 0xD002);
    }

    #[test]
    fn opcode_c1() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0xC000;
        cpu.push(0x1234, &mut memory);
        cpu.execute_opcode(0xc1, &mut memory);

        assert_eq!(cpu.registers.get_bc(), 0x1234);
        assert_eq!(cpu.registers.sp, 0xC000);
    }

    #[test]
    fn opcode_c2() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_word(0xC000, 0x1234);
        cpu.execute_opcode(0xC2, &mut memory);

        assert_eq!(cpu.registers.pc, 0x1234);
    }

    #[test]
    fn opcode_c3() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_word(0xC000, 0x1234);
        cpu.execute_opcode(0xC3, &mut memory);

        assert_eq!(cpu.registers.pc, 0x1234);
    }

    #[test]
    fn opcode_c4() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xD000;
        memory.write_word(0xC000, 0xC500);
        cpu.execute_opcode(0xc4, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC500);
        assert_eq!(cpu.registers.sp, 0xCFFE);
        assert_eq!(memory.read_word(cpu.registers.sp), 0xC002);
    }

    #[test]
    fn opcode_c5() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0xC000;
        cpu.registers.set_bc(0x1234);
        cpu.execute_opcode(0xc5, &mut memory);

        assert_eq!(cpu.pop(&memory), 0x1234);
        assert_eq!(cpu.registers.sp, 0xC000);
    }

    #[test]
    fn opcode_c6() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_byte(0xC000, 0xF0);
        cpu.registers.a = 0x0F;
        cpu.execute_opcode(0xC6, &mut memory);

        assert_eq!(cpu.registers.a, 0xFF);
    }

    #[test]
    fn opcode_c7() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xD000;
        cpu.execute_opcode(0xC7, &mut memory);

        assert_eq!(cpu.registers.pc, 0);
        assert_eq!(cpu.pop(&memory), 0xC000);
    }

    #[test]
    fn opcode_c8() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0xD000;
        memory.write_word(cpu.registers.sp, 0xC000);
        cpu.registers.f.insert(Flag::ZERO);
        cpu.execute_opcode(0xc8, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC000);
        assert_eq!(cpu.registers.sp, 0xD002);
    }

    #[test]
    fn opcode_c9() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0xD000;
        memory.write_word(cpu.registers.sp, 0xC000);
        cpu.execute_opcode(0xc9, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC000);
        assert_eq!(cpu.registers.sp, 0xD002);
    }

    #[test]
    fn opcode_ca() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.f.insert(Flag::ZERO);
        memory.write_word(0xC000, 0xD000);
        cpu.execute_opcode(0xca, &mut memory);

        assert_eq!(cpu.registers.pc, 0xD000);
    }

    #[test]
    fn opcode_cb() {
        //TODO
    }

    #[test]
    fn opcode_cc() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xD000;
        cpu.registers.f.insert(Flag::ZERO);
        memory.write_word(0xC000, 0xC500);
        cpu.execute_opcode(0xcc, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC500);
        assert_eq!(cpu.registers.sp, 0xCFFE);
        assert_eq!(memory.read_word(cpu.registers.sp), 0xC002);
    }

    #[test]
    fn opcode_cd() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xD000;
        memory.write_word(0xC000, 0xC500);
        cpu.execute_opcode(0xcd, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC500);
        assert_eq!(cpu.registers.sp, 0xCFFE);
        assert_eq!(memory.read_word(cpu.registers.sp), 0xC002);
    }

    #[test]
    fn opcode_ce() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0x10;
        memory.write_byte(0xC000, 0x20);
        cpu.registers.f.insert(Flag::FULL_CARRY);
        cpu.execute_opcode(0xce, &mut memory);

        assert_eq!(cpu.registers.a, 0x31);
    }

    #[test]
    fn opcode_cf() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xD000;
        cpu.execute_opcode(0xcf, &mut memory);

        assert_eq!(cpu.registers.pc, 0x08);
        assert_eq!(cpu.registers.sp, 0xCFFE);
        assert_eq!(memory.read_word(cpu.registers.sp), 0xC000);
    }

    #[test]
    fn opcode_d0() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0xD000;
        memory.write_word(cpu.registers.sp, 0xC000);
        cpu.execute_opcode(0xd0, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC000);
        assert_eq!(cpu.registers.sp, 0xD002);
    }

    #[test]
    fn opcode_d1() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.sp = 0xD000;
        memory.write_word(0xD000, 0x1234);
        cpu.execute_opcode(0xd1, &mut memory);

        assert_eq!(cpu.registers.get_de(), 0x1234);
        assert_eq!(cpu.registers.sp, 0xD002);
    }

    #[test]
    fn opcode_d2() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        memory.write_word(0xC000, 0xD000);
        cpu.execute_opcode(0xd2, &mut memory);

        assert_eq!(cpu.registers.pc, 0xD000);
    }

    #[test]
    #[should_panic]
    fn opcode_d3() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.execute_opcode(0xd3, &mut memory);
    }

    #[test]
    fn opcode_d4() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xD000;
        memory.write_word(0xC000, 0xC500);
        cpu.execute_opcode(0xd4, &mut memory);

        assert_eq!(cpu.registers.pc, 0xC500);
        assert_eq!(cpu.registers.sp, 0xCFFE);
        assert_eq!(memory.read_word(cpu.registers.sp), 0xC002);
    }

    #[test]
    fn opcode_d5() {
        let mut memory = Memory::new();
        let mut cpu = Cpu::new();

        cpu.registers.set_de(0x1234);
        cpu.registers.sp = 0xC002;
        cpu.execute_opcode(0xd5, &mut memory);

        assert_eq!(memory.read_word(0xC000), 0x1234);
        assert_eq!(cpu.registers.sp, 0xC000);
    }

    #[test]
    fn opcode_d6() {}
}
