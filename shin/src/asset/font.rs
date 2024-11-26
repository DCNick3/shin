use std::io::Cursor;

use anyhow::Context;
use shin::asset::AssetDataAccessor;
use shin_core::format::font::{read_lazy_font, LazyFont};

use crate::asset::Asset;

impl Asset for LazyFont {
    async fn load(data: AssetDataAccessor) -> anyhow::Result<Self> {
        read_lazy_font(&mut Cursor::new(data.read_all().await)).context("Reading font")
    }
}
