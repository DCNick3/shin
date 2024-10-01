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
            buffer: &self.ownership.get(),
            offset,
            size: Some(wgpu::BufferSize::new(size).unwrap()),
        }
    }
}

impl<O: BufferOwnership, T: ArrayBufferType> Buffer<O, T> {
    pub fn count(&self) -> u32 {
        (self.size.get() as usize / size_of::<T::Element>()) as u32
    }
}

impl<'a, T: ArrayBufferType> BufferRef<'a, T> {
    pub fn count(&self) -> u32 {
        (self.size.get() as usize / size_of::<T::Element>()) as u32
    }
}

pub type OwnedBuffer<T> = Buffer<Owned, T>;
pub type SharedBuffer<T> = Buffer<Shared, T>;
pub type AnyBuffer<T> = Buffer<AnyOwnership, T>;

pub type AnyVertexBuffer<T> = AnyBuffer<VertexMarker<T>>;
pub type AnyIndexBuffer = AnyBuffer<IndexMarker>;

pub type VertexBufferRef<'a, T> = BufferRef<'a, VertexMarker<T>>;
pub type IndexBufferRef<'a> = BufferRef<'a, IndexMarker>;

impl<T: BufferType> SharedBuffer<T> {
    pub fn slice_bytes(&self, start: BytesAddress, size: BytesAddress) -> Self {
        let ownership = self.ownership.clone();

        let offset = self.offset + start;
        let size = size;

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
        vertex_buffer: VertexBufferRef<'a, T>,
    },
    VertexAndIndexBuffer {
        vertex_buffer: VertexBufferRef<'a, T>,
        index_buffer: IndexBufferRef<'a>,
    },
    VertexData {
        vertex_data: &'a [T],
    },
    VertexAndIndexData {
        vertex_data: &'a [T],
        index_data: &'a [u16],
    },
}

/// Information necessary to make a right call to `draw` or `draw_indexed` after binding the vertex source.
#[derive(Debug)]
pub enum VertexSourceInfo {
    VertexBuffer { vertex_count: u32 },
    VertexAndIndexBuffer { index_count: u32 },
}

impl<'a, T: VertexType> VertexSource<'a, T> {
    pub fn info(&self) -> VertexSourceInfo {
        match self {
            VertexSource::VertexBuffer { vertex_buffer } => VertexSourceInfo::VertexBuffer {
                vertex_count: vertex_buffer.count(),
            },
            VertexSource::VertexAndIndexBuffer {
                vertex_buffer: _,
                index_buffer,
            } => VertexSourceInfo::VertexAndIndexBuffer {
                index_count: index_buffer.count(),
            },
            VertexSource::VertexData { vertex_data } => VertexSourceInfo::VertexBuffer {
                vertex_count: vertex_data.len() as u32,
            },
            VertexSource::VertexAndIndexData {
                vertex_data: _,
                index_data,
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
            VertexSource::VertexBuffer { vertex_buffer } => {
                pass.set_vertex_buffer(0, vertex_buffer.slice);
            }
            VertexSource::VertexAndIndexBuffer {
                vertex_buffer,
                index_buffer,
            } => {
                pass.set_vertex_buffer(0, vertex_buffer.slice);
                pass.set_index_buffer(index_buffer.slice, wgpu::IndexFormat::Uint16);
            }
            VertexSource::VertexData { vertex_data } => {
                let vertex_buffer = dynamic_buffer.get_vertex_with_data(vertex_data);
                pass.set_vertex_buffer(0, vertex_buffer.as_buffer_ref().slice);
            }
            VertexSource::VertexAndIndexData {
                vertex_data,
                index_data,
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
