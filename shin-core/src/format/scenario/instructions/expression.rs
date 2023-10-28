use std::io;

use binrw::{BinRead, BinResult, BinWrite, Endian};
use smallvec::SmallVec;
use snafu::Snafu;

use crate::format::scenario::instruction_elements::NumberSpec;

/// A single term in an expression. Represents a single operation on a stack machine
///
/// The stack works primarily with integers, however some operations can re-interpret them as other types.
///
/// Boolean values are represented as integers, with `0` being `false` and anything else being `true`, with `-1` being the preferred representation of `true`.
///
/// Real numbers are represented as fixed point numbers with 3 decimal places. (e.g. `1.234` is represented as `1234`)
///
/// Angles are represented as real numbers of turns, with `1.0` being a full turn, `0.5` being half a turn, etc.
///
/// Notation for the ops:
/// - pop(): Pop an integer from the stack
/// - push(x): Push an integer onto the stack
/// - real(x): Convert fixed-point integer to a real number (e.g. `1234` -> `1.234`)
/// - unreal(x): Convert real number to a fixed-point integer (e.g. `1.234` -> `1234`)
/// - bool(x): Convert integer to boolean (e.g. `0` -> `false`, else -> `true`)
/// - unbool(x): Convert boolean to integer (e.g. `false` -> `0`, `true` -> `-1` (sic!))
/// - angle(x): Convert fixed-point integer to angle in radians (e.g. `1000` -> 2pi)
/// - unangle(x): Convert angle in radians to fixed-point integer (e.g. 2pi -> `1000`)
#[derive(BinRead, BinWrite, Copy, Clone, Debug, PartialEq, Eq)]
#[brw(little)]
pub enum ExpressionTerm {
    /// Push a number onto the stack
    #[brw(magic(0x00u8))]
    Push(NumberSpec),
    /// `R=pop(), L=pop(), push(L + R)`
    #[brw(magic(0x01u8))]
    Add,
    /// `R=pop(), L=pop(), push(L - R)` TODO: is the order reversed?
    #[brw(magic(0x02u8))]
    Subtract,
    /// `R=pop(), L=pop(), push(L * R)`
    #[brw(magic(0x03u8))]
    Multiply,
    /// `R=pop(), L=pop(), push(L / R)`
    #[brw(magic(0x04u8))]
    Divide,
    /// `R=pop(), L=pop(), push(L mod R)`
    #[brw(magic(0x05u8))]
    Modulo,
    /// `R=pop(), L=pop(), push(L << R)`
    #[brw(magic(0x06u8))]
    ShiftLeft,
    /// `R=pop(), L=pop(), push(L >> R)`
    #[brw(magic(0x07u8))]
    ShiftRight,
    /// `R=pop(), L=pop(), push(L & R)`
    #[brw(magic(0x08u8))]
    BitwiseAnd,
    /// `R=pop(), L=pop(), push(L | R)`
    #[brw(magic(0x09u8))]
    BitwiseOr,
    /// `R=pop(), L=pop(), push(L ^ R)`
    #[brw(magic(0x0au8))]
    BitwiseXor,
    /// `V=pop(), push(-V)`
    #[brw(magic(0x0bu8))]
    Negate,
    /// `V=pop(), push(~V)`
    #[brw(magic(0x0cu8))]
    BitwiseNot,
    /// `V=pop(), push(abs(V))`
    #[brw(magic(0x0du8))]
    Abs,
    /// `R=pop(), L=pop(), push(unbool(L == R))`
    #[brw(magic(0x0eu8))]
    CmpEqual,
    /// `R=pop(), L=pop(), push(unbool(L != R))`
    #[brw(magic(0x0fu8))]
    CmpNotEqual,
    /// `R=pop(), L=pop(), push(unbool(L >= R))`
    #[brw(magic(0x10u8))]
    CmpGreaterOrEqual,
    /// `R=pop(), L=pop(), push(unbool(L > R))`
    #[brw(magic(0x11u8))]
    CmpGreater,
    /// `R=pop(), L=pop(), push(unbool(L <= R))`
    #[brw(magic(0x12u8))]
    CmpLessOrEqual,
    /// `R=pop(), L=pop(), push(unbool(L < R))`
    #[brw(magic(0x13u8))]
    CmpLess,
    /// `V=pop(), push(unbool(V == 0))`
    #[brw(magic(0x14u8))]
    CmpZero,
    /// `V=pop(), push(unbool(V != 0))`
    #[brw(magic(0x15u8))]
    CmpNotZero,
    /// `R=pop(), L=pop(), push(unbool(bool(L) && bool(R)))`
    #[brw(magic(0x16u8))]
    LogicalAnd,
    /// `R=pop(), L=pop(), push(unbool(bool(L) || bool(R)))`
    #[brw(magic(0x17u8))]
    LogicalOr,
    /// `C=pop(), T=pop(), F=pop(), push(if bool(C) { T } else { F })`
    #[brw(magic(0x18u8))]
    Select,
    /// `R=pop(), L=pop(), push(real(L) * real(R))`
    #[brw(magic(0x19u8))]
    MultiplyReal,
    /// `R=pop(), L=pop(), push(real(L) / real(R))`
    #[brw(magic(0x1au8))]
    DivideReal,
    /// `A=pop(), push(sin(angle(A)))`
    #[brw(magic(0x1bu8))]
    Sin,
    /// `A=pop(), push(cos(angle(A)))`
    #[brw(magic(0x1cu8))]
    Cos,
    /// `A=pop(), push(tan(angle(A)))`
    #[brw(magic(0x1du8))]
    Tan,
    /// `R=pop(), L=pop(), push(min(L, R))`
    #[brw(magic(0x1eu8))]
    Min,
    /// `R=pop(), L=pop(), push(max(L, R))`
    #[brw(magic(0x1fu8))]
    Max,
}

