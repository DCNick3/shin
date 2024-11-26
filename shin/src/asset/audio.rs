use anyhow::{Context, Result};
use shin_core::format::audio::{read_audio, AudioFile};

use crate::asset::{Asset, AssetDataAccessor};

impl Asset for AudioFile {
    async fn load(data: AssetDataAccessor) -> Result<Self> {
        read_audio(&data.read_all().await).context("Parsing audio file")
    }
}
