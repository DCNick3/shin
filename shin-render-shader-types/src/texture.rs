use enum_iterator::Sequence;

#[derive(Debug)]
pub struct TextureSamplerStore {
    pub linear: wgpu::Sampler,
}

impl TextureSamplerStore {
    pub fn new(device: &wgpu::Device) -> Self {
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Linear Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self { linear }
    }

    pub fn get(&self, sampler: TextureSampler) -> &wgpu::Sampler {
        match sampler {
            TextureSampler::Linear => &self.linear,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TextureSampler {
    Linear,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureSource<'a> {
    pub view: &'a wgpu::TextureView,
    pub sampler: TextureSampler,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum TextureTargetKind {
    Screen,
    RenderTexture,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureTarget<'a> {
    pub kind: TextureTargetKind,
    pub view: &'a wgpu::TextureView,
}

#[derive(Debug, Copy, Clone)]
pub struct DepthStencilTarget<'a> {
    pub view: &'a wgpu::TextureView,
}
