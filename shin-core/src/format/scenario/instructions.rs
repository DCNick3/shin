use crate::format::scenario::types::{Pad4, U16SmallList, U8SmallList, U8SmallNumberList};
use crate::vm::command::CompiletimeCommand;
use binrw::{BinRead, BinResult, BinWrite, Endian};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use smallvec::SmallVec;
use std::fmt::{Debug, Display};
use std::io;
use std::io::SeekFrom;

/// Code address - offset into the scenario file
#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[brw(little)]
pub struct CodeAddress(pub u32);

impl Debug for CodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}j", self.0)
    }
}

impl Display for CodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Memory address in the VM
///
/// It can refer to the global memory (for values smaller than [`MemoryAddress::STACK_ADDR_START`]) or to the stack
#[derive(BinRead, BinWrite, Copy, Clone)]
#[brw(little)]
pub struct MemoryAddress(u16);

impl Debug for MemoryAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(offset) = self.as_stack_offset() {
            write!(f, "stack[{}]", offset)
        } else {
            write!(f, "0x{:x}", self.0)
        }
    }
}

impl MemoryAddress {
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

/// Specifies how to get a 32-bit signed number at runtime
///
/// It can be a constant or a reference to memory
///
/// [FromVmCtx](crate::vm::FromVmCtx) trait is used to convert it to runtime representation in command definitions (see [crate::vm::command])
#[derive(Debug, Copy, Clone)]
pub enum NumberSpec {
    /// A constant number
    Constant(i32),
    Memory(MemoryAddress),
}

impl BinRead for NumberSpec {
    type Args<'a> = ();

    //noinspection SpellCheckingInspection
    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let t = u8::read_options(reader, endian, ())?;
        // t=TXXXXXXX
        // T=0 => XXXXXXX is a 7-bit signed constant
        // T=1 => futher processing needed
        Ok(if t & 0x80 != 0 {
            // t=1PPPKKKK
            let p = (t & 0x70) >> 4;
            let k = t & 0x0F;
            // does the sign extension of k, using bits [0:3] (4 bit number)
            let k_sext = (k as i32) << 28 >> 28;
            // P=0 => 12-bit signed constant (KKKK denotes the upper 4 bits, lsb is read from the next byte)
            // P=1 => 20-bit signed constant (KKKK denotes the upper 4 bits, 2 lower bytes are read from the stream)
            // P=2 => 26-bit signed constante (KKKK denotes the upper 4 bits, 3 lower bytes are read from the stream)
            // P=3 => 4-bit Mem1 address, KKKK is the address
            // P=4 => 12-bit Mem1 address, KKKK denotes the upper 4 bits, lsb is read from the next byte
            // P=5 => 4-bit Mem3 address, KKKK + 1 is the address
            match p {
                0 => Self::Constant(u8::read_options(reader, endian, ())? as i32 | (k_sext << 8)),
                1 => {
                    // it's big endian......
                    let b1 = u8::read_options(reader, endian, ())? as i32;
                    let b2 = u8::read_options(reader, endian, ())? as i32;
                    Self::Constant(b2 | (b1 << 8) | (k_sext << 16))
                }
                2 => {
                    // it's big endian......
                    let b1 = u8::read_options(reader, endian, ())? as i32;
                    let b2 = u8::read_options(reader, endian, ())? as i32;
                    let b3 = u8::read_options(reader, endian, ())? as i32;
                    Self::Constant(b3 | (b2 << 8) | (b1 << 16) | (k_sext << 24))
                }
                3 => Self::Memory(MemoryAddress::from_memory_addr(k as u16)),
                4 => Self::Memory(MemoryAddress::from_memory_addr(
                    u8::read_options(reader, endian, ())? as u16 | (k as u16) << 8,
                )),
                5 => Self::Memory(MemoryAddress::from_stack_offset(k as u16 + 1)),
                _ => unreachable!("unknown number spec type: P={}", p),
            }
        } else {
            // signed 7-bit integer
            // does the sign extension of t, using bits [0:6]
            let res = (t as i32 & 0x7f) << 25 >> 25;
            Self::Constant(res)
        })
    }
}

