use crate::mmu::Memory;
use crate::mmu::interrupt::Interrupt;

pub trait ByteTransfer {

    fn transfer(&mut self, cs: i32, sd: u8, sc: u8) -> Option<(u8, u8)>;

    fn disconnected(&self) -> bool;

    fn update(&mut self, cycles: i32, mmu: &mut Memory) {
        let result = self.transfer(
            cycles,
            mmu.read_byte(0xFF01),
            mmu.read_byte(0xFF02)
        );

        if self.disconnected() {
            mmu.write_byte(0xFF01, 0xFF);
        }

        if let Some((data, control)) = result {
            mmu.write_byte(0xFF01, data);
            mmu.write_byte(0xFF02, control);

            mmu.request_interrupt(Interrupt::Serial);
        }
    }
}

pub struct Unlinked;

impl ByteTransfer for Unlinked {
    fn transfer(&mut self, _: i32, _: u8, _: u8) -> Option<(u8, u8)> {
        None
    }

    fn disconnected(&self) -> bool {
        true
    }

    // update is sealed in crate external impls
    fn update(&mut self, _cycles: i32, _mmu: &mut Memory) {
    }
}
