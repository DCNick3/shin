pub struct TextureBindGroupLayout(pub wgpu::BindGroupLayout);

impl TextureBindGroupLayout {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TextureBindGroup layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        TextureBindGroupLayout(layout)
    }
}

#[derive(Debug)]
pub struct DefaultTextureSampler(pub wgpu::Sampler);

impl DefaultTextureSampler {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Default sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        DefaultTextureSampler(sampler)
    }
}

#[derive(Debug)]
pub struct TextureBindGroup(pub wgpu::BindGroup);

// TODO: when implementing, try to make it consistent with `ResizeableTexture`, which is a thing we use to keep Framebuffer-sized textures
// struct TextureInner {
//     texture: wgpu::Texture,
//     bind_group: TextureBindGroup,
//     width: u32,
//     height: u32,
// }
//
// pub struct ReadonlyTexture {}
// pub struct WriteableTexture {}
