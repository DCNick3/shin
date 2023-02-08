use crate::format::scenario::types::{U16SmallList, U8SmallList, U8SmallNumberList};
use crate::vm::command::CompiletimeCommand;
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use smallvec::SmallVec;
use std::fmt::Debug;
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
    type Args = ();

    //noinspection SpellCheckingInspection
    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let t = u8::read_options(reader, options, ())?;
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
                0 => Self::Constant(u8::read_options(reader, options, ())? as i32 | (k_sext << 8)),
                1 => {
                    // it's big endian......
                    let b1 = u8::read_options(reader, options, ())? as i32;
                    let b2 = u8::read_options(reader, options, ())? as i32;
                    Self::Constant(b2 | (b1 << 8) | (k_sext << 16))
                }
                2 => {
                    // it's big endian......
                    let b1 = u8::read_options(reader, options, ())? as i32;
                    let b2 = u8::read_options(reader, options, ())? as i32;
                    let b3 = u8::read_options(reader, options, ())? as i32;
                    Self::Constant(b3 | (b2 << 8) | (b1 << 16) | (k_sext << 24))
                }
                3 => Self::Memory(MemoryAddress::from_memory_addr(k as u16)),
                4 => Self::Memory(MemoryAddress::from_memory_addr(
                    u8::read_options(reader, options, ())? as u16 | (k as u16) << 8,
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
    type Args = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

#[derive(Debug, FromPrimitive)]
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
#[derive(Debug)]
pub struct UnaryOperation {
    pub ty: UnaryOperationType,
    /// Where to write the result to
    pub destination: MemoryAddress,
    /// The input value
    pub source: NumberSpec,
}

impl BinRead for UnaryOperation {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let temp = u8::read_options(reader, options, ())?;
        let ty = UnaryOperationType::from_u8(temp & 0x7f).ok_or_else(|| {
            binrw::Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown binary operation type: {}", temp & 0x7f),
            ))
        })?;
        let destination = MemoryAddress::read_options(reader, options, ())?;
        let source = if temp & 0x80 != 0 {
            NumberSpec::read_options(reader, options, ())?
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
    type Args = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

#[derive(Debug, FromPrimitive)]
pub enum BinaryOperationType {
    /// `R`: Ignore the left operand and return the right operand
    MovRight = 0,
    /// `0`: Ignore both operands and return 0
    Zero = 1,
    /// `L + R`: Add the left and right operands
    Add = 2,
    /// `L - R`: Subtract the right operand from the left operand
    Subtract = 3,
    /// `L * R`: Multiply the left and right operands
    Multiply = 4,
    /// `L / R`: Divide the left operand by the right operand
    Divide = 5,
    /// `L % R`: Return the remainder of the left operand divided by the right operand
    Remainder = 6,
    /// `L & R`: Bitwise AND the left and right operands
    BitwiseAnd = 7,
    /// `L | R`: Bitwise OR the left and right operands
    BitwiseOr = 8,
    /// `L ^ R`: Bitwise XOR the left and right operands
    BitwiseXor = 9,
    /// `L << R`: Shift the left operand left by the right operand
    LeftShift = 10,
    /// `L >> R`: Shift the left operand right by the right operand
    RightShift = 11,
    /// `real(L) * real(R)`: Add the left and right operands as real numbers
    ///
    /// Real numbers are represented as fixed point numbers with 3 decimal places. (e.g. `1.234` is represented as `1234`)
    MultiplyReal = 12,
    /// `real(L) / real(R)`: Divide the left operand by the right operand as real numbers
    ///
    /// Real numbers are represented as fixed point numbers with 3 decimal places. (e.g. `1.234` is represented as `1234`)
    DivideReal = 13,
    // TODO
    // 14: right = (int)(float)((float)(atan2f_0((float)left * 0.001, (float)right * 0.001) * 1000.0) / 6.2832);
    // 15: right = (1 << right) | left;
    // ....
}
#[derive(Debug)]
pub struct BinaryOperation {
    pub ty: BinaryOperationType,
    pub destination: MemoryAddress,
    pub left: NumberSpec,
    pub right: NumberSpec,
}

impl BinRead for BinaryOperation {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let temp = u8::read_options(reader, options, ())?;
        let ty = BinaryOperationType::from_u8(temp & 0x7F).ok_or_else(|| {
            binrw::Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown binary operation type: {}", temp & 0x7f),
            ))
        })?;
        let destination = MemoryAddress::read_options(reader, options, ())?;
        let left = if temp & 0x80 != 0 {
            NumberSpec::read_options(reader, options, ())?
        } else {
            NumberSpec::Memory(destination)
        };
        let right = NumberSpec::read_options(reader, options, ())?;

        Ok(Self {
            ty,
            left,
            right,
            destination,
        })
    }
}

