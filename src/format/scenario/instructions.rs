use binrw::{binrw, BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io;
use std::io::{Read, Seek, Write};

pub enum NumberSpec {
    Constant(i32),
    // technically there are two kinds of memories in the VM...
    // I think one is linear memory and another one is stack (known as Mem1 and Mem3 in ShinDataUtil)
    // but I didn't see stack used much...
    Memory(u32),
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
                1 | 2 => todo!("number spec with p={}", p),
                3 => Self::Memory(k as u32),
                4 => Self::Memory(u8::read_options(reader, options, ())? as u32 | (k as u32) << 8),
                5 => todo!("number spec with p={} (Mem3 address)", p),
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

#[derive(BinRead, BinWrite)]
#[repr(u8)]
#[brw(repr = u8)]
pub enum BinaryOperationType {
    Argument2 = 0,
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
}
pub struct BinaryOperation {
    ty: BinaryOperationType,
    left: NumberSpec,
    right: NumberSpec,
    destination: u16,
}

impl BinRead for BinaryOperation {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let temp = u8::read_options(reader, options, ())?;
        todo!()
        // if (temp & 0x80) != 0 {
        // } else {
        // }
    }
}

impl BinWrite for BinaryOperation {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

#[derive(BinRead, BinWrite)]
#[brw(little)]
pub struct JumpAddress {
    address: u32,
}

#[derive(FromPrimitive)]
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

pub struct JumpCond {
    pub is_negated: bool,
    pub condition: JumpCondType,
}

impl BinRead for JumpCond {
    type Args = ();

    fn read_options<R: Read + Seek>(
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

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(BinRead, BinWrite)]
#[brw(little)]
pub enum Command {
    #[brw(magic(0x0u8))]
    EXIT { arg1: u8, arg2: NumberSpec },
}

#[allow(non_camel_case_types)]
#[derive(BinRead, BinWrite)]
#[br(little)]
pub enum Instruction {
    #[brw(magic(0x41u8))]
    bo(BinaryOperation),
    // exp,
    #[brw(magic(0x46u8))]
    jc {
        cond: JumpCond,
        left: NumberSpec,
        right: NumberSpec,
        target: JumpAddress,
    },
    // j,
    // call,
    // ret,
    // jt,
    // rnd,
    // push,
    // pop,
    Command(Command),
}
