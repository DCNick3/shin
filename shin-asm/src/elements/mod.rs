use binrw::{BinRead, BinWrite};

mod register;

pub use shin_core::format::scenario::instruction_elements::{Register, RegisterRepr};

pub trait InstructionElement: BinRead + BinWrite {}