impl BinWrite for BinaryOperation {
    type Args = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

#[derive(FromPrimitive, Debug)]
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
    /// (TODO: fact-check) `L & (1 << R) != 0`
    BitSet = 0x7,
}

/// Jump condition
///
/// Describes how to get a boolean value from two numbers
#[derive(Debug)]
pub struct JumpCond {
    /// If true, the condition is negated
    pub is_negated: bool,
    /// The condition to check
    pub condition: JumpCondType,
}

impl BinRead for JumpCond {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let temp = u8::read_options(reader, options, ())?;
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
    type Args = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub enum ExpressionTerm {
    /// Push a number onto the stack
    #[brw(magic(0x00u8))]
    Push(NumberSpec),
    /// `L=pop(), R=pop(), push(L + R)`
    #[brw(magic(0x01u8))]
    Add,
    /// `L=pop(), R=pop(), push(L - R)`
    #[brw(magic(0x02u8))]
    Subtract,
    /// `L=pop(), R=pop(), push(L * R)`
    #[brw(magic(0x03u8))]
    Multiply,
    /// `L=pop(), R=pop(), push(L / R)`
    #[brw(magic(0x04u8))]
    Divide,
    /// `L=pop(), R=pop(), push(L % R)`
    #[brw(magic(0x05u8))]
    Remainder,

    /// `L=pop(), R=pop(), push(real(L) * real(R))`
    ///
    /// Real numbers are represented as fixed point numbers with 3 decimal places. (e.g. `1.234` is represented as `1234`)
    #[brw(magic(0x1au8))]
    MultiplyReal,

    /// `L=pop(), R=pop(), push(min(L, R))`
    #[brw(magic(0x1eu8))]
    Min,
    /// `L=pop(), R=pop(), push(max(L, R))`
    #[brw(magic(0x1fu8))]
    Max,
}

/// An expression is a sequence of terms that are evaluated in order.
/// This is basically a reverse polish notation expression, which can be evaluated with a stack machine.
#[derive(Debug)]
pub struct Expression(pub SmallVec<[ExpressionTerm; 6]>);

impl BinRead for Expression {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let mut res = SmallVec::new();
        loop {
            let v = u8::read_options(reader, options, ())?;
            if v == 0xff {
                break;
            } else {
                reader.seek(SeekFrom::Current(-1))?;
                res.push(ExpressionTerm::read_options(reader, options, ())?);
            }
        }
        Ok(Self(res))
    }
}

impl BinWrite for Expression {
    type Args = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

/// Represents 8 numbers, each of which may or may not be present.
///
/// If the number is not present, it is represented as `NumberSpec::Constant(0)`.
#[derive(Debug)]
pub struct BitmaskNumberArray(pub [NumberSpec; 8]);

impl BinRead for BitmaskNumberArray {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let mut res = [NumberSpec::Constant(0); 8];
        let mut mask = u8::read_options(reader, options, ())?;
        for res in res.iter_mut() {
            if mask & 1 != 0 {
                *res = NumberSpec::read_options(reader, options, ())?;
            }
            mask >>= 1;
        }
        Ok(Self(res))
    }
}

impl BinWrite for BitmaskNumberArray {
    type Args = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
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
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        // MessageId is a 24-bit (sic!) integer
        let b0 = u8::read_options(reader, options, ())?;
        let b1 = u8::read_options(reader, options, ())?;
        let b2 = u8::read_options(reader, options, ())?;

        let id = (b0 as u32) | ((b1 as u32) << 8) | ((b2 as u32) << 16);

        Ok(Self(id))
    }
}

impl BinWrite for MessageId {
    type Args = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
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
        table: U16SmallList<[NumberSpec; 32]>,
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
