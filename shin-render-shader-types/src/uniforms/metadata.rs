use enum_iterator::Sequence;

pub trait UniformType {
    const SCHEMA: TypeSchema;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TypeSchema {
    Primitive(PrimitiveType),
    Struct(StructSchema),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Sequence)]
pub enum PrimitiveType {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Float32x4x4,
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
