use std::sync::Arc;

pub trait BufferOwnership {
    fn new(buffer: wgpu::Buffer) -> Self;
    fn get(&self) -> &wgpu::Buffer;
}

#[derive(Debug)]
pub struct Owned(wgpu::Buffer);

#[derive(Clone, Debug)]
pub struct Shared(Arc<wgpu::Buffer>);

#[derive(Debug)]
pub enum AnyOwnership {
    Owned(Box<Owned>),
    Shared(Shared),
}

impl BufferOwnership for Owned {
    fn new(buffer: wgpu::Buffer) -> Self {
        Self(buffer)
    }

    fn get(&self) -> &wgpu::Buffer {
        &self.0
    }
}

impl BufferOwnership for Shared {
    fn new(buffer: wgpu::Buffer) -> Self {
        Self(Arc::new(buffer))
    }

    fn get(&self) -> &wgpu::Buffer {
        &self.0
    }
}

impl BufferOwnership for AnyOwnership {
    fn new(_buffer: wgpu::Buffer) -> Self {
        panic!("Do not create a buffer with AnyOwnership directly, use a specific type instead")
    }

    fn get(&self) -> &wgpu::Buffer {
        match self {
            AnyOwnership::Owned(owned) => owned.get(),
            AnyOwnership::Shared(shared) => shared.get(),
        }
    }
}
