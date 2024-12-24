use glam::{Vec2, Vec3, Vec4};
use shin_primitives::color::{FloatColor4, UnormColor};

pub trait VertexAttribute {
    const FORMAT: wgpu::VertexFormat;
}
impl VertexAttribute for f32 {
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32;
}

impl VertexAttribute for Vec2 {
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x2;
}

impl VertexAttribute for Vec3 {
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x3;
}

impl VertexAttribute for Vec4 {
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x4;
}

impl VertexAttribute for UnormColor {
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Unorm8x4;
}

impl VertexAttribute for FloatColor4 {
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x4;
}
