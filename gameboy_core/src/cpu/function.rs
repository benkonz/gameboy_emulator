use mmu::Memory;
use super::registers::Registers;
use super::stack;

pub fn call_nn(registers: &mut Registers, nn: u16, memory: &mut Memory) {
    stack::push(&mut registers.sp, registers.pc, memory);
    registers.pc = nn;
}

pub fn ret(registers: &mut Registers, memory: &Memory) {
    registers.pc = stack::pop(&mut registers.sp, memory);
}

#[cfg(test)]
mod tests {
    use cpu::registers::Registers;
    use cpu::function;
    use mmu::Memory;

    #[test]
    fn test_call_nn() {
        let mut memory = Memory::new();
        let mut registers: Registers = Default::default();
        registers.pc = 0xAAAA;
        registers.sp = 0xFF00;

        function::call_nn(&mut registers, 0xBBBB, &mut memory);

        assert_eq!(registers.pc, 0xBBBB);
        assert_eq!(registers.sp, 0xFEFE);
        assert_eq!(memory.read_word(registers.sp), 0xAAAA);
    }

    #[test]
    fn test_ret() {
        let mut memory = Memory::new();
        let mut registers : Registers = Default::default();
        registers.pc = 0xAAAA;
        registers.sp = 0xFF00;

        function::call_nn(&mut registers, 0xBBBB, &mut memory);
        function::ret(&mut registers, &memory);

        assert_eq!(registers.pc, 0xAAAA);
        assert_eq!(registers.sp, 0xFF00);
    }
}