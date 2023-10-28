use std::{fmt::Debug, io, io::Seek, marker::PhantomData};

use binrw::{BinRead, BinResult, BinWrite, Endian};

use super::Register;
use crate::{
    format::scenario::instruction_elements::RegisterRepr,
    vm::{IntoRuntimeForm, VmCtx},
};

/// Specifies how to get a 32-bit signed number at runtime
///
/// It can be a constant or be referencing a register.
///
/// [FromVmCtx](crate::vm::FromVmCtx) trait is used to convert it to runtime representation in command definitions (see [crate::vm::command])
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UntypedNumberSpec {
    Constant(i32),
    Register(Register),
}

impl Debug for UntypedNumberSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(c) => write!(f, "{}", c),
            Self::Register(r) => write!(f, "{}", r),
        }
    }
}

impl BinRead for UntypedNumberSpec {
    type Args<'a> = ();

    //noinspection SpellCheckingInspection
    fn read_options<R: io::Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let pos = reader.stream_position()?;

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
            // P=2 => 28-bit signed constante (KKKK denotes the upper 4 bits, 3 lower bytes are read from the stream)
            // P=3 => 4-bit regular register, KKKK is the index
            // P=4 => 12-bit regular register, KKKK denotes the upper 4 bits, lsb is read from the next byte
            // P=5 => 4-bit argument register, KKKK is the index
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
                3 => Self::Register(Register::from_regular_register(k as u16)),
                4 => Self::Register(Register::from_regular_register(
                    u8::read_options(reader, endian, ())? as u16 | (k as u16) << 8,
                )),
                5 => Self::Register(Register::from_argument(k as u16)),
                _ => {
                    return Err(binrw::Error::AssertFail {
                        message: format!("Unknown NumberSpec type: t=0x{:02x}, P={}", t, p),
                        pos,
                    })
                }
            }
        } else {
            // signed 7-bit integer
            // does the sign extension of t, using bits [0:6]
            let res = (t as i32 & 0x7f) << 25 >> 25;
            Self::Constant(res)
        })
    }
}

impl BinWrite for UntypedNumberSpec {
    type Args<'a> = ();

    fn write_options<W: io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        use RegisterRepr::*;
        use UntypedNumberSpec::*;

        fn enc_t(p: u8, k: u8) -> u8 {
            assert!(p < 6);
            assert!(k < 16);

            0x80 | (p << 4) | k
        }

        let pos = writer.stream_position()?;
        let mut write_byte = |b: u8| -> BinResult<()> { b.write_options(writer, endian, ()) };

        match *self {
            // 7-bit signed constant
            Constant(val @ -0x40..=0x3f) => write_byte((val as i8 as u8) & 0x7f),
            // 12-bit signed constant
            Constant(val @ -0x800..=0x7ff) => {
                let k = ((val >> 8) & 0xf) as u8;
                let b1 = (val & 0xff) as u8;

                write_byte(enc_t(0, k))?;
                write_byte(b1)
            }
            // 20-bit signed constant
            Constant(val @ -0x80000..=0x7ffff) => {
                let k = ((val >> 16) & 0xf) as u8;
                let b1 = ((val >> 8) & 0xff) as u8;
                let b2 = (val & 0xff) as u8;

                write_byte(enc_t(1, k))?;
                write_byte(b1)?;
                write_byte(b2)
            }
            // 28-bit signed constant
            Constant(val @ -0x8000000..=0x7ffffff) => {
                let k = ((val >> 24) & 0xf) as u8;
                let b1 = ((val >> 16) & 0xff) as u8;
                let b2 = ((val >> 8) & 0xff) as u8;
                let b3 = (val & 0xff) as u8;

                write_byte(enc_t(2, k))?;
                write_byte(b1)?;
                write_byte(b2)?;
                write_byte(b3)
            }
            Constant(val) => Err(binrw::Error::AssertFail {
                message: format!("NumberSpec constant value out of range: {}", val),
                pos,
            }),
            Register(register) => match register.repr() {
                Regular(index @ 0..=15) => write_byte(enc_t(3, index as u8)),
                Regular(index) => {
                    assert!(index <= 0xfff);
                    let k = (index >> 8) as u8;
                    let b1 = (index & 0xff) as u8;

                    write_byte(enc_t(4, k))?;
                    write_byte(b1)
                }
                Argument(index) => write_byte(enc_t(5, index as u8)),
            },
        }
    }
}

#[derive(BinRead)]
pub struct NumberSpec<T = i32>(UntypedNumberSpec, PhantomData<T>);

impl<T> Clone for NumberSpec<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}
impl<T> Copy for NumberSpec<T> {}
impl<T> PartialEq for NumberSpec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for NumberSpec<T> {}

impl<T> NumberSpec<T> {
    pub const fn new(spec: UntypedNumberSpec) -> Self {
        Self(spec, PhantomData)
    }

    pub fn into_untyped(self) -> UntypedNumberSpec {
        self.0
    }
}

// See https://github.com/jam1garner/binrw/pull/230
impl<T> BinWrite for NumberSpec<T> {
    type Args<'a> = ();

    fn write_options<W: io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.0.write_options(writer, endian, args)
    }
}

impl<T> Debug for NumberSpec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: FromNumber> IntoRuntimeForm for NumberSpec<T> {
    type Output = T;
    fn into_runtime_form(self, ctx: &VmCtx) -> Self::Output {
        ctx.get_number(self)
    }
}

pub trait FromNumber {
    fn from_number(number: i32) -> Self;
}

