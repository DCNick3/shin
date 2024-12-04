use std::marker::PhantomData;

use crate::{buffer::BytesAddress, vertices::VertexType};

// TODO: this is very conservative, maybe we can find a way to relax this at runtime somehow
pub const MIN_UNIFORM_BUFFER_ALIGNMENT: BytesAddress = BytesAddress::new(256);

pub trait BufferType {
    const MIN_ALIGNMENT: BytesAddress;

    fn is_valid_offset(offset: BytesAddress) -> bool;

    fn is_valid_size(size: BytesAddress) -> bool;
}

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
    const MIN_ALIGNMENT: BytesAddress = BytesAddress::new(4);

    fn is_valid_offset(offset: BytesAddress) -> bool {
        offset.is_aligned_to(Self::MIN_ALIGNMENT)
    }

    fn is_valid_size(size: BytesAddress) -> bool {
        size.is_aligned_to(Self::MIN_ALIGNMENT)
    }
}
impl ArrayBufferType for RawMarker {
    type Element = u8;
}

impl<T: VertexType> BufferType for VertexMarker<T> {
    const MIN_ALIGNMENT: BytesAddress = if std::mem::align_of::<T>() < 4 {
        BytesAddress::new(4)
    } else {
        BytesAddress::from_usize(std::mem::align_of::<T>())
    };

    fn is_valid_offset(offset: BytesAddress) -> bool {
        offset.is_aligned_to(Self::MIN_ALIGNMENT)
    }

    fn is_valid_size(size: BytesAddress) -> bool {
        size.is_aligned_to(BytesAddress::from_usize(std::mem::size_of::<T>()))
    }
}
impl<T: VertexType> ArrayBufferType for VertexMarker<T> {
    type Element = T;
}

impl BufferType for IndexMarker {
    const MIN_ALIGNMENT: BytesAddress = BytesAddress::new(4);

    fn is_valid_offset(offset: BytesAddress) -> bool {
        offset.is_aligned_to(Self::MIN_ALIGNMENT)
    }

    fn is_valid_size(size: BytesAddress) -> bool {
        // TODO: what if the byte size is 2*m, where m is odd?
        // seems like a valid size of u16 array to me...
        size.is_aligned_to(Self::MIN_ALIGNMENT)
    }
}
impl ArrayBufferType for IndexMarker {
    type Element = u16;
}

impl<T: encase::ShaderType + encase::ShaderSize> BufferType for UniformMarker<T> {
    const MIN_ALIGNMENT: BytesAddress = MIN_UNIFORM_BUFFER_ALIGNMENT;

    fn is_valid_offset(offset: BytesAddress) -> bool {
        offset.is_aligned_to(Self::MIN_ALIGNMENT)
    }

    fn is_valid_size(size: BytesAddress) -> bool {
        size == BytesAddress::new(T::SHADER_SIZE.get())
    }
}
impl<T: encase::ShaderSize + encase::internal::WriteInto> StructBufferType for UniformMarker<T> {
    type Value = T;
}
