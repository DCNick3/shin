use std::sync::Arc;

use anyhow::{Context, Result};
use shin_core::format::audio::{AudioFile, read_audio};

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

impl Asset for AudioFile {
    type Args = ();

    async fn load(
        _context: &Arc<AssetLoadContext>,
        _args: (),
        _name: &str,
        data: AssetDataAccessor,
    ) -> Result<Self> {
        read_audio(&data.read_all().await).context("Parsing audio file")
    }
}
