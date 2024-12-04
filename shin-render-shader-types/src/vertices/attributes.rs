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

    pub const PASTEL_GREEN: Self = Self(0xffc1e1c1);
    pub const PASTEL_PINK: Self = Self(0xffdcd1ff);

    pub const WHITE: Self = Self(0xffffffff);
    pub const BLACK: Self = Self(0xff000000);
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(transparent)]
pub struct FloatColor4 {
    inner: Vec4,
}

impl FloatColor4 {
    pub const fn from_unorm(color: UnormColor) -> Self {
        Self {
            inner: Vec4::new(
                (color.0 & 0xff) as f32 / 255.0,
                ((color.0 >> 8) & 0xff) as f32 / 255.0,
                ((color.0 >> 16) & 0xff) as f32 / 255.0,
                ((color.0 >> 24) & 0xff) as f32 / 255.0,
            ),
        }
    }

    pub const fn from_vec4(vec4: Vec4) -> Self {
        Self { inner: vec4 }
    }

    pub const RED: Self = Self::from_unorm(UnormColor::RED);
    pub const GREEN: Self = Self::from_unorm(UnormColor::GREEN);
    pub const BLUE: Self = Self::from_unorm(UnormColor::BLUE);

    pub const PASTEL_GREEN: Self = Self::from_unorm(UnormColor::PASTEL_GREEN);
    pub const PASTEL_PINK: Self = Self::from_unorm(UnormColor::PASTEL_PINK);

    pub const WHITE: Self = Self::from_unorm(UnormColor::WHITE);
    pub const BLACK: Self = Self::from_unorm(UnormColor::BLACK);
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
