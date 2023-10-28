use std::{io, io::SeekFrom};

use binrw::{BinRead, BinResult, BinWrite, Endian};
use smallvec::SmallVec;

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

/// An expression is a sequence of terms that are evaluated in order.
/// This is basically a reverse polish notation expression, which can be evaluated with a stack machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression(pub SmallVec<ExpressionTerm, 6>);

impl BinRead for Expression {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,

        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let mut res = SmallVec::new();
        loop {
            let v = u8::read_options(reader, endian, ())?;
            if v == 0xff {
                break;
            } else {
                reader.seek(SeekFrom::Current(-1))?;
                res.push(ExpressionTerm::read_options(reader, endian, ())?);
            }
        }
        Ok(Self(res))
    }
}

impl BinWrite for Expression {
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
