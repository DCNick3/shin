use openh264::formats::YUVSource;
use std::num::NonZeroU32;
use std::ops::Range;
use wgpu::include_wgsl;

pub struct VideoRenderer {
    tex_y: wgpu::Texture,
    tex_u: wgpu::Texture,
    tex_v: wgpu::Texture,
    bind_group: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
}

impl VideoRenderer {
    pub fn new(
        yuv: &impl YUVSource,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        // note that this assumes 4:2:0 chroma subsampling is used
        // as of now, this is the only subsampling supported by openh264 crate

        let tex_y = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VideoRenderer Y Texture"),
            size: wgpu::Extent3d {
                width: yuv.width() as u32,
                height: yuv.height() as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let tex_u = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VideoRenderer U Texture"),
            size: wgpu::Extent3d {
                width: yuv.width() as u32 / 2,
                height: yuv.height() as u32 / 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Snorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let tex_v = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VideoRenderer V Texture"),
            size: wgpu::Extent3d {
                width: yuv.width() as u32 / 2,
                height: yuv.height() as u32 / 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Snorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let shader_module = device.create_shader_module(include_wgsl!("video_renderer.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("VideoRenderer Bind Group Layout"),
            entries: &[
                // Y texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // U texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // V texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // texture sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("VideoRenderer Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("VideoRenderer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[
                    // this corresponds to a PosColTexVertex in shin
                    wgpu::VertexBufferLayout {
                        array_stride: 36,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // position
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            },
                            // color
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 12,
                                shader_location: 1,
                            },
                            // texture_coordinate
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 28,
                                shader_location: 2,
                            },
                        ],
                    },
                ],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("VideoRenderer Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: Default::default(),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: Default::default(),
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VideoRenderer Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                // Y texture
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &tex_y.create_view(&Default::default()),
                    ),
                },
                // U texture
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &tex_u.create_view(&Default::default()),
                    ),
                },
                // V texture
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &tex_v.create_view(&Default::default()),
                    ),
                },
                // texture sampler
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let result = Self {
            tex_y,
            tex_u,
            tex_v,
            bind_group,

            pipeline,
        };

        result.write_data(yuv, &queue);

        result
    }

    pub fn write_data(&self, yuv: &impl YUVSource, queue: &wgpu::Queue) {
        // note that this assumes 4:2:0 chroma subsampling is used
        // as of now, this is the only subsampling supported by openh264 crate

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tex_y,
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            yuv.y(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(yuv.y_stride() as u32).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                // Y is not subsampled
                width: yuv.width() as u32,
                height: yuv.height() as u32,
                depth_or_array_layers: 1,
            },
        );
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tex_u,
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            yuv.u(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(yuv.u_stride() as u32).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                // U is subsampled by 2
                width: yuv.width() as u32 / 2,
                height: yuv.height() as u32 / 2,
                depth_or_array_layers: 1,
            },
        );
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tex_v,
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            yuv.v(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(yuv.v_stride() as u32).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                // V is not subsampled
                width: yuv.width() as u32 / 2,
                height: yuv.height() as u32 / 2,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn draw<'enc>(
        &'enc self,
        render_pass: &mut wgpu::RenderPass<'enc>,
        vertices: Range<u32>,
        instances: Range<u32>,
    ) {
        //
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(vertices, instances);
    }
}
