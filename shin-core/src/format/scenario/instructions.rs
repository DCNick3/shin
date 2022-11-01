use crate::format::scenario::{U16SmallList, U8SmallList, U8SmallNumberList};
use crate::format::text::read_sjis_string;
use crate::vm::command::CompiletimeCommand;
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use smallvec::SmallVec;
use std::fmt::Debug;
use std::io;
use std::io::SeekFrom;

#[derive(BinRead, BinWrite, Copy, Clone)]
#[brw(little)]
pub struct CodeAddress(pub u32);

impl Debug for CodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}j", self.0)
    }
}

#[derive(BinRead, BinWrite, Copy, Clone)]
#[brw(little)]
pub struct MemoryAddress(pub u16);

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
    /// addresses larger than 0x1000 are treated as relative to the stack top (Aka mem3)
    pub const STACK_ADDR_START: u16 = 0x1000;

    pub fn as_stack_offset(&self) -> Option<u16> {
        if self.0 >= Self::STACK_ADDR_START {
            Some(self.0 - Self::STACK_ADDR_START)
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NumberSpec {
    Constant(i32),
    // technically there are two kinds of memories in the VM...
    // I think one is linear memory and another one is stack (known as Mem1 and Mem3 in ShinDataUtil)
    // but I didn't see stack used much...
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
        // TXXXXXXX
        // T=0 => XXXXXXX is a 7-bit signed constant
        // T=1 => futher processing needed
        Ok(if t & 0x80 != 0 {
            // 1PPPKKKK
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
                    let b1 = u8::read_options(reader, options, ())? as i32;
                    let b2 = u8::read_options(reader, options, ())? as i32;
                    Self::Constant(b1 | (b2 << 8) | (k_sext << 16))
                }
                2 => {
                    let b1 = u8::read_options(reader, options, ())? as i32;
                    let b2 = u8::read_options(reader, options, ())? as i32;
                    let b3 = u8::read_options(reader, options, ())? as i32;
                    Self::Constant(b1 | (b2 << 8) | (b3 << 16) | (k_sext << 24))
                }
                3 => Self::Memory(MemoryAddress(k as u16)),
                4 => Self::Memory(MemoryAddress(
                    u8::read_options(reader, options, ())? as u16 | (k as u16) << 8,
                )),
                5 => Self::Memory(MemoryAddress(
                    MemoryAddress::STACK_ADDR_START + (k as u16 + 1),
                )),
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
    Zero = 0,
    XorFFFF = 1,
    Negate = 2,
    Abs = 3,
}
#[derive(Debug)]
pub struct UnaryOperation {
    pub ty: UnaryOperationType,
    pub destination: MemoryAddress,
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
    MovRight = 0,
    Zero = 1,
    Add = 2,
    Subtract = 3,
    Multiply = 4,
    Divide = 5,
    Remainder = 6,
    BitwiseAnd = 7,
    BitwiseOr = 8,
    BitwiseXor = 9,
    LeftShift = 10,
    RightShift = 11,
    MultiplyReal = 12,
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
    Equal = 0x0,
    NotEqual = 0x01,
    GreaterOrEqual = 0x02,
    Greater = 0x03,
    LessOrEqual = 0x04,
    Less = 0x05,
    BitwiseAndNotZero = 0x06,
    BitSet = 0x7,
}

#[derive(Debug)]
pub struct JumpCond {
    pub is_negated: bool,
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
    #[brw(magic(0x00u8))]
    Push(NumberSpec),
    #[brw(magic(0x01u8))]
    Add,
    #[brw(magic(0x02u8))]
    Subtract,
    #[brw(magic(0x03u8))]
    Multiply,
    #[brw(magic(0x04u8))]
    Divide,
    #[brw(magic(0x05u8))]
    Remainder,

    #[brw(magic(0x1au8))]
    MultiplyReal,

    #[brw(magic(0x1eu8))]
    Min,
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

#[derive(Debug)]
pub struct StringArray(pub SmallVec<[String; 4]>);

impl BinRead for StringArray {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let size = u16::read_options(reader, options, ())?;
        let pos = reader.stream_position()?;
        let mut res = SmallVec::new();
        loop {
            let s = read_sjis_string(reader, None)?;

            res.push(s);

            let v = u8::read_options(reader, options, ())?;
            if v == 0x00 {
                break;
            } else {
                reader.seek(SeekFrom::Current(-1))?;
            }
        }
        let end = reader.stream_position()?;
        let diff = end - pos;
        assert_eq!(diff, size as u64);
        Ok(Self(res))
    }
}

impl BinWrite for StringArray {
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

#[allow(non_camel_case_types)]
#[derive(BinRead, BinWrite, Debug)]
#[br(little)]
pub enum Instruction {
    #[brw(magic(0x40u8))]
    uo(UnaryOperation),
    #[brw(magic(0x41u8))]
    bo(BinaryOperation),
    #[brw(magic(0x42u8))]
    exp {
        dest: MemoryAddress,
        expr: Expression,
    },

    #[brw(magic(0x44u8))]
    gt {
        dest: MemoryAddress,
        value: NumberSpec,
        table: U16SmallList<[NumberSpec; 32]>,
    },
    /// Jump Conditional
    #[brw(magic(0x46u8))]
    jc {
        cond: JumpCond,
        left: NumberSpec,
        right: NumberSpec,
        target: CodeAddress,
    },

    /// Jump Unconditional
    #[brw(magic(0x47u8))]
    j {
        target: CodeAddress,
    },
    // ShinDataUtil is using names "call" and "return" for opcodes 0x48 and 0x49
    // while this is kinda true, there are instructions that are much more like "call" and "return"
    // I think I will rename these to gosub or smth, because they do not pass any parameters
    // (Higurashi does not use mem3 aka data stack at all, maybe because the script was converted)
    /// Call a Subroutine without Parameters
    #[brw(magic(0x48u8))]
    gosub {
        target: CodeAddress,
    },
    /// Return from a Subroutine called with `gosub`
    #[brw(magic(0x49u8))]
    retsub {},
    /// Jump via Table
    /// Used to implement switch statements
    #[brw(magic(0x4au8))]
    jt {
        value: NumberSpec,
        table: U16SmallList<[CodeAddress; 32]>,
    },
    // 0x4b not implemented
    #[brw(magic(0x4cu8))]
    rnd {
        dest: MemoryAddress,
        min: NumberSpec,
        max: NumberSpec,
    },
    /// Push Values to call stack
    /// Used to preserve values of memory probably
    #[brw(magic(0x4du8))]
    push {
        values: U8SmallNumberList,
    },
    /// Pop Values from call stack
    /// Used to restore values of memory previously pushed by push
    #[brw(magic(0x4eu8))]
    pop {
        dest: U8SmallList<[MemoryAddress; 6]>,
    },
    /// Call Subroutine with Parameters
    #[brw(magic(0x4fu8))]
    call {
        target: CodeAddress,
        args: U8SmallNumberList,
    },
    /// Return from Subroutine called with `call`
    #[brw(magic(0x50u8))]
    r#return {},
    Command(CompiletimeCommand),
}
