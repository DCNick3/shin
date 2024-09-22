pub mod metadata;

use encase::ShaderType;
use glam::{Mat4, Vec4};

pub use crate::uniforms::metadata::UniformType;
use crate::{
    uniforms::metadata::{FieldSchema, PrimitiveType, StructSchema, TypeSchema},
    vertices::FloatColor4,
};

macro_rules! impl_primitive {
    ($($ty:ty => $v:expr),*) => {
        $(
            impl UniformType for $ty {
                const SCHEMA: TypeSchema = TypeSchema::Primitive($v);
            }
        )*
    };
}

impl_primitive! {
    f32 => PrimitiveType::Float32,
    Vec4 => PrimitiveType::Float32x4,
    Mat4 => PrimitiveType::Float32x4x4
}

#[derive(ShaderType)]
pub struct ClearUniformParams {
    pub color: FloatColor4,
}

impl UniformType for ClearUniformParams {
    const SCHEMA: TypeSchema = TypeSchema::Struct(StructSchema {
        name: "ClearUniformParams",
        size: ClearUniformParams::METADATA.min_size.get() as u32,
        alignment: ClearUniformParams::METADATA.alignment.get() as u32,
        fields: &[FieldSchema {
            name: "color",
            ty: &<Vec4 as UniformType>::SCHEMA,
            offset: ClearUniformParams::METADATA.extra.offsets[0] as u32,
        }],
    });
}

#[derive(ShaderType)]
pub struct FillUniformParams {
    pub transform: Mat4,
}

impl UniformType for FillUniformParams {
    const SCHEMA: TypeSchema = TypeSchema::Struct(StructSchema {
        name: "FillUniformParams",
        size: FillUniformParams::METADATA.min_size.get() as u32,
        alignment: FillUniformParams::METADATA.alignment.get() as u32,
        fields: &[FieldSchema {
            name: "transform",
            ty: &<Mat4 as UniformType>::SCHEMA,
            offset: FillUniformParams::METADATA.extra.offsets[0] as u32,
        }],
    });
}

#[derive(ShaderType)]
pub struct SpriteUniformParams {
    pub transform: Mat4,
}

impl UniformType for SpriteUniformParams {
    const SCHEMA: TypeSchema = TypeSchema::Struct(StructSchema {
        name: "SpriteUniformParams",
        size: SpriteUniformParams::METADATA.min_size.get() as u32,
        alignment: SpriteUniformParams::METADATA.alignment.get() as u32,
        fields: &[FieldSchema {
            name: "transform",
            ty: &<Mat4 as UniformType>::SCHEMA,
            offset: SpriteUniformParams::METADATA.extra.offsets[0] as u32,
        }],
    });
}

#[derive(ShaderType)]
pub struct FontUniformParams {
    pub transform: Mat4,
    pub color1: Vec4,
    pub color2: Vec4,
}

impl UniformType for FontUniformParams {
    const SCHEMA: TypeSchema = TypeSchema::Struct(StructSchema {
        name: "FontUniformParams",
        size: FontUniformParams::METADATA.min_size.get() as u32,
        alignment: FontUniformParams::METADATA.alignment.get() as u32,
        fields: &[
            FieldSchema {
                name: "transform",
                ty: &<Mat4 as UniformType>::SCHEMA,
                offset: FontUniformParams::METADATA.extra.offsets[0] as u32,
            },
            FieldSchema {
                name: "color1",
                ty: &<Vec4 as UniformType>::SCHEMA,
                offset: FontUniformParams::METADATA.extra.offsets[1] as u32,
            },
            FieldSchema {
                name: "color2",
                ty: &<Vec4 as UniformType>::SCHEMA,
                offset: FontUniformParams::METADATA.extra.offsets[2] as u32,
            },
        ],
    });
}
