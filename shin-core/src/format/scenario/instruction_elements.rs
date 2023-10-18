use binrw::{BinRead, BinWrite};
use std::fmt::Debug;

/// Register address in the vm
///
/// It can refer to the global memory (for values smaller than [`Register::STACK_ADDR_START`]) or to the stack
#[derive(BinRead, BinWrite, Copy, Clone)]
#[brw(little)]
pub struct Register(u16);

impl Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(offset) = self.as_stack_offset() {
            write!(f, "stack[{}]", offset)
        } else {
            write!(f, "0x{:x}", self.0)
        }
    }
}

impl Register {
    /// Addresses larger than 0x1000 are treated as relative to the stack top (Aka mem3)
    pub const STACK_ADDR_START: u16 = 0x1000;

    pub fn as_stack_offset(&self) -> Option<u16> {
        if self.0 >= Self::STACK_ADDR_START {
            Some(self.raw() - Self::STACK_ADDR_START + 1)
        } else {
            None
        }
    }

    pub fn from_stack_offset(offset: u16) -> Self {
        assert!(offset > 0);
        Self(offset + Self::STACK_ADDR_START - 1)
    }

    pub fn from_memory_addr(addr: u16) -> Self {
        assert!(addr < Self::STACK_ADDR_START);
        Self(addr)
    }

    pub fn raw(&self) -> u16 {
        self.0
    }
}
