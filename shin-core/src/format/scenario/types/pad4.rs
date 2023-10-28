use std::{io, io::Seek};

use binrw::{BinRead, BinResult, BinWrite, Endian};
use derivative::Derivative;

/// Pad the contents to 4 bytes
///
/// (Used in [super::Instruction::gt])
#[derive(Derivative, PartialEq, Eq, Copy, Clone)]
#[derivative(Debug = "transparent")]
pub struct Pad4<T>(pub T);

impl<T: for<'a> BinRead<Args<'a> = ()> + 'static> BinRead for Pad4<T> {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let pos = reader.stream_position()?;
        let res = <_>::read_options(reader, endian, args)?;
        let new_pos = reader.stream_position()?;

        assert!(new_pos - pos <= 4, "Pad4: read more than 4 bytes");

        // read the padding bytes
        for _ in 0..(4 - (new_pos - pos)) {
            u8::read_options(reader, endian, ())?;
        }

        Ok(Self(res))
    }
}
impl<T: for<'a> BinWrite<Args<'a> = ()>> BinWrite for Pad4<T> {
    type Args<'a> = ();

    fn write_options<W: io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        let pos = writer.stream_position()?;
        <_>::write_options(&self.0, writer, endian, ())?;
        let new_pos = writer.stream_position()?;

        assert!(new_pos - pos <= 4, "Pad4: wrote more than 4 bytes");

        // write the padding zero bytes
        for _ in 0..(4 - (new_pos - pos)) {
            0u8.write_options(writer, endian, ())?;
        }

        Ok(())
    }
}
