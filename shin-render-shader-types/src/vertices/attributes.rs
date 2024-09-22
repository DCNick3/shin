use encase::ShaderType;
use glam::{Vec2, Vec3, Vec4};

pub trait VertexAttribute {
    const FORMAT: wgpu::VertexFormat;
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
pub struct UnormColor(pub u32);

impl UnormColor {
    pub const RED: Self = Self(0xff0000ff);
    pub const GREEN: Self = Self(0xff00ff00);
    pub const BLUE: Self = Self(0xffff0000);
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(transparent)]
pub struct FloatColor4 {
    inner: Vec4,
}

impl FloatColor4 {
    pub const BLUE: Self = Self {
        inner: Vec4::new(0.0, 0.0, 1.0, 1.0),
    };
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(transparent)]
pub struct FloatColor3 {
    inner: Vec3,
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

impl VertexAttribute for FloatColor3 {
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x3;
}
