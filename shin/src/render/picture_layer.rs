use crate::asset::picture::GpuPicture;
use crate::interpolator::{Easing, Interpolator};
use crate::render::pipelines::{DrawSource, SpriteVertex};
use crate::render::{pipelines, RenderContext};
use cgmath::{Matrix4, Vector2, Vector3, Vector4};
use wgpu::util::DeviceExt;

pub struct PictureLayer {
    picture: GpuPicture,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    rotation: Interpolator,
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

        let mut interpolator = Interpolator::new(0.0);
        interpolator.enqueue(400.0, 6.0, Easing::EaseIn);
        interpolator.enqueue(-400.0, 8.0, Easing::Identity);
        interpolator.enqueue(0.0, 6.0, Easing::EaseOut);

        Self {
            picture,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            rotation: interpolator,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.rotation.update(delta_time);
    }

    pub fn render<'a>(&'a self, ctx: &mut RenderContext<'a, '_>) {
        pipelines::sprite::draw(
            ctx,
            DrawSource::VertexIndexBuffer {
                vertex_buffer: &self.vertex_buffer,
                index_buffer: &self.index_buffer,
                indices: 0..self.num_indices,
                instances: 0..1,
            },
            &self.picture,
            Matrix4::from_angle_z(cgmath::Deg(self.rotation.value())),
        );
    }
}
