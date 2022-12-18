//! Support for BUP files, storing the character bustup sprites.

use binrw::{BinRead, BinWrite};

#[derive(BinRead, BinWrite, Debug)]
#[br(little, magic = b"PIC4")]
struct BustupHeader {
    //
}

pub struct Bustup {}