impl ExpressionTerm {
    pub fn argument_count(&self) -> usize {
        match self {
            ExpressionTerm::Push(_) => 0,
            ExpressionTerm::Add => 2,
            ExpressionTerm::Subtract => 2,
            ExpressionTerm::Multiply => 2,
            ExpressionTerm::Divide => 2,
            ExpressionTerm::Modulo => 2,
            ExpressionTerm::ShiftLeft => 2,
            ExpressionTerm::ShiftRight => 2,
            ExpressionTerm::BitwiseAnd => 2,
            ExpressionTerm::BitwiseOr => 2,
            ExpressionTerm::BitwiseXor => 2,
            ExpressionTerm::Negate => 1,
            ExpressionTerm::BitwiseNot => 1,
            ExpressionTerm::Abs => 1,
            ExpressionTerm::CmpEqual => 2,
            ExpressionTerm::CmpNotEqual => 2,
            ExpressionTerm::CmpGreaterOrEqual => 2,
            ExpressionTerm::CmpGreater => 2,
            ExpressionTerm::CmpLessOrEqual => 2,
            ExpressionTerm::CmpLess => 2,
            ExpressionTerm::CmpZero => 1,
            ExpressionTerm::CmpNotZero => 1,
            ExpressionTerm::LogicalAnd => 2,
            ExpressionTerm::LogicalOr => 2,
            ExpressionTerm::Select => 3,
            ExpressionTerm::MultiplyReal => 2,
            ExpressionTerm::DivideReal => 2,
            ExpressionTerm::Sin => 1,
            ExpressionTerm::Cos => 1,
            ExpressionTerm::Tan => 1,
            ExpressionTerm::Min => 2,
            ExpressionTerm::Max => 2,
        }
    }
}

#[derive(BinRead, BinWrite, Copy, Clone, Debug, PartialEq, Eq)]
enum ExpressionTermOpt {
    #[brw(magic(0xffu8))]
    None,
    Some(ExpressionTerm),
}

