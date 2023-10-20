use crate::vm::{FromVmCtx, FromVmCtxDefault, VmCtx};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::fmt::Debug;
use std::io;

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

impl FromVmCtx<U8Bool> for bool {
    fn from_vm_ctx(_: &VmCtx, input: U8Bool) -> Self {
        input.0
    }
}
impl FromVmCtxDefault for U8Bool {
    type Output = bool;
}
