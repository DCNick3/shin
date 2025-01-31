use crate::{
    buffer::{
        bytes_address::BytesAddress,
        types::{
            ArrayBufferType, IndexMarker, RawMarker, StructBufferType, UniformMarker, VertexMarker,
        },
        SharedBuffer,
    },
    vertices::VertexType,
};

pub trait DynamicBufferBackend {
    fn get_with_raw_data(
        &mut self,
        alignment: BytesAddress,
        data: &[u8],
    ) -> SharedBuffer<RawMarker>;

    fn get_with_struct_data<T: StructBufferType>(&mut self, data: &T::Value) -> SharedBuffer<T> {
        // can't use a statically-sized array here because of `<T::Value as encase::ShaderSize>::SHADER_SIZE.get() as usize`
        let mut buffer = vec![0u8; <T::Value as encase::ShaderSize>::SHADER_SIZE.get() as usize];

        let mut buffer = encase::UniformBuffer::new(buffer.as_mut_slice());
        buffer.write(data).unwrap();
        let buffer = buffer.into_inner();

        self.get_with_raw_data(T::OFFSET_ALIGNMENT, buffer)
            .downcast()
    }

    fn get_with_slice_data<T: ArrayBufferType>(&mut self, data: &[T::Element]) -> SharedBuffer<T> {
        let data: &[u8] = bytemuck::cast_slice(data);

        self.get_with_raw_data(T::OFFSET_ALIGNMENT, data).downcast()
    }

    fn get_uniform_with_data<T: encase::ShaderSize + encase::internal::WriteInto>(
        &mut self,
        data: &T,
    ) -> SharedBuffer<UniformMarker<T>> {
        self.get_with_struct_data(data)
    }

    fn get_vertex_with_data<T: VertexType>(&mut self, data: &[T]) -> SharedBuffer<VertexMarker<T>> {
        self.get_with_slice_data(data)
    }

    fn get_index_with_data(&mut self, data: &[u16]) -> SharedBuffer<IndexMarker> {
        self.get_with_slice_data(data)
    }
}