#[derive(Debug, Snafu)]
pub enum ExpressionValidationError {
    /// The expression underflows the stack at term `{pos}`
    StackUnderflow { pos: usize },
    /// The expression doesn't leave a single value on the stack after evaluation (it leaves `{actual}`)
    NotSingleValue { actual: usize },
}

/// An expression is a sequence of terms that are evaluated in order.
/// This is basically a reverse polish notation expression, which can be evaluated with a stack machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression(SmallVec<ExpressionTerm, 6>);

impl Expression {
    pub fn new_unchecked<I: IntoIterator<Item = ExpressionTerm>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }

    pub fn new<I: IntoIterator<Item = ExpressionTerm>>(
        iter: I,
    ) -> Result<Self, ExpressionValidationError> {
        let res = Self::new_unchecked(iter);
        res.validate()?;
        Ok(res)
    }

    pub fn validate(&self) -> Result<(), ExpressionValidationError> {
        // TODO: we can also probably do some type-checking for the expression?
        // though, it's probably better to do it in the `shin-asm`
        let mut stack = 0;
        for (pos, term) in self.0.iter().enumerate() {
            stack -= term.argument_count() as isize;
            stack += 1;
            if stack < 0 {
                return Err(ExpressionValidationError::StackUnderflow { pos });
            }
        }

        if stack != 1 {
            return Err(ExpressionValidationError::NotSingleValue {
                actual: stack as usize,
            });
        }

        Ok(())
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ExpressionTerm> {
        self.0.iter()
    }
}

impl BinRead for Expression {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let pos = reader.stream_position()?;

        let mut res = SmallVec::new();
        loop {
            match ExpressionTermOpt::read_options(reader, endian, ())? {
                ExpressionTermOpt::None => break,
                ExpressionTermOpt::Some(expr) => res.push(expr),
            }
        }
        let res = Self(res);

        res.validate().map_err(|e| binrw::Error::Custom {
            pos,
            err: Box::new(e),
        })?;

        Ok(res)
    }
}

impl BinWrite for Expression {
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        let pos = writer.stream_position()?;
        self.validate().map_err(|e| binrw::Error::Custom {
            pos,
            err: Box::new(e),
        })?;

        for term in &self.0 {
            term.write_options(writer, endian, ())?;
        }
        0xffu8.write_options(writer, endian, ())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use binrw::{io::NoSeek, BinWrite};

    use super::{Expression, ExpressionTerm, NumberSpec};
    use crate::format::{
        scenario::instructions::expression::ExpressionValidationError,
        test_util::assert_enc_dec_pair,
    };

    #[test]
    fn enc_dec() {
        assert_enc_dec_pair(
            &Expression::new_unchecked([ExpressionTerm::Push(NumberSpec::constant(42))]),
            "002aff",
        );
        assert_enc_dec_pair(
            &Expression::new_unchecked([
                ExpressionTerm::Push(NumberSpec::constant(1)),
                ExpressionTerm::Push(NumberSpec::constant(2)),
                ExpressionTerm::Add,
            ]),
            "0001000201ff",
        );
        assert_enc_dec_pair(
            &Expression::new_unchecked([
                ExpressionTerm::Push(NumberSpec::constant(1)),
                ExpressionTerm::Push(NumberSpec::constant(2)),
                ExpressionTerm::Push(NumberSpec::constant(3)),
                ExpressionTerm::Select,
            ]),
            "00010002000318ff",
        );
    }

    #[test]
    fn enc_invalid() {
        match Expression::new_unchecked([])
            .write_le(&mut NoSeek::new(Vec::new()))
            .unwrap_err()
        {
            binrw::Error::Custom { err, .. } => match err.downcast::<ExpressionValidationError>() {
                Ok(e) => match e.as_ref() {
                    ExpressionValidationError::NotSingleValue { actual: 0 } => {}
                    e => panic!("unexpected error: {:?}", e),
                },
                Err(e) => panic!("unexpected error: {:?}", e),
            },
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
