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
        todo!()
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
