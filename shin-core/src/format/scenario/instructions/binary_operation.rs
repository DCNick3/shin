use std::io;

use binrw::{BinRead, BinResult, BinWrite, Endian};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::format::scenario::instruction_elements::{NumberSpec, Register, UntypedNumberSpec};

/// An operation on two numbers
///
/// See [super::ExpressionTerm] for details on how the numbers are interpreted and functions used to describe operations
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum BinaryOperationType {
    /// `R`: Ignore the left operand and return the right operand
    MovRight = 0x00,
    /// `0`: Ignore both operands and return 0
    Zero = 0x01,
    /// `L + R`: Add the left and right operands
    Add = 0x02,
    /// `L - R`: Subtract the right operand from the left operand
    Subtract = 0x03,
    /// `L * R`: Multiply the left and right operands
    Multiply = 0x04,
    /// `L / R`: Integer divide the left operand by the right operand
    Divide = 0x05,
    /// `L mod R`: Return the modulo of the left operand divided by the right operand
    ///
    /// Modulo is defined as `L - R * floor(L / R)`
    Modulo = 0x06,
    /// `L & R`: Bitwise AND the left and right operands
    BitwiseAnd = 0x07,
    /// `L | R`: Bitwise OR the left and right operands
    BitwiseOr = 0x08,
    /// `L ^ R`: Bitwise XOR the left and right operands
    BitwiseXor = 0x09,
    /// `L << R`: Shift the left operand left by the right operand
    LeftShift = 0x0a,
    /// `L >> R`: Shift the left operand right by the right operand
    RightShift = 0x0b,
    /// `unreal(real(L) * real(R))`: Add the left and right operands as real numbers
    ///
    /// Real numbers are represented as fixed point numbers with 3 decimal places. (e.g. `1.234` is represented as `1234`)
    MultiplyReal = 0x0c,
    /// `unreal(real(L) / real(R))`: Divide the left operand by the right operand as real numbers
    ///
    /// Real numbers are represented as fixed point numbers with 3 decimal places. (e.g. `1.234` is represented as `1234`)
    DivideReal = 0x0d,
    /// `unangle(atan2(real(L), real(R)))`
    ATan2 = 0x0e,
    /// `L | (1 << R)`: Set the bit at the right operand in the left operand
    SetBit = 0x0f,
    /// `L & ~(1 << R)`: Clear the bit at the right operand in the left operand
    ClearBit = 0x10,
    /// Defined as `ctz((0xffffffff << R) & L)`
    ///
    /// For the love of god, I can't figure out what this is supposed to do.
    ACursedOperation = 0x11,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BinaryOperation {
    pub ty: BinaryOperationType,
    pub destination: Register,
    pub left: NumberSpec,
    pub right: NumberSpec,
}

impl BinRead for BinaryOperation {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,

        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let temp = u8::read_options(reader, endian, ())?;
        let ty = BinaryOperationType::from_u8(temp & 0x7F).ok_or_else(|| {
            binrw::Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown binary operation type: {}", temp & 0x7f),
            ))
        })?;
        let destination = Register::read_options(reader, endian, ())?;
        let left = if temp & 0x80 != 0 {
            NumberSpec::read_options(reader, endian, ())?
        } else {
            NumberSpec::register(destination)
        };
        let right = NumberSpec::read_options(reader, endian, ())?;

        Ok(Self {
            ty,
            left,
            right,
            destination,
        })
    }
}

impl BinWrite for BinaryOperation {
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        let separate_lhs =
            self.left.into_untyped() != UntypedNumberSpec::Register(self.destination);

        let t = self.ty.to_u8().unwrap();
        assert_eq!(t & 0x7f, t);

        let t = t | if separate_lhs { 0x80 } else { 0 };
        t.write_options(writer, endian, ())?;

        self.destination.write_options(writer, endian, ())?;
        if separate_lhs {
            self.left.write_options(writer, endian, ())?;
        }
        self.right.write_options(writer, endian, ())?;

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::{BinaryOperation, BinaryOperationType};
    use crate::format::{
        scenario::instruction_elements::NumberSpec, test_util::assert_enc_dec_pair,
    };

    #[test]
    fn enc_dec() {
        assert_enc_dec_pair(
            &BinaryOperation {
                ty: BinaryOperationType::MovRight,
                destination: "$v0".parse().unwrap(),
                left: NumberSpec::register("$v0".parse().unwrap()),
                right: NumberSpec::constant(0),
            },
            "00000000",
        );
        assert_enc_dec_pair(
            &BinaryOperation {
                ty: BinaryOperationType::Zero,
                destination: "$v1".parse().unwrap(),
                left: NumberSpec::register("$v1".parse().unwrap()),
                right: NumberSpec::constant(42),
            },
            "0101002a",
        );
        assert_enc_dec_pair(
            &BinaryOperation {
                ty: BinaryOperationType::Multiply,
                destination: "$v1".parse().unwrap(),
                left: NumberSpec::constant(2),
                right: NumberSpec::constant(42),
            },
            "840100022a",
        );
    }
}
