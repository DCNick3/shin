use std::sync::{Arc, RwLock};

use winit::dpi::PhysicalSize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

impl From<PhysicalSize<u32>> for SurfaceSize {
    fn from(value: PhysicalSize<u32>) -> Self {
        let (width, height) = value.into();
        SurfaceSize { width, height }
    }
}

impl From<SurfaceSize> for wgpu::Extent3d {
    fn from(value: SurfaceSize) -> Self {
        wgpu::Extent3d {
            width: value.width,
            height: value.height,
            depth_or_array_layers: 1,
        }
    }
}

pub struct SurfaceResizeSource {
    inner: Arc<RwLock<SurfaceSize>>,
}

impl SurfaceResizeSource {
    pub fn new(size: SurfaceSize) -> Self {
        SurfaceResizeSource {
            inner: Arc::new(RwLock::new(size)),
        }
    }

    pub fn resize(&self, size: SurfaceSize) {
        *self.inner.write().unwrap() = size;
    }

    pub fn handle(&self) -> SurfaceResizeHandle {
        SurfaceResizeHandle {
            inner: self.inner.clone(),
            last_known_size: SurfaceSize::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceResizeHandle {
    inner: Arc<RwLock<SurfaceSize>>,
    last_known_size: SurfaceSize,
}

impl SurfaceResizeHandle {
    pub fn get(&mut self) -> SurfaceSize {
        let size = *self.inner.read().unwrap();
        self.last_known_size = size;
        size
    }

    pub fn update(&mut self) -> Option<SurfaceSize> {
        let size = *self.inner.read().unwrap();
        if size != self.last_known_size {
            self.last_known_size = size;
            Some(size)
        } else {
            None
        }
    }
}
