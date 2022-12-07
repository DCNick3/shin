use crate::asset::picture::GpuPicture;
use crate::layer::{Layer, LayerProperties};
use crate::render::pipelines::{DrawSource, SpriteVertex};
use crate::render::{pipelines, RenderContext, Renderable};
use crate::update::{Updatable, UpdateContext};
use cgmath::{Matrix4, Vector2, Vector3, Vector4};
use shin_core::vm::command::layer::LayerProperty;
use wgpu::util::DeviceExt;

pub struct PictureLayer {
    picture: GpuPicture,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    props: LayerProperties,
}

impl PictureLayer {
    pub fn new(device: &wgpu::Device, picture: GpuPicture) -> Self {
        let origin_translate = -Vector3::new(picture.origin_x as f32, picture.origin_y as f32, 0.0);

        let color = Vector4::new(1.0, 1.0, 1.0, 1.0);

        let vertices = [
            // 0
            SpriteVertex {
                position: Vector3::new(0.0, picture.height() as f32, 0.0) + origin_translate,
                color,
                texture_coordinate: Vector2::new(0.0, 0.0),
            },
            // 1
            SpriteVertex {
                position: Vector3::new(0.0, 0.0, 0.0) + origin_translate,
                color,
                texture_coordinate: Vector2::new(0.0, 1.0),
            },
            // 2
            SpriteVertex {
                position: Vector3::new(picture.width() as f32, picture.height() as f32, 0.0)
                    + origin_translate,
                color,
                texture_coordinate: Vector2::new(1.0, 0.0),
            },
            // 3
            SpriteVertex {
                position: Vector3::new(picture.width() as f32, 0.0, 0.0) + origin_translate,
                color,
                texture_coordinate: Vector2::new(1.0, 1.0),
            },
        ];

        let indices = [0u16, 1, 2, 2, 1, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PictureLayer vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PictureLayer index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            picture,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            props: LayerProperties::new(),
        }
    }
}

impl Renderable for PictureLayer {
    fn render<'a>(&'a self, ctx: &mut RenderContext<'a, '_>) {
        pipelines::sprite::draw(
            ctx,
            DrawSource::VertexIndexBuffer {
                vertex_buffer: &self.vertex_buffer,
                index_buffer: &self.index_buffer,
                indices: 0..self.num_indices,
                instances: 0..1,
            },
            &self.picture,
            // TODO: actually use all the properties
            // TODO: move the transformation matrix calculation to the LayerProperties struct
            Matrix4::from_angle_z(cgmath::Deg(
                self.props.get_property(LayerProperty::Rotation),
            )),
        );
    }

    fn resize(&mut self, _size: (u32, u32)) {
        // no internal buffers to resize
    }
}

impl Updatable for PictureLayer {
    fn update(&mut self, ctx: &UpdateContext) {
        self.props.update(ctx);
    }
}

impl Layer for PictureLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
