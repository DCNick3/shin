use std::{fmt::Debug, io};

use binrw::{BinRead, BinResult, BinWrite, Endian};

use crate::vm::{IntoRuntimeForm, VmCtx};

/// Message ID - a 24-bit integer
///
/// It is used to check whether a message was seen before.
#[derive(Clone, PartialEq, Eq)]
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
        let b0 = (self.0 & 0xFF) as u8;
        let b1 = ((self.0 >> 8) & 0xFF) as u8;
        let b2 = ((self.0 >> 16) & 0xFF) as u8;

        b0.write(_writer)?;
        b1.write(_writer)?;
        b2.write(_writer)?;

        Ok(())
    }
}

impl Debug for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoRuntimeForm for MessageId {
    type Output = MessageId;
    fn into_runtime_form(self, _: &VmCtx) -> Self::Output {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::MessageId;
    use crate::format::scenario::test_util::assert_enc_dec_pair;

    #[test]
    fn enc_dec() {
        assert_enc_dec_pair(&MessageId(0x123456), "563412");
        assert_enc_dec_pair(&MessageId(0x000000), "000000");
        assert_enc_dec_pair(&MessageId(0xffffff), "ffffff");
    }
}
