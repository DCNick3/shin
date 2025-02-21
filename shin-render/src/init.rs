use anyhow::Context;
use shin_render_shader_types::{buffer::BytesAddress, texture::TextureSamplerStore};
use tracing::{debug, info};
use wgpu::{InstanceFlags, SurfaceTarget};

use crate::{
    DEPTH_STENCIL_FORMAT,
    depth_stencil::DepthStencil,
    dynamic_buffer::DynamicBuffer,
    pipelines::PipelineStorage,
    resize::{CanvasSize, ResizeHandle, SurfaceSize},
    resizeable_texture::ResizeableTexture,
};

#[derive(Debug)]
pub struct ResizeableSurface<'window> {
    device: wgpu::Device,
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

        let texture = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                debug!("Surface error, recreating surface");
                self.surface.configure(&self.device, &self.surface_config);
                return self.get_current_texture();
            }
            Err(e) => return Err(e),
        };
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
    device: wgpu::Device,
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
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuInitResult<'static> {
    pub fn into_resources(
        self,
        surface_resize_handle: ResizeHandle<SurfaceSize>,
        canvas_resize_handle: ResizeHandle<CanvasSize>,
    ) -> (WgpuResources, RenderResources) {
        let dynamic_buffer = DynamicBuffer::new(
            self.device.clone(),
            self.queue.clone(),
            BytesAddress::new(1024 * 1024),
        );

        let pipelines = PipelineStorage::new(self.device.clone(), self.surface_texture_format);

        let surface_depth_stencil_buffer = ResizeableTexture::new(
            self.device.clone(),
            Some("Surface DepthStencil".to_string()),
            DEPTH_STENCIL_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            surface_resize_handle,
        );
        let canvas_depth_stencil_buffer = DepthStencil::new(
            self.device.clone(),
            canvas_resize_handle,
            Some("Canvas DepthStencil".to_string()),
        );
        let sampler_store = TextureSamplerStore::new(&self.device);

        (
            WgpuResources {
                instance: self.instance,
                adapter: self.adapter,
                device: self.device,
                queue: self.queue,
            },
            RenderResources {
                surface: self.surface,
                surface_depth_stencil_buffer,
                canvas_depth_stencil_buffer,
                sampler_store,
                dynamic_buffer,
                pipelines,
                surface_texture_format: self.surface_texture_format,
            },
        )
    }
}

pub async fn init_wgpu<'window>(
    surface_target: impl Into<SurfaceTarget<'window>>,
    surface_resize_handle: ResizeHandle<SurfaceSize>,
    trace_path: Option<&std::path::Path>,
) -> anyhow::Result<WgpuInitResult<'window>> {
    info!("Initializing wgpu...");

    let backends = wgpu::Backends::from_env().unwrap_or(wgpu::Backends::all());
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends,
        flags: InstanceFlags::debugging(),
        ..Default::default()
    });
    let surface = instance
        .create_surface(surface_target)
        .context("Creating surface")?;

    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    for adapter in adapters {
        let info = adapter.get_info();
        info!("Found adapter: {:?}", info);
    }

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
                // this enables SPIRV_SHADER_PASSTHROUGH if available
                required_features: adapter.features() & wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
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
    let surface_texture_format = *surface
        .get_capabilities(&adapter)
        .formats
        .iter()
        .find(|f| !f.is_srgb())
        .unwrap();

    debug!(
        "Picked {:?} as the surface texture format",
        surface_texture_format
    );

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
    device: wgpu::Device,
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

pub struct WgpuResources {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct RenderResources {
    // render-related resources
    pub surface: ResizeableSurface<'static>,
    // keeping two depth stencil buffers because surface and canvas can potentially be of different sizes
    // needed to initiate render passes
    // might want to move it to a separate struct
    pub surface_depth_stencil_buffer: ResizeableTexture<SurfaceSize>,
    pub canvas_depth_stencil_buffer: DepthStencil,
    pub sampler_store: TextureSamplerStore,
    pub dynamic_buffer: DynamicBuffer,
    pub pipelines: PipelineStorage,

    // render parameters or idk
    pub surface_texture_format: wgpu::TextureFormat,
}
