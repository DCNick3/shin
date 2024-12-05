use encase::ShaderType;
use glam::{vec4, Vec2, Vec3, Vec4};

pub trait VertexAttribute {
    const FORMAT: wgpu::VertexFormat;
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
pub struct UnormColor(pub u32);

impl UnormColor {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self((r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((a as u32) << 24))
    }

    pub fn into_vec4(self) -> Vec4 {
        vec4(
            (self.0 & 0xff) as f32 / 255.0,
            ((self.0 >> 8) & 0xff) as f32 / 255.0,
            ((self.0 >> 16) & 0xff) as f32 / 255.0,
            ((self.0 >> 24) & 0xff) as f32 / 255.0,
        )
    }

    pub const RED: Self = Self(0xff0000ff);
    pub const GREEN: Self = Self(0xff00ff00);
    pub const BLUE: Self = Self(0xffff0000);

    pub const PASTEL_GREEN: Self = Self(0xffc1e1c1);
    pub const PASTEL_PINK: Self = Self(0xffdcd1ff);

    pub const WHITE: Self = Self(0xffffffff);
    pub const BLACK: Self = Self(0xff000000);
}

#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(C)]
pub struct FloatColor4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl FloatColor4 {
    /// Creates a new [`FloatColor4`] from an integer of form `0xARGB` (4 bits per channel).
    pub fn from_4bpp_property(value: i32) -> Self {
        let alpha = ((value & 0xf000) >> 12) as f32 / 0xf as f32;
        let red = ((value & 0x0f00) >> 8) as f32 / 0xf as f32;
        let green = ((value & 0x00f0) >> 4) as f32 / 0xf as f32;
        let blue = ((value & 0x000f) >> 0) as f32 / 0xf as f32;

        Self::from_rgba(red, green, blue, alpha)
    }

    pub const fn from_unorm(color: UnormColor) -> Self {
        Self {
            r: (color.0 & 0xff) as f32 / 255.0,
            g: ((color.0 >> 8) & 0xff) as f32 / 255.0,
            b: ((color.0 >> 16) & 0xff) as f32 / 255.0,
            a: ((color.0 >> 24) & 0xff) as f32 / 255.0,
        }
    }

    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_vec4(vec: Vec4) -> Self {
        let [r, g, b, a] = vec.to_array();
        Self { r, g, b, a }
    }

    pub const fn into_vec4(self) -> Vec4 {
        vec4(self.r, self.g, self.b, self.a)
    }

    pub const fn into_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn into_unorm(self) -> UnormColor {
        let [r, g, b, a] = self.into_array().map(|c| (c * 255.0) as u8);
        UnormColor::from_rgba(r, g, b, a)
    }

    pub const RED: Self = Self::from_unorm(UnormColor::RED);
    pub const GREEN: Self = Self::from_unorm(UnormColor::GREEN);
    pub const BLUE: Self = Self::from_unorm(UnormColor::BLUE);

    pub const PASTEL_GREEN: Self = Self::from_unorm(UnormColor::PASTEL_GREEN);
    pub const PASTEL_PINK: Self = Self::from_unorm(UnormColor::PASTEL_PINK);

    pub const WHITE: Self = Self::from_unorm(UnormColor::WHITE);
    pub const BLACK: Self = Self::from_unorm(UnormColor::BLACK);
}

impl std::ops::Mul<FloatColor4> for FloatColor4 {
    type Output = FloatColor4;

    fn mul(self, rhs: FloatColor4) -> Self::Output {
        Self::from_vec4(self.into_vec4() * rhs.into_vec4())
    }
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
