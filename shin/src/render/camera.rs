use crate::render::bind_groups::{BindGroupLayouts, CameraBindGroup};
use cgmath::{Matrix4, SquareMatrix};
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraParams {
    pub projection_matrix: Matrix4<f32>,
}

pub const VIRTUAL_WIDTH: f32 = 1920.0;
pub const VIRTUAL_HEIGHT: f32 = 1080.0;

pub struct Camera {
    projection_matrix: Matrix4<f32>,
    buffer: wgpu::Buffer,
    bind_group: Arc<CameraBindGroup>,
}

impl Camera {
    fn compute_projection_matrix(window_size: (u32, u32)) -> Matrix4<f32> {
        let (window_width, window_height) = window_size;

        let w = window_width as f32 / VIRTUAL_WIDTH;
        let h = window_height as f32 / VIRTUAL_HEIGHT;

        let (viewport_width, viewport_height) = if w < h {
            (VIRTUAL_WIDTH, VIRTUAL_HEIGHT * h / w)
        } else {
            (VIRTUAL_WIDTH * w / h, VIRTUAL_HEIGHT)
        };

        let mut projection_matrix = Matrix4::identity();
        projection_matrix.x.x = 2.0 / viewport_width;
        projection_matrix.y.y = 2.0 / viewport_height;
        projection_matrix.z.z = 1.0 / 1000.0;
        projection_matrix.w.w = 1.0;
        projection_matrix
    }

    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &BindGroupLayouts,
        window_size: (u32, u32),
    ) -> Self {
        let projection_matrix = Self::compute_projection_matrix(window_size);

        let camera_params = CameraParams { projection_matrix };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = CameraBindGroup(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &bind_group_layouts.camera,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        }));

        Self {
            projection_matrix,
            buffer,
            bind_group: Arc::new(bind_group),
        }
    }

    pub fn resize(&mut self, _device: &wgpu::Device, queue: &mut wgpu::Queue, size: (u32, u32)) {
        self.projection_matrix = Self::compute_projection_matrix(size);
        let mtx = [self.projection_matrix];
        let contents = bytemuck::cast_slice(&mtx);
        queue.write_buffer(&self.buffer, 0, contents);
    }

    pub fn bind_group(&self) -> &Arc<CameraBindGroup> {
        &self.bind_group
    }
}
