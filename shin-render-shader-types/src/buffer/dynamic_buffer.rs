use std::sync::Arc;

use crate::{
    buffer::{
        bytes_address::BytesAddress,
        ownership::BufferOwnership,
        types::{
            ArrayBufferType, BufferType, RawMarker, StructBufferType, UniformMarker, VertexMarker,
        },
        Buffer, BufferUsage, SharedBuffer,
    },
    vertices::VertexType,
};

/// Dynamically allocates space in a gpu buffer, mostly used for submitting uniform data
pub struct DynamicBuffer {
    queue: Arc<wgpu::Queue>,
    chunk_size: BytesAddress,
    position: BytesAddress,
    buffer: SharedBuffer<RawMarker>,
}

impl DynamicBuffer {
    const ALIGNMENT: BytesAddress = BytesAddress::new(16);

    pub fn new(device: &wgpu::Device, queue: Arc<wgpu::Queue>, chunk_size: BytesAddress) -> Self {
        let buffer = Buffer::allocate_raw(
            device,
            chunk_size,
            BufferUsage::DynamicBuffer,
            Some("DynamicBuffer"),
        );

        Self {
            queue,
            chunk_size,
            position: BytesAddress::ZERO,
            buffer,
        }
    }

    fn get_with_raw_data(
        &mut self,
        alignment: BytesAddress,
        data: &[u8],
    ) -> SharedBuffer<RawMarker> {
        let offset = self.position.align_to(alignment);
        let size = BytesAddress::new(data.len() as _);

        if self.position + size > self.chunk_size {
            todo!("allocate a new buffer")
        }

        assert!(RawMarker::is_valid_offset(offset));
        assert!(RawMarker::is_valid_size(size));

        self.position = (offset + size).align_to(Self::ALIGNMENT);

        self.queue
            .write_buffer(self.buffer.ownership.get(), offset.get(), data);

        self.buffer.slice_bytes(offset, size)
    }

    pub fn get_with_struct_data<T: StructBufferType>(
        &mut self,
        data: &T::Value,
    ) -> SharedBuffer<T> {
        // can't use a statically-sized array here because of `<T::Value as encase::ShaderSize>::SHADER_SIZE.get() as usize`
        let mut buffer = vec![0u8; <T::Value as encase::ShaderSize>::SHADER_SIZE.get() as usize];

        let mut buffer = encase::UniformBuffer::new(buffer.as_mut_slice());
        buffer.write(data).unwrap();
        let buffer = buffer.into_inner();

        self.get_with_raw_data(T::MIN_ALIGNMENT, buffer).downcast()
    }

    pub fn get_with_slice_data<T: ArrayBufferType>(
        &mut self,
        data: &[T::Element],
    ) -> SharedBuffer<T> {
        let data: &[u8] = bytemuck::cast_slice(data);

        self.get_with_raw_data(T::MIN_ALIGNMENT, data).downcast()
    }

    pub fn get_uniform_with_data<T: encase::ShaderSize + encase::internal::WriteInto>(
        &mut self,
        data: &T,
    ) -> SharedBuffer<UniformMarker<T>> {
        self.get_with_struct_data(data)
    }

    pub fn get_vertex_with_data<T: VertexType>(
        &mut self,
        data: &[T],
    ) -> SharedBuffer<VertexMarker<T>> {
        self.get_with_slice_data(data)
    }
}
