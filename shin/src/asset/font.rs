use std::io::Cursor;

use anyhow::Context;
use shin_core::format::font::{read_lazy_font, FontLazy};

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

impl Asset for FontLazy {
    type Args = ();

    async fn load(
        _context: &AssetLoadContext,
        _args: (),
        data: AssetDataAccessor,
    ) -> anyhow::Result<Self> {
        read_lazy_font(&mut Cursor::new(data.read_all().await)).context("Reading font")
    }
}
