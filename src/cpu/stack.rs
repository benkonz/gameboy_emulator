use mmu::Memory;

pub fn push(sp: &mut u16, nn: u16, memory: &mut Memory) {
    *sp -= 2;
    memory.write_word(*sp, nn);
}

pub fn pop(sp: &mut u16, memory: &Memory) -> u16 {
    let word = memory.read_word(*sp);
    *sp += 2;

    word
}

#[cfg(test)]
mod tests {
    use mmu::Memory;
    use cpu::stack;

    #[test]
    fn test_push_nn() {
        let mut memory = Memory::new();
        let mut sp = 0xFF00;

        stack::push(&mut sp, 0xFFFF, &mut memory);

        assert_eq!(memory.read_word(sp), 0xFFFF);
        assert_eq!(sp, 0xFEFE);
    }

    #[test]
    fn test_pop_nn() {
        let mut memory = Memory::new();
        let mut sp = 0xFEFE;
        stack::push(&mut sp, 0xFFFF, &mut memory);

        let word = stack::pop(&mut sp, &memory);

        assert_eq!(word, 0xFFFF);
        assert_eq!(sp, 0xFEFE);
    }
}