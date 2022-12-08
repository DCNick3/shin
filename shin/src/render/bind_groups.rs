use crate::render::common_resources::GpuCommonResources;
use std::ops::Deref;

pub struct BindGroupLayouts {
    pub camera: wgpu::BindGroupLayout,
    pub texture: wgpu::BindGroupLayout,
}

impl BindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            camera: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None, // TODO: should I specify this?
                    },
                    count: None,
                }],
            }),
            texture: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
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
            }),
        }
    }
}

pub struct CameraBindGroup(pub wgpu::BindGroup);
impl CameraBindGroup {
    // CameraBindGroup is actually part of GpuCommonResources...
    // pub fn new(resources: &GpuCommonResources, camera: BindingResource) -> Self {
    //     Self(
    //         resources
    //             .device
    //             .create_bind_group(&wgpu::BindGroupDescriptor {
    //                 label: Some("camera_bind_group"),
    //                 layout: &resources.bind_group_layouts.camera,
    //                 entries: &[wgpu::BindGroupEntry {
    //                     binding: 0,
    //                     resource: camera,
    //                 }],
    //             }),
    //     )
    // }
}
impl Deref for CameraBindGroup {
    type Target = wgpu::BindGroup;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct TextureBindGroup(pub wgpu::BindGroup);
impl TextureBindGroup {
    pub fn new(
        resources: &GpuCommonResources,
        texture_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        label: Option<&str>,
    ) -> Self {
        let bind_group = resources
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label,
                layout: &resources.bind_group_layouts.texture,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
        Self(bind_group)
    }
}
impl Deref for TextureBindGroup {
    type Target = wgpu::BindGroup;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
