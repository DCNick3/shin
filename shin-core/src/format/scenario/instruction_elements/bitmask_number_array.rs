use super::UntypedNumberSpec;
use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::io;

/// Represents 8 numbers, each of which may or may not be present.
///
/// If the number is not present, it is treated as `NumberSpec::Constant(0)`.
#[derive(Debug, Copy, Clone)]
pub struct BitmaskNumberArray(pub [UntypedNumberSpec; 8]);

impl BinRead for BitmaskNumberArray {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,

        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let mut res = [UntypedNumberSpec::Constant(0); 8];
        let mut mask = u8::read_options(reader, endian, ())?;
        for res in res.iter_mut() {
            if mask & 1 != 0 {
                *res = UntypedNumberSpec::read_options(reader, endian, ())?;
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
