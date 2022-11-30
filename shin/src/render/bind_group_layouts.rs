use crate::asset::picture::make_picture_bind_group_layout;
use crate::render::camera::make_camera_bind_group_layout;

pub struct BindGroupLayouts {
    pub camera: wgpu::BindGroupLayout,
    pub picture: wgpu::BindGroupLayout,
}

impl BindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            camera: make_camera_bind_group_layout(device),
            picture: make_picture_bind_group_layout(device),
        }
    }
}
