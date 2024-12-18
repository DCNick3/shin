use std::sync::Arc;

pub struct AssetLoadContext {
    pub wgpu_device: Arc<wgpu::Device>,
    pub wgpu_queue: Arc<wgpu::Queue>,
    pub bustup_cache: crate::asset::bustup::BlockCache,
}