impl BinWrite for NumberSpec {
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

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum UnaryOperationType {
    /// Ignore the source and return 0
    Zero = 0,
    /// Xor the input with 0xFFFF
    XorFFFF = 1,
    /// Negate the input
    Negate = 2,
    /// Take the absolute value of the input
    Abs = 3,
}
#[derive(Debug, Copy, Clone)]
pub struct UnaryOperation {
    pub ty: UnaryOperationType,
    /// Where to write the result to
    pub destination: MemoryAddress,
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
        let destination = MemoryAddress::read_options(reader, endian, ())?;
        let source = if temp & 0x80 != 0 {
            NumberSpec::read_options(reader, endian, ())?
        } else {
            NumberSpec::Memory(destination)
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

/// An operation on two numbers
///
/// See [ExpressionTerm] for details on how the numbers are interpreted and functions used to describe operations
#[derive(Debug, Copy, Clone, FromPrimitive)]
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
#[derive(Debug, Copy, Clone)]
pub struct BinaryOperation {
    pub ty: BinaryOperationType,
    pub destination: MemoryAddress,
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
        let destination = MemoryAddress::read_options(reader, endian, ())?;
        let left = if temp & 0x80 != 0 {
            NumberSpec::read_options(reader, endian, ())?
        } else {
            NumberSpec::Memory(destination)
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
        _writer: &mut W,
        _endian: Endian,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

#[derive(FromPrimitive, Copy, Clone, Debug)]
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
#[derive(Debug, Copy, Clone)]
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
        _writer: &mut W,
        _endian: Endian,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

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
#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
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
#[derive(Debug, Clone)]
pub struct Expression(pub SmallVec<[ExpressionTerm; 6]>);

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

/// Represents 8 numbers, each of which may or may not be present.
///
/// If the number is not present, it is represented as `NumberSpec::Constant(0)`.
#[derive(Debug, Copy, Clone)]
pub struct BitmaskNumberArray(pub [NumberSpec; 8]);

impl BinRead for BitmaskNumberArray {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,

        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let mut res = [NumberSpec::Constant(0); 8];
        let mut mask = u8::read_options(reader, endian, ())?;
        for res in res.iter_mut() {
            if mask & 1 != 0 {
                *res = NumberSpec::read_options(reader, endian, ())?;
            }
            mask >>= 1;
        }
        Ok(Self(res))
    }
}

impl BinWrite for BitmaskNumberArray {
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

/// Message ID - a 24-bit integer
///
/// It is used to check whether a message was seen before.
pub struct MessageId(pub u32);

impl BinRead for MessageId {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,

        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        // MessageId is a 24-bit (sic!) integer
        let b0 = u8::read_options(reader, endian, ())?;
        let b1 = u8::read_options(reader, endian, ())?;
        let b2 = u8::read_options(reader, endian, ())?;

        let id = (b0 as u32) | ((b1 as u32) << 8) | ((b2 as u32) << 16);

        Ok(Self(id))
    }
}

impl BinWrite for MessageId {
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

impl Debug for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an instruction read from a script.
#[allow(non_camel_case_types)]
#[derive(BinRead, BinWrite, Debug)]
#[br(little)]
pub enum Instruction {
    /// Unary operation
    ///
    /// Loads one argument, computes a single result and stores the result at the destination address.
    #[brw(magic(0x40u8))]
    uo(UnaryOperation),
    /// Binary operation
    ///
    /// Loads two arguments, computes a single result and stores the result at the destination address.
    #[brw(magic(0x41u8))]
    bo(BinaryOperation),

    /// Complex expression
    ///
    /// This can load multiple arguments, compute a single result and store the result at the destination address.
    ///
    /// The expression itself is encoded as a reverse polish notation expression.
    #[brw(magic(0x42u8))]
    exp {
        dest: MemoryAddress,
        expr: Expression,
    },

    /// Get Table
    ///
    /// This selects a number from a table based on the value of the index and stores the result at the destination address.
    #[brw(magic(0x44u8))]
    gt {
        dest: MemoryAddress,
        index: NumberSpec,
        // TODO: this encoding is wrong! Each element in the table is padded to 4 bytes!
        table: U16SmallList<[Pad4<NumberSpec>; 32]>,
    },
    /// Jump Conditional
    ///
    /// Compares two numbers and jumps to a target address if the condition is true.
    #[brw(magic(0x46u8))]
    jc {
        cond: JumpCond,
        left: NumberSpec,
        right: NumberSpec,
        target: CodeAddress,
    },

    /// Jump Unconditional
    #[brw(magic(0x47u8))]
    j { target: CodeAddress },
    // ShinDataUtil is using names "call" and "return" for opcodes 0x48 and 0x49
    // while this is kinda true, there are instructions that are much more like "call" and "return"
    // I think I will rename these to gosub or smth, because they do not pass any parameters
    // (Higurashi does not use mem3 aka data stack at all, maybe because the script was converted)
    /// Call a Subroutine without Parameters (legacy call?)
    ///
    /// It appears that this is the older way of calling functions (before the introduction of [call](Instruction::call)).
    ///
    /// The umi scenario still uses this (a bit).
    ///
    /// The return must be done with [retsub](Instruction::retsub).
    #[brw(magic(0x48u8))]
    gosub { target: CodeAddress },
    /// Return from a Subroutine called with [gosub](Instruction::gosub)
    #[brw(magic(0x49u8))]
    retsub {},
    /// Jump via Table
    ///
    /// Jump to a target address based on the value of the index.
    #[brw(magic(0x4au8))]
    jt {
        index: NumberSpec,
        table: U16SmallList<[CodeAddress; 32]>,
    },
    // 0x4b not implemented
    /// Generate a random number between min and max (inclusive)
    #[brw(magic(0x4cu8))]
    rnd {
        dest: MemoryAddress,
        min: NumberSpec,
        max: NumberSpec,
    },
    /// Push Values to call stack
    ///
    /// Used to preserve values of memory in the function. Must be restored with [pop](Instruction::pop) before using [return](`Instruction::return`) or [retsub](Instruction::retsub)
    #[brw(magic(0x4du8))]
    push { values: U8SmallNumberList },
    /// Pop Values from call stack
    ///
    /// Used to restore values of memory previously pushed by [push](Instruction::push)
    #[brw(magic(0x4eu8))]
    pop {
        dest: U8SmallList<[MemoryAddress; 6]>,
    },
    /// Call Subroutine with Parameters
    ///
    /// The return must be done with [return](`Instruction::return`).
    #[brw(magic(0x4fu8))]
    call {
        target: CodeAddress,
        args: U8SmallNumberList,
    },
    /// Return from Subroutine called with [call](Instruction::call)
    #[brw(magic(0x50u8))]
    r#return {},

    /// Send command to the game engine
    Command(CompiletimeCommand),
}
