use enum_iterator::Sequence;

pub trait UniformType {
    const SCHEMA: TypeSchema;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TypeSchema {
    Primitive(PrimitiveType),
    Struct(StructSchema),
    Array(ArraySchema),
}

impl TypeSchema {
    pub const fn size(&self) -> u32 {
        match self {
            TypeSchema::Primitive(ty) => ty.size(),
            TypeSchema::Struct(schema) => schema.size,
            TypeSchema::Array(schema) => schema.stride * schema.length,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Sequence)]
pub enum PrimitiveType {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Float32x4x4,
    Uint32,
}

impl PrimitiveType {
    pub const fn size(&self) -> u32 {
        match self {
            PrimitiveType::Float32 => 4,
            PrimitiveType::Float32x2 => 8,
            PrimitiveType::Float32x3 => 12,
            PrimitiveType::Float32x4 => 16,
            PrimitiveType::Float32x4x4 => 64,
            PrimitiveType::Uint32 => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StructSchema {
    pub name: &'static str,
    pub size: u32,
    pub alignment: u32,
    pub fields: &'static [FieldSchema],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FieldSchema {
    pub name: &'static str,
    pub ty: &'static TypeSchema,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ArraySchema {
    pub ty: &'static TypeSchema,
    pub length: u32,
    pub stride: u32,
}
