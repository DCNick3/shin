use std::sync::Arc;

use anyhow::Context;
use shin_render_shader_types::buffer::BytesAddress;
use tracing::{debug, info};
use wgpu::SurfaceTarget;

use crate::{
    dynamic_buffer::DynamicBuffer,
    pipelines::{PipelineStorage, DEPTH_STENCIL_FORMAT},
    resize::{CanvasSize, ResizeHandle, SurfaceSize},
    resizeable_texture::ResizeableTexture,
};

#[derive(Debug)]
pub struct ResizeableSurface<'window> {
    device: Arc<wgpu::Device>,
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    resize_handle: ResizeHandle<SurfaceSize>,
}

impl ResizeableSurface<'_> {
    pub fn get_current_texture(
        &mut self,
    ) -> Result<((f32, f32, f32, f32), SurfaceTextureWithView), wgpu::SurfaceError> {
        if let Some(new_size) = self.resize_handle.update() {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }

        let viewport = self.resize_handle.get_viewport();

        let texture = self.surface.get_current_texture()?;
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Ok((viewport, SurfaceTextureWithView { texture, view }))
    }
}

pub struct SurfaceTextureWithView {
    pub texture: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
}

fn configure_surface(
    device: Arc<wgpu::Device>,
    surface: wgpu::Surface,
    mut surface_resize_handle: ResizeHandle<SurfaceSize>,
    surface_texture_format: wgpu::TextureFormat,
) -> ResizeableSurface {
    let SurfaceSize { width, height } = surface_resize_handle.get();

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_texture_format,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        desired_maximum_frame_latency: 2,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);

    ResizeableSurface {
        device: device.clone(),
        surface,
        surface_config,
        resize_handle: surface_resize_handle,
    }
}

#[derive(Debug)]
pub struct WgpuInitResult<'window> {
    pub instance: wgpu::Instance,
    pub surface: ResizeableSurface<'window>,
    pub adapter: wgpu::Adapter,
    pub surface_texture_format: wgpu::TextureFormat,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

pub async fn init_wgpu<'window>(
    surface_target: impl Into<SurfaceTarget<'window>>,
    surface_resize_handle: ResizeHandle<SurfaceSize>,
    trace_path: Option<&std::path::Path>,
) -> anyhow::Result<WgpuInitResult<'window>> {
    info!("Initializing wgpu...");

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
                required_features: if cfg!(not(target_arch = "wasm32")) {
                    wgpu::Features::SPIRV_SHADER_PASSTHROUGH
                } else {
                    wgpu::Features::empty()
                },
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
        .map_err(|e| anyhow::Error::msg(format!("Failed to create wgpu device: {:?}", e)))
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

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let surface = configure_surface(
        device.clone(),
        surface,
        surface_resize_handle,
        surface_texture_format,
    );

    Ok(WgpuInitResult {
        instance,
        surface,
        adapter,
        surface_texture_format,
        device,
        queue,
    })
}

/// Re-create a surface with the same parameters as an old one. This function is designed to be used on platforms that have application suspension/resume events, like iOS, Android and web.
pub fn surface_reinit<'window>(
    instance: &wgpu::Instance,
    device: Arc<wgpu::Device>,
    surface_target: impl Into<SurfaceTarget<'window>>,
    surface_resize_handle: ResizeHandle<SurfaceSize>,
    surface_texture_format: wgpu::TextureFormat,
) -> anyhow::Result<ResizeableSurface<'window>> {
    info!("Re-creating surface...");
    let surface = instance
        .create_surface(surface_target)
        .context("Creating surface")?;

    Ok(configure_surface(
        device,
        surface,
        surface_resize_handle,
        surface_texture_format,
    ))
}

pub struct RenderResources {
    // the wgpu stuff
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,

    // render-related resources
    pub surface: ResizeableSurface<'static>,
    // keeping two depth stencil buffers because surface and canvas can potentially be of different sizes
    pub surface_depth_stencil_buffer: ResizeableTexture<SurfaceSize>,
    pub canvas_depth_stencil_buffer: ResizeableTexture<CanvasSize>,
    pub dynamic_buffer: DynamicBuffer,
    pub pipelines: PipelineStorage,

    // render parameters or idk
    pub surface_texture_format: wgpu::TextureFormat,
}

impl RenderResources {
    pub fn new(
        wgpu: WgpuInitResult<'static>,
        surface_resize_handle: ResizeHandle<SurfaceSize>,
        canvas_resize_handle: ResizeHandle<CanvasSize>,
    ) -> Self {
        let dynamic_buffer = DynamicBuffer::new(
            wgpu.device.clone(),
            wgpu.queue.clone(),
            BytesAddress::new(1024 * 1024),
        );

        let pipelines = PipelineStorage::new(wgpu.device.clone(), wgpu.surface_texture_format);

        let surface_depth_stencil_buffer = ResizeableTexture::new(
            wgpu.device.clone(),
            DEPTH_STENCIL_FORMAT,
            surface_resize_handle,
        );
        let canvas_depth_stencil_buffer = ResizeableTexture::new(
            wgpu.device.clone(),
            DEPTH_STENCIL_FORMAT,
            canvas_resize_handle,
        );

        Self {
            instance: wgpu.instance,
            adapter: wgpu.adapter,
            device: wgpu.device,
            queue: wgpu.queue,
            surface: wgpu.surface,
            surface_depth_stencil_buffer,
            canvas_depth_stencil_buffer,
            dynamic_buffer,
            pipelines,
            surface_texture_format: wgpu.surface_texture_format,
        }
    }
}
