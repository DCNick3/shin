mod bytes_address;
mod dynamic_buffer;
pub mod ownership;
pub mod types;

use std::marker::PhantomData;

use ownership::{AnyOwnership, BufferOwnership, Owned, Shared};
use types::BufferType;
use wgpu::util::DeviceExt as _;

pub use self::{bytes_address::BytesAddress, dynamic_buffer::DynamicBufferBackend};
use crate::{
    buffer::types::{ArrayBufferType, IndexMarker, RawMarker, VertexMarker},
    vertices::VertexType,
};

pub enum BufferUsage {
    /// COPY_DST | INDEX | VERTEX | UNIFORM
    DynamicBuffer,
    /// VERTEX
    Vertex,
    /// INDEX
    Index,
}

impl From<BufferUsage> for wgpu::BufferUsages {
    fn from(value: BufferUsage) -> Self {
        match value {
            BufferUsage::DynamicBuffer => {
                wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::UNIFORM
            }
            BufferUsage::Vertex => wgpu::BufferUsages::VERTEX,
            BufferUsage::Index => wgpu::BufferUsages::INDEX,
        }
    }
}

#[derive(Debug)]
pub struct Buffer<O: BufferOwnership, T: BufferType> {
    ownership: O,
    offset: BytesAddress,
    size: BytesAddress,
    phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct BufferRef<'a, T: BufferType> {
    slice: wgpu::BufferSlice<'a>,
    size: BytesAddress,
    phantom: PhantomData<T>,
}

impl<O: BufferOwnership, T: BufferType> Buffer<O, T> {
    // TODO: perhaps make it private?
    pub fn allocate_raw(
        device: &wgpu::Device,
        size_bytes: BytesAddress,
        usage: BufferUsage,
        label: Option<&str>,
    ) -> Self {
        let offset = BytesAddress::new(0);
        let size = size_bytes;

        assert!(T::is_valid_offset(offset));
        assert!(T::is_valid_size(size));

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: size.get(),
            usage: usage.into(),
            mapped_at_creation: false,
        });

        Buffer {
            ownership: O::new(buffer),
            offset,
            size,
            phantom: PhantomData,
        }
    }

    // TODO: it would be nice to support mapping the typed buffer and allowing the API user to write to it directly
    // instead of building the whole buffer in memory and then copying it to the GPU
    pub fn allocate_raw_with_contents(
        device: &wgpu::Device,
        contents: &[u8],
        usage: BufferUsage,
        label: Option<&str>,
    ) -> Self {
        let offset = BytesAddress::new(0);
        let size = BytesAddress::new(contents.len() as _);

        assert!(T::is_valid_offset(offset));
        assert!(T::is_valid_size(size));

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents,
            usage: usage.into(),
        });

        Buffer {
            ownership: O::new(buffer),
            offset,
            size,
            phantom: PhantomData,
        }
    }

    pub fn from_wgpu_buffer(buffer: wgpu::Buffer) -> Self {
        let offset = BytesAddress::new(0);
        let size = BytesAddress::new(buffer.size());

        assert!(T::is_valid_offset(offset));
        assert!(T::is_valid_size(size));

        Buffer {
            ownership: O::new(buffer),
            offset,
            size,
            phantom: PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, offset: BytesAddress, data: &[u8]) {
        queue.write_buffer(self.ownership.get(), offset.get(), data);
    }

    pub fn as_buffer_ref(&self) -> BufferRef<T> {
        let slice = self
            .ownership
            .get()
            .slice(self.offset.get()..(self.offset + self.size).get());

        BufferRef {
            slice,
            size: self.size,
            phantom: PhantomData,
        }
    }

    pub fn as_buffer_binding(&self) -> wgpu::BufferBinding {
        let offset = self.offset.get();
        let size = self.size.get();

        wgpu::BufferBinding {
            buffer: self.ownership.get(),
            offset,
            size: Some(wgpu::BufferSize::new(size).unwrap()),
        }
    }
}

impl<O: BufferOwnership, T: ArrayBufferType> Buffer<O, T> {
    pub fn as_sliced_buffer_ref(&self, offset: usize, size: usize) -> BufferRef<T> {
        let element_size = size_of::<T::Element>();

        // convert array offset and size into bytes
        let offset = BytesAddress::from_usize(offset * element_size);
        let size = BytesAddress::from_usize(size * element_size);

        // check if we are within the bounds of the buffer
        assert!((BytesAddress::ZERO..self.size).contains(&offset));
        assert!((BytesAddress::ZERO..=self.size).contains(&(offset + size)));

        let new_offset = self.offset + offset;

        let slice = self
            .ownership
            .get()
            .slice(new_offset.get()..(new_offset + size).get());

        BufferRef {
            slice,
            size,
            phantom: PhantomData,
        }
    }

    pub fn count(&self) -> u32 {
        (self.size.get() as usize / size_of::<T::Element>()) as u32
    }
}

impl<T: ArrayBufferType> BufferRef<'_, T> {
    pub fn count(&self) -> u32 {
        (self.size.get() as usize / size_of::<T::Element>()) as u32
    }
}

pub type OwnedBuffer<T> = Buffer<Owned, T>;
pub type SharedBuffer<T> = Buffer<Shared, T>;
pub type AnyBuffer<T> = Buffer<AnyOwnership, T>;

pub type OwnedVertexBuffer<T> = OwnedBuffer<VertexMarker<T>>;
pub type OwnedIndexBuffer = OwnedBuffer<IndexMarker>;

pub type AnyVertexBuffer<T> = AnyBuffer<VertexMarker<T>>;
pub type AnyIndexBuffer = AnyBuffer<IndexMarker>;

