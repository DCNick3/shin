use std::io;

use binrw::{BinRead, BinResult, BinWrite, Endian};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::format::scenario::instruction_elements::{NumberSpec, Register, UntypedNumberSpec};

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub enum UnaryOperationType {
    /// Ignore the source and return 0
    Zero = 0,
    /// Xor the input with 0xFFFF
    Not16 = 1,
    /// Negate the input
    Negate = 2,
    /// Take the absolute value of the input
    Abs = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnaryOperation {
    pub ty: UnaryOperationType,
    /// Where to write the result to
    pub destination: Register,
    /// The input value
    pub source: NumberSpec,
}

impl BinRead for UnaryOperation {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,

        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let temp = u8::read_options(reader, endian, ())?;
        let ty = UnaryOperationType::from_u8(temp & 0x7f).ok_or_else(|| {
            binrw::Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown binary operation type: {}", temp & 0x7f),
            ))
        })?;
        let destination = Register::read_options(reader, endian, ())?;
        let source = if temp & 0x80 != 0 {
            NumberSpec::read_options(reader, endian, ())?
        } else {
            NumberSpec::new(UntypedNumberSpec::Register(destination))
        };
        Ok(Self {
            ty,
            source,
            destination,
        })
    }
}

impl BinWrite for UnaryOperation {
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _endian: Endian,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}
