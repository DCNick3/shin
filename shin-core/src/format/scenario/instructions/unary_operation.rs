use std::io;

use binrw::{BinRead, BinResult, BinWrite, Endian};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::format::scenario::instruction_elements::{NumberSpec, Register, UntypedNumberSpec};

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
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
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        let separate_source =
            self.source.into_untyped() != UntypedNumberSpec::Register(self.destination);

        let t = self.ty.to_u8().unwrap();
        assert_eq!(t & 0x7f, t);

        let t = t | if separate_source { 0x80 } else { 0 };
        t.write_options(writer, endian, ())?;

        self.destination.write_options(writer, endian, ())?;
        if separate_source {
            self.source.write_options(writer, endian, ())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use shin_core::format::scenario::instruction_elements::UntypedNumberSpec;

    use super::{UnaryOperation, UnaryOperationType};
    use crate::format::{
        scenario::instruction_elements::NumberSpec, test_util::assert_enc_dec_pair,
    };

    #[test]
    fn enc_dec() {
        assert_enc_dec_pair(
            &UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::new(UntypedNumberSpec::Register("$v0".parse().unwrap())),
            },
            "000000",
        );
        assert_enc_dec_pair(
            &UnaryOperation {
                ty: UnaryOperationType::Not16,
                destination: "$v1".parse().unwrap(),
                source: NumberSpec::new(UntypedNumberSpec::Register("$v1".parse().unwrap())),
            },
            "010100",
        );
        assert_enc_dec_pair(
            &UnaryOperation {
                ty: UnaryOperationType::Not16,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::new(UntypedNumberSpec::Constant(0)),
            },
            "81000000",
        );
        assert_enc_dec_pair(
            &UnaryOperation {
                ty: UnaryOperationType::Abs,
                destination: "$a0".parse().unwrap(),
                source: NumberSpec::new(UntypedNumberSpec::Constant(42)),
            },
            "8300102a",
        );
    }
}
