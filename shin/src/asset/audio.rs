use anyhow::{Context, Result};
use shin_core::format::audio::{read_audio, AudioFile};

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

impl Asset for AudioFile {
    async fn load(_context: &AssetLoadContext, data: AssetDataAccessor) -> Result<Self> {
        read_audio(&data.read_all().await).context("Parsing audio file")
    }
}
