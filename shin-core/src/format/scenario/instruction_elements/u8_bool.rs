use std::{fmt::Debug, io};

use binrw::{BinRead, BinResult, BinWrite, Endian};

use crate::vm::{IntoRuntimeForm, VmCtx};

#[derive(Clone, PartialEq, Eq)]
pub struct U8Bool(pub bool);

impl Debug for U8Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl BinRead for U8Bool {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        Ok(Self(u8::read_options(reader, endian, ())? != 0))
    }
}

impl BinWrite for U8Bool {
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        let v = if self.0 { 1 } else { 0 };
        u8::write_options(&v, writer, endian, ())
    }
}

impl IntoRuntimeForm for U8Bool {
    type Output = bool;

    fn into_runtime_form(self, _: &VmCtx) -> Self::Output {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::U8Bool;
    use crate::format::scenario::test_util::{assert_dec, assert_enc_dec_pair};

    #[test]
    fn enc_dec() {
        assert_enc_dec_pair(&U8Bool(true), "01");
        assert_enc_dec_pair(&U8Bool(false), "00");

        // these are weird, but we do handle them the same way the game does
        // maybe we should be more strict and error out on these?
        assert_dec(&U8Bool(true), "02");
        assert_dec(&U8Bool(true), "ff");
    }
}
