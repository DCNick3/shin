use std::sync::{Arc, RwLock};

use dpi::{PhysicalPosition, PhysicalSize};

pub trait SizeAspect: PartialEq + Copy + From<PhysicalSize<u32>> + Into<PhysicalSize<u32>> {
    fn get(params: &ViewportParams) -> Self;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

impl From<PhysicalSize<u32>> for SurfaceSize {
    fn from(size: PhysicalSize<u32>) -> Self {
        SurfaceSize {
            width: size.width,
            height: size.height,
        }
    }
}

impl Into<PhysicalSize<u32>> for SurfaceSize {
    fn into(self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.width, self.height)
    }
}

impl SizeAspect for SurfaceSize {
    fn get(params: &ViewportParams) -> Self {
        params.surface_size
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct CanvasSize {
    pub width: u32,
    pub height: u32,
}

impl From<PhysicalSize<u32>> for CanvasSize {
    fn from(size: PhysicalSize<u32>) -> Self {
        CanvasSize {
            width: size.width,
            height: size.height,
        }
    }
}

impl Into<PhysicalSize<u32>> for CanvasSize {
    fn into(self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.width, self.height)
    }
}

impl SizeAspect for CanvasSize {
    fn get(params: &ViewportParams) -> Self {
        params.canvas_size
    }
}

/// Stores the info of the target sizes for the render stack
///
/// This stores two different sizes: the surface size and the canvas size.
///
/// The surface size is the size of the OS-provided surface, corresponding to the screen size of the device or window.
///
/// The canvas size is the size of the contents the application wants to render. This can often be different from the surface size to keep the aspect ratio of the content.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ViewportParams {
    pub surface_size: SurfaceSize,
    pub canvas_offset: PhysicalPosition<u32>,
    pub canvas_size: CanvasSize,
}

impl ViewportParams {
    pub fn both(size: PhysicalSize<u32>) -> Self {
        ViewportParams {
            surface_size: size.into(),
            canvas_offset: PhysicalPosition::default(),
            canvas_size: size.into(),
        }
    }

    pub fn with_aspect_ratio(size: PhysicalSize<u32>, aspect_ratio: f64) -> Self {
        let canvas_size = if size.width as f64 / size.height as f64 > aspect_ratio {
            PhysicalSize::new((size.height as f64 * aspect_ratio) as u32, size.height)
        } else {
            PhysicalSize::new(size.width, (size.width as f64 / aspect_ratio) as u32)
        };

        let canvas_offset = PhysicalPosition::new(
            (size.width - canvas_size.width) / 2,
            (size.height - canvas_size.height) / 2,
        );

        ViewportParams {
            surface_size: size.into(),
            canvas_offset,
            canvas_size: canvas_size.into(),
        }
    }

    pub fn get<Aspect: SizeAspect>(&self) -> Aspect {
        Aspect::get(self)
    }
}

pub struct SurfaceResizeSource {
    inner: Arc<RwLock<ViewportParams>>,
}

impl SurfaceResizeSource {
    pub fn new(size: ViewportParams) -> Self {
        SurfaceResizeSource {
            inner: Arc::new(RwLock::new(size)),
        }
    }

    pub fn resize(&self, size: ViewportParams) {
        *self.inner.write().unwrap() = size;
    }

    pub fn handle<Aspect: SizeAspect + Default>(&self) -> ResizeHandle<Aspect> {
        ResizeHandle {
            inner: self.inner.clone(),
            last_known_size: Default::default(),
        }
    }

    pub fn surface_handle(&self) -> ResizeHandle<SurfaceSize> {
        self.handle()
    }

    pub fn canvas_handle(&self) -> ResizeHandle<CanvasSize> {
        self.handle()
    }
}

#[derive(Debug, Clone)]
pub struct ResizeHandle<Aspect> {
    inner: Arc<RwLock<ViewportParams>>,
    last_known_size: Aspect,
}

impl<Aspect: SizeAspect> ResizeHandle<Aspect> {
    pub fn get(&mut self) -> Aspect {
        let size = self.inner.read().unwrap().get::<Aspect>();
        self.last_known_size = size;
        size
    }

    pub fn update(&mut self) -> Option<Aspect> {
        let size = self.inner.read().unwrap().get::<Aspect>();
        if size != self.last_known_size {
            self.last_known_size = size;
            Some(size)
        } else {
            None
        }
    }
}

impl ResizeHandle<SurfaceSize> {
    /// Get size and offset parameters for [`wgpu::RenderPass::set_viewport`] to render into the specified canvas region.  
    pub fn get_viewport(&self) -> (f32, f32, f32, f32) {
        let params = self.inner.read().unwrap();
        let canvas_offset = params.canvas_offset;
        let canvas_size = params.canvas_size;

        let x = canvas_offset.x as f32;
        let y = canvas_offset.y as f32;
        let width = canvas_size.width as f32;
        let height = canvas_size.height as f32;

        (x, y, width, height)
    }
}