pub type VertexBufferRef<'a, T> = BufferRef<'a, VertexMarker<T>>;
pub type IndexBufferRef<'a> = BufferRef<'a, IndexMarker>;

impl<T: BufferType> SharedBuffer<T> {
    pub fn slice_bytes(&self, start: BytesAddress, size: BytesAddress) -> Self {
        let ownership = self.ownership.clone();

        let offset = self.offset + start;

        assert!((self.offset..self.offset + self.size).contains(&offset));
        assert!((self.offset..self.offset + self.size).contains(&(offset + size)));

        assert!(T::is_valid_offset(offset));
        assert!(T::is_valid_size(size));

        Self {
            ownership,
            offset,
            size,
            phantom: Default::default(),
        }
    }
}

impl<T: ArrayBufferType> OwnedBuffer<T> {
    pub fn allocate_with_array_contents(
        device: &wgpu::Device,
        data: &[T::Element],
        usage: BufferUsage,
        label: Option<&str>,
    ) -> Self {
        let data: &[u8] = bytemuck::cast_slice(data);

        Self::allocate_raw_with_contents(device, data, usage, label)
    }
}

impl<T: VertexType> OwnedBuffer<VertexMarker<T>> {
    pub fn allocate_vertex(device: &wgpu::Device, data: &[T], label: Option<&str>) -> Self {
        Self::allocate_with_array_contents(device, data, BufferUsage::Vertex, label)
    }
}

impl OwnedBuffer<IndexMarker> {
    pub fn allocate_index(device: &wgpu::Device, data: &[u16], label: Option<&str>) -> Self {
        Self::allocate_with_array_contents(device, data, BufferUsage::Index, label)
    }
}

impl<T: BufferType> From<OwnedBuffer<T>> for AnyBuffer<T> {
    fn from(value: OwnedBuffer<T>) -> Self {
        AnyBuffer {
            ownership: AnyOwnership::Owned(Box::new(value.ownership)),
            offset: value.offset,
            size: value.size,
            phantom: Default::default(),
        }
    }
}

impl<T: BufferType> From<SharedBuffer<T>> for AnyBuffer<T> {
    fn from(value: SharedBuffer<T>) -> Self {
        AnyBuffer {
            ownership: AnyOwnership::Shared(value.ownership.clone()),
            offset: value.offset,
            size: value.size,
            phantom: Default::default(),
        }
    }
}

impl<O: BufferOwnership> Buffer<O, RawMarker> {
    pub fn downcast<T: BufferType>(self) -> Buffer<O, T> {
        assert!(T::is_valid_offset(self.offset));
        assert!(T::is_valid_size(self.size));
        Buffer {
            ownership: self.ownership,
            offset: self.offset,
            size: self.size,
            phantom: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum VertexSource<'a, T: VertexType> {
    VertexBuffer {
        vertices: VertexBufferRef<'a, T>,
    },
    VertexAndIndexBuffer {
        vertices: VertexBufferRef<'a, T>,
        indices: IndexBufferRef<'a>,
    },
    VertexData {
        vertices: &'a [T],
    },
    VertexAndIndexData {
        vertices: &'a [T],
        indices: &'a [u16],
    },
}

/// Information necessary to make a right call to `draw` or `draw_indexed` after binding the vertex source.
#[derive(Debug)]
pub enum VertexSourceInfo {
    VertexBuffer { vertex_count: u32 },
    VertexAndIndexBuffer { index_count: u32 },
}

impl<T: VertexType> VertexSource<'_, T> {
    pub fn info(&self) -> VertexSourceInfo {
        match self {
            VertexSource::VertexBuffer {
                vertices: vertex_buffer,
            } => VertexSourceInfo::VertexBuffer {
                vertex_count: vertex_buffer.count(),
            },
            VertexSource::VertexAndIndexBuffer {
                vertices: _,
                indices: index_buffer,
            } => VertexSourceInfo::VertexAndIndexBuffer {
                index_count: index_buffer.count(),
            },
            VertexSource::VertexData {
                vertices: vertex_data,
            } => VertexSourceInfo::VertexBuffer {
                vertex_count: vertex_data.len() as u32,
            },
            VertexSource::VertexAndIndexData {
                vertices: _,
                indices: index_data,
            } => VertexSourceInfo::VertexAndIndexBuffer {
                index_count: index_data.len() as u32,
            },
        }
    }

    pub fn bind(
        &self,
        dynamic_buffer: &mut impl DynamicBufferBackend,
        pass: &mut wgpu::RenderPass,
    ) {
        match self {
            VertexSource::VertexBuffer {
                vertices: vertex_buffer,
            } => {
                pass.set_vertex_buffer(0, vertex_buffer.slice);
            }
            VertexSource::VertexAndIndexBuffer {
                vertices: vertex_buffer,
                indices: index_buffer,
            } => {
                pass.set_vertex_buffer(0, vertex_buffer.slice);
                pass.set_index_buffer(index_buffer.slice, wgpu::IndexFormat::Uint16);
            }
            VertexSource::VertexData {
                vertices: vertex_data,
            } => {
                let vertex_buffer = dynamic_buffer.get_vertex_with_data(vertex_data);
                pass.set_vertex_buffer(0, vertex_buffer.as_buffer_ref().slice);
            }
            VertexSource::VertexAndIndexData {
                vertices: vertex_data,
                indices: index_data,
            } => {
                let vertex_buffer = dynamic_buffer.get_vertex_with_data(vertex_data);
                let index_buffer = dynamic_buffer.get_index_with_data(index_data);
                pass.set_vertex_buffer(0, vertex_buffer.as_buffer_ref().slice);
                pass.set_index_buffer(
                    index_buffer.as_buffer_ref().slice,
                    wgpu::IndexFormat::Uint16,
                );
            }
        }
    }
}
