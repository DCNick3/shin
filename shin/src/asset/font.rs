use crate::asset::Asset;
use anyhow::Context;
use shin_core::format::font::{read_lazy_font, LazyFont};
use std::io::Cursor;

impl Asset for LazyFont {
    fn load_from_bytes(data: Vec<u8>) -> anyhow::Result<Self> {
        read_lazy_font(&mut Cursor::new(data)).context("Reading font")
    }
}
