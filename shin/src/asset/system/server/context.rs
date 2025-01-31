use std::sync::Arc;

use shin_core::format::bustup::BustupBlockId;

use crate::asset::{picture::GpuPictureBlock, system::cache::AssetCache};

pub struct AssetLoadContext {
    pub wgpu_device: Arc<wgpu::Device>,
    pub wgpu_queue: Arc<wgpu::Queue>,
    pub bustup_cache: AssetCache<BustupBlockId, GpuPictureBlock>,
}
