use std::io::Cursor;

use anyhow::{Context, Result};
use shin_audio::AudioManager;
use shin_video::{mp4::Mp4, VideoPlayer};

use crate::asset::system::{Asset, AssetDataAccessor, AssetDataCursor, AssetLoadContext};

pub struct Movie {
    // TODO: allow to start decoding the video before the first frame is requested
    mp4: Mp4<AssetDataCursor>,
}

impl Asset for Movie {
    async fn load(_context: &AssetLoadContext, data: AssetDataAccessor) -> Result<Self> {
        let cursor = data.cursor();
        let mp4 = Mp4::new(cursor).context("Reading Mp4")?;
        Ok(Self { mp4 })
    }
}

impl Movie {
    pub fn play(&self, device: &wgpu::Device, audio_manager: &AudioManager) -> Result<VideoPlayer> {
        VideoPlayer::new(device, audio_manager, self.mp4.clone())
    }
}
