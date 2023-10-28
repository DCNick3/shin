use std::fmt::{Debug, Display};

use binrw::{BinRead, BinWrite};

/// Code address - offset into the scenario file
#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[brw(little)]
pub struct CodeAddress(pub u32);

impl Debug for CodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.0)
    }
}

impl Display for CodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
