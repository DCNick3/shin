pub mod metadata;

use encase::ShaderType;
use glam::{Mat4, Vec4};
use shin_primitives::color::FloatColor4;

pub use crate::uniforms::metadata::UniformType;
use crate::uniforms::metadata::{
    ArraySchema, FieldSchema, PrimitiveType, StructSchema, TypeSchema,
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
    FloatColor4 => PrimitiveType::Float32x4,
    Mat4 => PrimitiveType::Float32x4x4,
    u32 => PrimitiveType::Uint32
}

impl<T, const S: usize> UniformType for [T; S]
where
    T: UniformType,
{
    const SCHEMA: TypeSchema = TypeSchema::Array(ArraySchema {
        ty: &T::SCHEMA,
        length: S as u32,
        stride: T::SCHEMA.size(),
    });
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
    pub color1: FloatColor4,
    pub color2: FloatColor4,
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
                ty: &<FloatColor4 as UniformType>::SCHEMA,
                offset: FontUniformParams::METADATA.extra.offsets[1] as u32,
            },
            FieldSchema {
                name: "color2",
                ty: &<FloatColor4 as UniformType>::SCHEMA,
                offset: FontUniformParams::METADATA.extra.offsets[2] as u32,
            },
        ],
    });
}

#[derive(ShaderType)]
pub struct FontBorderUniformParams {
    pub transform: Mat4,
    // can't pass [Vec2; 8] due to alignment requirements,
    // so distances are packed pairwise
    pub dist: [Vec4; 4],
    pub color: FloatColor4,
}

impl UniformType for FontBorderUniformParams {
    const SCHEMA: TypeSchema = TypeSchema::Struct(StructSchema {
        name: "FontBorderUniformParams",
        size: FontBorderUniformParams::METADATA.min_size.get() as u32,
        alignment: FontBorderUniformParams::METADATA.alignment.get() as u32,
        fields: &[
            FieldSchema {
                name: "transform",
                ty: &<Mat4 as UniformType>::SCHEMA,
                offset: FontBorderUniformParams::METADATA.extra.offsets[0] as u32,
            },
            FieldSchema {
                name: "dist",
                ty: &<[Vec4; 4] as UniformType>::SCHEMA,
                offset: FontBorderUniformParams::METADATA.extra.offsets[1] as u32,
            },
            FieldSchema {
                name: "color",
                ty: &<FloatColor4 as UniformType>::SCHEMA,
                offset: FontBorderUniformParams::METADATA.extra.offsets[2] as u32,
            },
        ],
    });
}

#[derive(ShaderType)]
pub struct LayerUniformParams {
    pub transform: Mat4,
    pub color: FloatColor4,
    pub fragment_param: Vec4,
    // in reality those are enums, but wgsl doesn't natively support them
    // we can probably be a bit smarter and generate constants for those but that's a paaaaaain
    pub output_type: u32,
    pub fragment_operation: u32,
}

impl UniformType for LayerUniformParams {
    const SCHEMA: TypeSchema = TypeSchema::Struct(StructSchema {
        name: "LayerUniformParams",
        size: LayerUniformParams::METADATA.min_size.get() as u32,
        alignment: LayerUniformParams::METADATA.alignment.get() as u32,
        fields: &[
            FieldSchema {
                name: "transform",
                ty: &<Mat4 as UniformType>::SCHEMA,
                offset: LayerUniformParams::METADATA.extra.offsets[0] as u32,
            },
            FieldSchema {
                name: "color",
                ty: &<FloatColor4 as UniformType>::SCHEMA,
                offset: LayerUniformParams::METADATA.extra.offsets[1] as u32,
            },
            FieldSchema {
                name: "fragment_param",
                ty: &<Vec4 as UniformType>::SCHEMA,
                offset: LayerUniformParams::METADATA.extra.offsets[2] as u32,
            },
            FieldSchema {
                name: "output_type",
                ty: &<u32 as UniformType>::SCHEMA,
                offset: LayerUniformParams::METADATA.extra.offsets[3] as u32,
            },
            FieldSchema {
                name: "fragment_operation",
                ty: &<u32 as UniformType>::SCHEMA,
                offset: LayerUniformParams::METADATA.extra.offsets[4] as u32,
            },
        ],
    });
}

#[derive(ShaderType)]
pub struct MovieUniformParams {
    pub transform: Mat4,
    pub color_bias: Vec4,
    pub color_transform: [Vec4; 3],
}

impl UniformType for MovieUniformParams {
    const SCHEMA: TypeSchema = TypeSchema::Struct(StructSchema {
        name: "MovieUniformParams",
        size: MovieUniformParams::METADATA.min_size.get() as u32,
        alignment: MovieUniformParams::METADATA.alignment.get() as u32,
        fields: &[
            FieldSchema {
                name: "transform",
                ty: &<Mat4 as UniformType>::SCHEMA,
                offset: MovieUniformParams::METADATA.extra.offsets[0] as u32,
            },
            FieldSchema {
                name: "color_bias",
                ty: &<Vec4 as UniformType>::SCHEMA,
                offset: MovieUniformParams::METADATA.extra.offsets[1] as u32,
            },
            FieldSchema {
                name: "color_transform",
                ty: &<[Vec4; 3] as UniformType>::SCHEMA,
                offset: MovieUniformParams::METADATA.extra.offsets[2] as u32,
            },
        ],
    });
}

#[derive(ShaderType)]
pub struct WiperDefaultUniformParams {
    pub transform: Mat4,
    pub alpha: Vec4,
}

impl UniformType for WiperDefaultUniformParams {
    const SCHEMA: TypeSchema = TypeSchema::Struct(StructSchema {
        name: "WiperDefaultUniformParams",
        size: WiperDefaultUniformParams::METADATA.min_size.get() as u32,
        alignment: WiperDefaultUniformParams::METADATA.alignment.get() as u32,
        fields: &[
            FieldSchema {
                name: "transform",
                ty: &<Mat4 as UniformType>::SCHEMA,
                offset: WiperDefaultUniformParams::METADATA.extra.offsets[0] as u32,
            },
            FieldSchema {
                name: "alpha",
                ty: &<Vec4 as UniformType>::SCHEMA,
                offset: WiperDefaultUniformParams::METADATA.extra.offsets[1] as u32,
            },
        ],
    });
}
