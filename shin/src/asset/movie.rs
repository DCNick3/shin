use anyhow::{Context, Result};
use shin_audio::AudioManager;
use shin_video::{mp4::Mp4, VideoPlayerHandle};

use crate::asset::system::{Asset, AssetDataAccessor, AssetDataCursor, AssetLoadContext};

pub struct Movie {
    label: String,
    // TODO: allow to start decoding the video before the first frame is requested
    mp4: Mp4<AssetDataCursor>,
}

impl Asset for Movie {
    type Args = ();

    async fn load(
        _context: &AssetLoadContext,
        _args: (),
        name: &str,
        data: AssetDataAccessor,
    ) -> Result<Self> {
        let cursor = data.cursor();
        let mp4 = Mp4::new(cursor).context("Reading Mp4")?;
        Ok(Self {
            label: name.to_string(),
            mp4,
        })
    }
}

impl Movie {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn play(
        &self,
        device: &wgpu::Device,
        audio_manager: &AudioManager,
    ) -> Result<VideoPlayerHandle> {
        VideoPlayerHandle::new(device, audio_manager, self.mp4.clone())
    }
}
