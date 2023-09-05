use binrw::{BinRead, BinWrite};

mod register;

pub use register::{Register, RegisterRepr};

pub trait InstructionElement: BinRead + BinWrite {}
