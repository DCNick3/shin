use std::marker::PhantomData;

use crate::{buffer::BytesAddress, vertices::VertexType};

// TODO: this is very conservative, maybe we can find a way to relax this at runtime somehow
pub const MIN_UNIFORM_BUFFER_ALIGNMENT: BytesAddress = BytesAddress::new(256);

pub trait BufferType {
    const OFFSET_ALIGNMENT: BytesAddress;
    const LOGICAL_SIZE_STRIDE: BytesAddress;
    const IS_ARRAY_TYPE: bool;

    fn is_valid_offset(offset: BytesAddress) -> bool {
        offset.is_aligned_to(Self::OFFSET_ALIGNMENT)
    }

    fn is_valid_logical_size(size: BytesAddress) -> bool {
        if Self::IS_ARRAY_TYPE {
            size.is_aligned_to(Self::LOGICAL_SIZE_STRIDE)
        } else {
            size == Self::LOGICAL_SIZE_STRIDE
        }
    }
}

// I would like to impose bounds in these on IS_ARRAY_TYPE, but it doesn't seem possible with todays rust
// https://github.com/rust-lang/rfcs/issues/3095
pub trait ArrayBufferType: BufferType {
    type Element: bytemuck::NoUninit;
}
pub trait StructBufferType: BufferType {
    type Value: encase::ShaderSize + encase::internal::WriteInto;
}

#[derive(Debug)]
pub struct RawMarker;

/// Represents a typed vertex buffer: an array of vertices
#[derive(Debug)]
pub struct VertexMarker<T: VertexType>(PhantomData<T>);

/// Represents an index buffer: an array of 16-big unsigned indices
#[derive(Debug)]
pub struct IndexMarker;

/// Represents a typed uniform buffer: a single instance of a struct to be passed to a shader
#[derive(Debug)]
pub struct UniformMarker<T>(PhantomData<T>);

impl BufferType for RawMarker {
    const OFFSET_ALIGNMENT: BytesAddress = BytesAddress::new(4);
    const LOGICAL_SIZE_STRIDE: BytesAddress = BytesAddress::new(1);
    const IS_ARRAY_TYPE: bool = true;
}
impl ArrayBufferType for RawMarker {
    type Element = u8;
}

impl<T: VertexType> BufferType for VertexMarker<T> {
    const OFFSET_ALIGNMENT: BytesAddress = if std::mem::align_of::<T>() < 4 {
        BytesAddress::new(4)
    } else {
        BytesAddress::from_usize(std::mem::align_of::<T>())
    };
    const LOGICAL_SIZE_STRIDE: BytesAddress = BytesAddress::from_usize(std::mem::size_of::<T>());
    const IS_ARRAY_TYPE: bool = true;
}
impl<T: VertexType> ArrayBufferType for VertexMarker<T> {
    type Element = T;
}

impl BufferType for IndexMarker {
    const OFFSET_ALIGNMENT: BytesAddress = BytesAddress::new(4);
    const LOGICAL_SIZE_STRIDE: BytesAddress = BytesAddress::new(2);
    const IS_ARRAY_TYPE: bool = true;
}
impl ArrayBufferType for IndexMarker {
    type Element = u16;
}

impl<T: encase::ShaderType + encase::ShaderSize> BufferType for UniformMarker<T> {
    const OFFSET_ALIGNMENT: BytesAddress = MIN_UNIFORM_BUFFER_ALIGNMENT;
    const LOGICAL_SIZE_STRIDE: BytesAddress = BytesAddress::new(T::SHADER_SIZE.get());
    const IS_ARRAY_TYPE: bool = true;
}
impl<T: encase::ShaderSize + encase::internal::WriteInto> StructBufferType for UniformMarker<T> {
    type Value = T;
}
