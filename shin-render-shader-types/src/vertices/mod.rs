mod attributes;

use bytemuck::NoUninit;
use glam::{Vec2, Vec3, Vec4};
use shin_primitives::color::UnormColor;

pub use self::attributes::VertexAttribute;

// TODO: replace vector types with custom types that do not require any alignment besides 4 bytes
// Then we'll be able to build without enabling `glam`'s `scalar-math` feature

pub trait VertexType: NoUninit {
    const NAME: &'static str;
    const ATTRIBUTE_NAMES: &'static [&'static str];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];
    const DESCRIPTOR: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: Self::ATTRIBUTES,
    };
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C, packed)]
pub struct PosVertex {
    pub position: Vec3,
}

// TODO: this is very deriveable
impl VertexType for PosVertex {
    const NAME: &'static str = "PosVertex";
    const ATTRIBUTE_NAMES: &'static [&'static str] = &["position"];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[wgpu::VertexAttribute {
        format: <Vec3 as VertexAttribute>::FORMAT,
        offset: std::mem::offset_of!(PosVertex, position) as wgpu::BufferAddress,
        shader_location: 0,
    }];
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C, packed)]
pub struct PosColVertex {
    pub position: Vec3,
    pub color: UnormColor,
}

impl VertexType for PosColVertex {
    const NAME: &'static str = "PosColVertex";
    const ATTRIBUTE_NAMES: &'static [&'static str] = &["position", "color"];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        wgpu::VertexAttribute {
            format: <Vec3 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(PosColVertex, position) as wgpu::BufferAddress,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: <UnormColor as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(PosColVertex, color) as wgpu::BufferAddress,
            shader_location: 1,
        },
    ];
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C, packed)]
pub struct PosColTexVertex {
    pub position: Vec3,
    pub color: UnormColor,
    pub texture_position: Vec2,
}

impl PosColTexVertex {
    pub fn pos_col(self) -> PosColVertex {
        PosColVertex {
            position: self.position,
            color: self.color,
        }
    }
}

impl VertexType for PosColTexVertex {
    const NAME: &'static str = "PosColTexVertex";
    const ATTRIBUTE_NAMES: &'static [&'static str] = &["position", "color", "texture_position"];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        wgpu::VertexAttribute {
            format: <Vec3 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(PosColTexVertex, position) as wgpu::BufferAddress,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: <UnormColor as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(PosColTexVertex, color) as wgpu::BufferAddress,
            shader_location: 1,
        },
        wgpu::VertexAttribute {
            format: <Vec2 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(PosColTexVertex, texture_position) as wgpu::BufferAddress,
            shader_location: 2,
        },
    ];
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C, packed)]
pub struct TextVertex {
    /// Combined position (xy) and texture coordinate (zw)
    pub position: Vec4,
    /// 1-channel color (tint can be added with uniform parameter)
    pub color: f32,
}

impl VertexType for TextVertex {
    const NAME: &'static str = "TextVertex";
    const ATTRIBUTE_NAMES: &'static [&'static str] = &["position", "color"];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        wgpu::VertexAttribute {
            format: <Vec4 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(TextVertex, position) as wgpu::BufferAddress,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: <f32 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(TextVertex, color) as wgpu::BufferAddress,
            shader_location: 1,
        },
    ];
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C, packed)]
pub struct BlendVertex {
    pub position: Vec3,
    pub color: UnormColor,
    /// Packed texture coordinate for two textures (xy and zw)
    pub texture_position: Vec4,
}

impl VertexType for BlendVertex {
    const NAME: &'static str = "BlendVertex";
    const ATTRIBUTE_NAMES: &'static [&'static str] = &["position", "color", "texture_position"];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        wgpu::VertexAttribute {
            format: <Vec3 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(BlendVertex, position) as wgpu::BufferAddress,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: <UnormColor as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(BlendVertex, color) as wgpu::BufferAddress,
            shader_location: 1,
        },
        wgpu::VertexAttribute {
            format: <Vec4 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(BlendVertex, texture_position) as wgpu::BufferAddress,
            shader_location: 2,
        },
    ];
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C, packed)]
pub struct WindowVertex {
    pub position: Vec4,
    pub texture_position: Vec4,
}

impl VertexType for WindowVertex {
    const NAME: &'static str = "WindowVertex";
    const ATTRIBUTE_NAMES: &'static [&'static str] = &["position", "texture_position"];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        wgpu::VertexAttribute {
            format: <Vec4 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(WindowVertex, position) as wgpu::BufferAddress,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: <Vec4 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(WindowVertex, texture_position) as wgpu::BufferAddress,
            shader_location: 1,
        },
    ];
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C, packed)]
pub struct PosTexVertex {
    pub position: Vec2,
    pub texture_position: Vec2,
}

impl VertexType for PosTexVertex {
    const NAME: &'static str = "PosTexVertex";
    const ATTRIBUTE_NAMES: &'static [&'static str] = &["position", "texture_position"];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        wgpu::VertexAttribute {
            format: <Vec2 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(PosTexVertex, position) as wgpu::BufferAddress,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: <Vec2 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(PosTexVertex, texture_position) as wgpu::BufferAddress,
            shader_location: 1,
        },
    ];
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C, packed)]
pub struct MaskVertex {
    pub position: Vec2,
    pub texture_position: Vec2,
    pub mask_position: Vec2,
}

impl VertexType for MaskVertex {
    const NAME: &'static str = "MaskVertex";
    const ATTRIBUTE_NAMES: &'static [&'static str] =
        &["position", "texture_position", "mask_position"];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        wgpu::VertexAttribute {
            format: <Vec2 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(MaskVertex, position) as wgpu::BufferAddress,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: <Vec2 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(MaskVertex, texture_position) as wgpu::BufferAddress,
            shader_location: 1,
        },
        wgpu::VertexAttribute {
            format: <Vec2 as VertexAttribute>::FORMAT,
            offset: std::mem::offset_of!(MaskVertex, mask_position) as wgpu::BufferAddress,
            shader_location: 2,
        },
    ];
}
