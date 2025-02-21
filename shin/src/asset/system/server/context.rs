use shin_core::format::bustup::BustupBlockId;

use crate::asset::{picture::GpuPictureBlock, system::cache::AssetCache};

pub struct AssetLoadContext {
    pub wgpu_device: wgpu::Device,
    pub wgpu_queue: wgpu::Queue,
    pub bustup_cache: AssetCache<BustupBlockId, GpuPictureBlock>,
}
