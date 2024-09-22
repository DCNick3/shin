use std::sync::{Arc, RwLock};

use anyhow::Context;
use tracing::{debug, info};
use wgpu::SurfaceTarget;

use crate::new_render::resize::{SurfaceResizeHandle, SurfaceSize};

pub struct ResizeableSurface<'window> {
    device: Arc<wgpu::Device>,
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    resize_handle: SurfaceResizeHandle,
}

impl ResizeableSurface<'_> {
    pub fn get_current_texture(&mut self) -> Result<SurfaceTextureWithView, wgpu::SurfaceError> {
        if let Some(new_size) = self.resize_handle.update() {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }

        let texture = self.surface.get_current_texture()?;
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Ok(SurfaceTextureWithView { texture, view })
    }
}

pub struct SurfaceTextureWithView {
    pub texture: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
}

pub struct WgpuInitResult<'window> {
    pub instance: wgpu::Instance,
    pub surface: ResizeableSurface<'window>,
    pub adapter: wgpu::Adapter,
    pub surface_texture_format: wgpu::TextureFormat,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

pub async fn init<'window>(
    surface_target: impl Into<SurfaceTarget<'window>>,
    mut surface_resize_handle: SurfaceResizeHandle,
    trace_path: Option<&std::path::Path>,
) -> anyhow::Result<WgpuInitResult<'window>> {
    let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    });
    let surface = instance
        .create_surface(surface_target)
        .context("Creating surface")?;
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(
        &instance,
        // NOTE: this select the low-power GPU by default
        // it's fine, but if we want to use the high-perf one in the future we will have to change our logic
        Some(&surface),
    )
    .await
    .context("Failed to find appropriate wgpu adapter")?;

    info!("Selected an adapter {:?}", adapter.get_info(),);
    debug!("Adapter limits: {:?}", adapter.limits());

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                // TODO: probably will need to make it configurable with a wgsl fallback at some point
                required_features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                required_limits: wgpu::Limits {
                    // This is required in order to support higher resolutions
                    // TODO: make it configurable for lower-end devices
                    max_texture_dimension_2d: 4096,
                    ..wgpu::Limits::downlevel_webgl2_defaults()
                },
                memory_hints: Default::default(),
            },
            trace_path,
        )
        .await
        .context("Failed to create wgpu device")?;

    // we DON'T want sRGB-correctness, as the original game doesn't have it
    let surface_texture_format = surface
        .get_capabilities(&adapter)
        .formats
        .iter()
        .filter(|f| !f.is_srgb())
        .next()
        .unwrap()
        .clone();

    debug!(
        "Picked {:?} as the surface texture format",
        surface_texture_format
    );

    let SurfaceSize { width, height } = surface_resize_handle.get();

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_texture_format,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let surface = ResizeableSurface {
        device: device.clone(),
        surface,
        surface_config,
        resize_handle: surface_resize_handle,
    };

    Ok(WgpuInitResult {
        instance,
        surface,
        adapter,
        surface_texture_format,
        device,
        queue,
    })
}
