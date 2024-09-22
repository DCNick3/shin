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

pub struct DefaultTextureSampler {}

#[derive(Debug)]
pub struct TextureBindGroup(pub wgpu::BindGroup);

struct TextureInner {
    texture: wgpu::Texture,
    bind_group: TextureBindGroup,
    width: u32,
    height: u32,
}

pub struct ReadonlyTexture {}
pub struct WriteableTexture {}
