use std::io;

use binrw::{BinRead, BinResult, BinWrite, Endian};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum JumpCondType {
    /// `L == R`
    Equal = 0x0,
    /// `L != R`
    NotEqual = 0x1,
    /// `L >= R`
    GreaterOrEqual = 0x2,
    /// `L > R`
    Greater = 0x3,
    /// `L <= R`
    LessOrEqual = 0x4,
    /// `L < R`
    Less = 0x5,
    /// `L & R != 0`
    BitwiseAndNotZero = 0x6,
    /// `L & (1 << R) != 0`
    BitSet = 0x7,
}

/// Jump condition
///
/// Describes how to get a boolean value from two numbers
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct JumpCond {
    /// If true, the condition is negated
    pub is_negated: bool,
    /// The condition to check
    pub condition: JumpCondType,
}

impl BinRead for JumpCond {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let temp = u8::read_options(reader, endian, ())?;
        let is_negated = temp & 0x80 != 0;
        let condition = temp & 0x7F;
        let condition = JumpCondType::from_u8(condition).ok_or_else(|| {
            binrw::Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown jump condition type: {}", condition),
            ))
        })?;

        Ok(Self {
            is_negated,
            condition,
        })
    }
}

impl BinWrite for JumpCond {
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        let t = self.condition.to_u8().unwrap();
        assert_eq!(t & 0x7f, t);
        let t = t | if self.is_negated { 0x80 } else { 0 };

        t.write_options(writer, endian, ())
    }
}

#[cfg(test)]
mod tests {
    use super::{JumpCond, JumpCondType};
    use crate::format::test_util::assert_enc_dec_pair;

    #[test]
    fn enc_dec() {
        assert_enc_dec_pair(
            &JumpCond {
                condition: JumpCondType::Equal,
                is_negated: false,
            },
            "00",
        );
        assert_enc_dec_pair(
            &JumpCond {
                condition: JumpCondType::Equal,
                is_negated: true,
            },
            "80",
        );
        assert_enc_dec_pair(
            &JumpCond {
                condition: JumpCondType::BitSet,
                is_negated: false,
            },
            "07",
        );
        assert_enc_dec_pair(
            &JumpCond {
                condition: JumpCondType::BitSet,
                is_negated: true,
            },
            "87",
        );
    }
}