impl FromNumber for bool {
    fn from_number(number: i32) -> Self {
        number != 0
    }
}

impl FromNumber for i32 {
    fn from_number(number: i32) -> Self {
        number
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{io::NoSeek, BinRead, BinWrite};

    use super::UntypedNumberSpec::*;
    use crate::format::{
        scenario::instruction_elements::UntypedNumberSpec, test_util::assert_enc_dec_pair,
    };

    #[test]
    fn enc_dec_const_7bit() {
        assert_enc_dec_pair(&Constant(0), "00");
        assert_enc_dec_pair(&Constant(1), "01");
        assert_enc_dec_pair(&Constant(-1), "7f");
        assert_enc_dec_pair(&Constant(63), "3f");
        assert_enc_dec_pair(&Constant(-63), "41");
        assert_enc_dec_pair(&Constant(-64), "40");
    }
    #[test]
    fn enc_dec_const_12bit() {
        // t=1000KKKK, so `8` is always the first nibble
        assert_enc_dec_pair(&Constant(64), "8040");
        assert_enc_dec_pair(&Constant(65), "8041");
        assert_enc_dec_pair(&Constant(-65), "8fbf");
        assert_enc_dec_pair(&Constant(127), "807f");
        assert_enc_dec_pair(&Constant(-128), "8f80");
        assert_enc_dec_pair(&Constant(128), "8080");
        assert_enc_dec_pair(&Constant(2047), "87ff");
        assert_enc_dec_pair(&Constant(-2047), "8801");
        assert_enc_dec_pair(&Constant(-2048), "8800");
    }

    #[test]
    fn enc_dec_const_20bit() {
        // t=1001KKKK, so `9` is always the first nibble
        assert_enc_dec_pair(&Constant(2048), "900800");
        assert_enc_dec_pair(&Constant(2049), "900801");
        assert_enc_dec_pair(&Constant(-2049), "9ff7ff");
        assert_enc_dec_pair(&Constant(4095), "900fff");
        assert_enc_dec_pair(&Constant(-4096), "9ff000");
        assert_enc_dec_pair(&Constant(4096), "901000");
        assert_enc_dec_pair(&Constant(524287), "97ffff");
        assert_enc_dec_pair(&Constant(-524287), "980001");
        assert_enc_dec_pair(&Constant(-524288), "980000");
    }

    #[test]
    fn enc_dec_const_26bit() {
        // t=1010KKKK, so `a` is always the first nibble
        assert_enc_dec_pair(&Constant(524288), "a0080000");
        assert_enc_dec_pair(&Constant(524289), "a0080001");
        assert_enc_dec_pair(&Constant(-524289), "aff7ffff");
        assert_enc_dec_pair(&Constant(16777215), "a0ffffff");
        assert_enc_dec_pair(&Constant(16777216), "a1000000");
        assert_enc_dec_pair(&Constant(-16777216), "af000000");
        assert_enc_dec_pair(&Constant(134217727), "a7ffffff");
        assert_enc_dec_pair(&Constant(-134217727), "a8000001");
        assert_enc_dec_pair(&Constant(-134217728), "a8000000");
    }

    #[test]
    fn enc_dec_reg_4bit() {
        // t=1011KKKK, so `b` is always the first nibble
        assert_enc_dec_pair(&Register("$v0".parse().unwrap()), "b0");
        assert_enc_dec_pair(&Register("$v1".parse().unwrap()), "b1");
        assert_enc_dec_pair(&Register("$v15".parse().unwrap()), "bf");
    }

    #[test]
    fn enc_dec_reg_12bit() {
        // t=1100KKKK, so `c` is always the first nibble
        assert_enc_dec_pair(&Register("$v16".parse().unwrap()), "c010");
        assert_enc_dec_pair(&Register("$v32".parse().unwrap()), "c020");
        assert_enc_dec_pair(&Register("$v4095".parse().unwrap()), "cfff");
    }

    #[test]
    fn enc_dec_reg_arg() {
        // t=1101KKKK, so `d` is always the first nibble
        assert_enc_dec_pair(&Register("$a0".parse().unwrap()), "d0");
        assert_enc_dec_pair(&Register("$a1".parse().unwrap()), "d1");
        assert_enc_dec_pair(&Register("$a15".parse().unwrap()), "df");
    }

    #[test]
    fn enc_out_of_range() {
        fn assert_out_of_range_error(value: i32) {
            match Constant(value)
                .write_le(&mut NoSeek::new(Vec::new()))
                .unwrap_err()
            {
                binrw::Error::AssertFail { message, pos } => {
                    assert_eq!(
                        message,
                        format!("NumberSpec constant value out of range: {}", value)
                    );
                    assert_eq!(pos, 0);
                }
                v => panic!("unexpected error: {:?}", v),
            };
        }

        assert_out_of_range_error(134217728);
        assert_out_of_range_error(-134217729);
        assert_out_of_range_error(i32::MAX);
        assert_out_of_range_error(i32::MIN);
    }

    #[test]
    fn dec_unknown_type() {
        match UntypedNumberSpec::read_le(&mut NoSeek::new(Cursor::new([0xf0, 0x00]))).unwrap_err() {
            binrw::Error::AssertFail { message, pos } => {
                assert_eq!(message, "Unknown NumberSpec type: t=0xf0, P=7".to_string());
                assert_eq!(pos, 0);
            }
            v => panic!("unexpected error: {:?}", v),
        };
    }
}
