use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, trace, warn};

use shin_core::format::scenario::instructions::CodeAddress;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::window::Fullscreen;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::asset::LayeredAssetIo;
use crate::audio::AudioManager;
use crate::{
    adv::assets::AdvAssets,
    adv::Adv,
    asset::AnyAssetServer,
    fps_counter::FpsCounter,
    input::RawInputState,
    render::overlay::{OverlayManager, OverlayVisitable},
    render::BindGroupLayouts,
    render::Camera,
    render::GpuCommonResources,
    render::Pillarbox,
    render::Pipelines,
    render::{RenderTarget, Renderable, SpriteVertexBuffer},
    update::{Updatable, UpdateContext},
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

struct State {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    window_size: (u32, u32),
    resources: Arc<GpuCommonResources>,
    camera: Camera,
    // TODO: do we want to pull the bevy deps?
    time: bevy_time::Time,
    screen_vertices: SpriteVertexBuffer,
    render_target: RenderTarget,
    pillarbox: Pillarbox,
    asset_server: Arc<AnyAssetServer>,
    input: RawInputState,
    overlay_manager: OverlayManager,
    fps_counter: FpsCounter,
    adv: Adv,
}

impl State {
    async fn new(window: &Window) -> Self {
        let window_size = window.inner_size();
        let window_size = (window_size.width, window_size.height);

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let backend_bits = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
        let instance = wgpu::Instance::new(backend_bits);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(
            &instance,
            backend_bits,
            // NOTE: this select the low-power GPU by default
            // it's fine, but if we want to use the high-perf one in the future we will have to ditch this function
            Some(&surface),
        )
        .await
        .unwrap();

        info!("Selected an adapter {:?}", adapter.get_info(),);
        debug!("Adapter limits: {:?}", adapter.limits());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::PUSH_CONSTANTS,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: wgpu::Limits {
                        max_texture_dimension_2d: 4096,
                        // TODO: maybe we should use uniform buffers more...
                        max_push_constant_size: 128,

                        ..wgpu::Limits::downlevel_webgl2_defaults()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        // TODO: make a better selection?
        // TODO: rn we don't really support switching this
        // it may be worth to add one more pass to convert from internal (Rgba8) to the preferred output format
        // or support having everything in the preferred format? (sounds hard)
        let surface_texture_format = surface.get_supported_formats(&adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_texture_format,
            width: window_size.0,
            height: window_size.1,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let bind_group_layouts = BindGroupLayouts::new(&device);
        let pipelines = Pipelines::new(&device, &bind_group_layouts, surface_texture_format);

        let camera = Camera::new(window_size);

        let resources = Arc::new(GpuCommonResources {
            device,
            queue,
            render_buffer_size: RwLock::new(camera.render_buffer_size()),
            bind_group_layouts,
            pipelines,
        });

        let overlay = OverlayManager::new(&resources, surface_texture_format);

        let screen_vertices = SpriteVertexBuffer::new_fullscreen(&resources);
        let render_target = RenderTarget::new(
            &resources,
            camera.render_buffer_size(),
            Some("Window RenderTarget"),
        );

        let pillarbox = Pillarbox::new(&resources);

        let audio_manager = Arc::new(AudioManager::new());

        let assets_directory = PathBuf::from("assets");

        let mut asset_io = LayeredAssetIo::new();

        if let Err(e) = asset_io.try_with_dir(assets_directory.join("data")) {
            warn!("Failed to load data directory: {}", e);
        }
        if let Err(e) = asset_io.try_with_rom(assets_directory.join("data.rom")) {
            warn!("Failed to load rom file: {}", e);
        }

        if asset_io.is_empty() {
            panic!("No assets configured, have you copied your game files?");
        }

        debug!("Asset IO: {:#?}", asset_io);

        let asset_server = Arc::new(AnyAssetServer::new(asset_io.into()));

        let adv_assets =
            pollster::block_on(AdvAssets::load(&asset_server)).expect("Loading assets failed");

        let mut adv = Adv::new(&resources, audio_manager, adv_assets, 0, 42);

        adv.fast_forward_to(CodeAddress(0xb03f5));

        Self {
            surface,
            surface_config: config,
            window_size,
            resources,
            camera,
            time: bevy_time::Time::default(),
            screen_vertices,
            render_target,
            pillarbox,
            asset_server,
            input: RawInputState::new(),
            overlay_manager: overlay,
            fps_counter: FpsCounter::new(),
            adv,
        }
    }

    fn reconfigure_surface(&mut self) {
        self.surface
            .configure(&self.resources.device, &self.surface_config);
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.window_size = new_size;
            self.surface_config.width = new_size.0;
            self.surface_config.height = new_size.1;
            self.surface
                .configure(&self.resources.device, &self.surface_config);

            self.camera.resize(new_size);
            self.render_target
                .resize(&self.resources, self.camera.render_buffer_size());

            debug!(
                "Window resized to {:?}, new render buffer size is {:?}",
                new_size,
                self.camera.render_buffer_size()
            );

            *self.resources.render_buffer_size.write().unwrap() = self.camera.render_buffer_size();

            self.pillarbox.resize(&self.resources);
            self.adv.resize(&self.resources);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        self.input.on_winit_event(event);
        false
    }

    fn update(&mut self) {
        self.time.update();

        let mut input = self.input.clone();

        self.overlay_manager
            .start_update(&self.time, &input, self.window_size);
        self.overlay_manager.visit_overlays(|collector| {
            self.fps_counter.visit_overlay(collector);
            input.visit_overlay(collector);
            self.adv.visit_overlay(collector);
        });
        self.overlay_manager
            .finish_update(&self.resources, &mut input);

        let update_context = UpdateContext {
            time: &self.time,
            gpu_resources: &self.resources,
            asset_server: &self.asset_server,
            raw_input_state: &input,
        };

        self.adv.update(&update_context);
        self.fps_counter.update(&update_context);

        // NOTE: it's important that the input is updated after everything else, as it clears some state after it should have been handled
        self.input.update();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // render everything to the render target
        {
            let mut encoder = self.resources.start_encoder();
            let mut render_pass = self
                .render_target
                .begin_render_pass(&mut encoder, Some("Screen RenderPass"));

            self.adv.render(
                &self.resources,
                &mut render_pass,
                self.camera.projection_matrix(),
            );
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut encoder = self.resources.start_encoder();
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Final RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.resources.pipelines.sprite_screen.draw(
                &mut render_pass,
                self.screen_vertices.vertex_source(),
                self.render_target.bind_group(),
                self.camera.screen_projection_matrix(),
            );
            self.pillarbox.render(
                &self.resources,
                &mut render_pass,
                self.camera.screen_projection_matrix(),
            );

            self.overlay_manager
                .render(&self.resources, &mut render_pass);
        }

        output.present();

        Ok(())
    }
}

fn create_task_pools() {
    // bevy params:
    // TaskPoolOptions {
    //     // By default, use however many cores are available on the system
    //     min_total_threads: 1,
    //     max_total_threads: std::usize::MAX,
    //
    //     // Use 25% of cores for IO, at least 1, no more than 4
    //     io: TaskPoolThreadAssignmentPolicy {
    //         min_threads: 1,
    //         max_threads: 4,
    //         percent: 0.25,
    //     },
    //
    //     // Use 25% of cores for async compute, at least 1, no more than 4
    //     async_compute: TaskPoolThreadAssignmentPolicy {
    //         min_threads: 1,
    //         max_threads: 4,
    //         percent: 0.25,
    //     },
    //
    //     // Use all remaining cores for compute (at least 1)
    //     compute: TaskPoolThreadAssignmentPolicy {
    //         min_threads: 1,
    //         max_threads: std::usize::MAX,
    //         percent: 1.0, // This 1.0 here means "whatever is left over"
    //     },
    // }

    let total_threads = bevy_tasks::available_parallelism().clamp(1, usize::MAX);
    trace!("Assigning {} cores to default task pools", total_threads);

    let mut remaining_threads = total_threads;

    fn get_number_of_threads(
        percent: f32,
        min_threads: usize,
        max_threads: usize,
        remaining_threads: usize,
        total_threads: usize,
    ) -> usize {
        let mut desired = (total_threads as f32 * percent).round() as usize;

        // Limit ourselves to the number of cores available
        desired = desired.min(remaining_threads);

        // Clamp by min_threads, max_threads. (This may result in us using more threads than are
        // available, this is intended. An example case where this might happen is a device with
        // <= 2 threads.
        desired.clamp(min_threads, max_threads)
    }

    {
        // Determine the number of IO threads we will use
        let io_threads = get_number_of_threads(0.25, 1, 4, remaining_threads, total_threads);

        trace!("IO Threads: {}", io_threads);
        remaining_threads = remaining_threads.saturating_sub(io_threads);

        bevy_tasks::IoTaskPool::init(|| {
            bevy_tasks::TaskPoolBuilder::default()
                .num_threads(io_threads)
                .thread_name("IO Task Pool".to_string())
                .build()
        });
    }

    {
        // Use the rest for async compute threads
        let async_compute_threads = remaining_threads;
        // get_number_of_threads(0.25, 1, 4, remaining_threads, total_threads);

        trace!("Async Compute Threads: {}", async_compute_threads);
        remaining_threads = remaining_threads.saturating_sub(async_compute_threads);

        bevy_tasks::AsyncComputeTaskPool::init(|| {
            bevy_tasks::TaskPoolBuilder::default()
                .num_threads(async_compute_threads)
                .thread_name("Async Compute Task Pool".to_string())
                .build()
        });
    }

    // do not initialize the compute task pool, we do not use it (at least for now)
    trace!("Remaining Threads: {}", remaining_threads);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            tracing_subscriber::fmt::init();
        }
    }

    create_task_pools();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1920, 1080))
        .with_maximized(false)
        .with_position(LogicalPosition::new(1080, 0))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    // UPDATED!
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode:
                                        Some(VirtualKeyCode::Escape | VirtualKeyCode::Q),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::F11),
                                    ..
                                },
                            ..
                        } => {
                            window.set_fullscreen(
                                window
                                    .fullscreen()
                                    .map_or_else(|| Some(Fullscreen::Borderless(None)), |_| None),
                            );
                        }
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::F10),
                                    ..
                                },
                            ..
                        } => window.set_inner_size(PhysicalSize::new(1920, 1080)),
                        WindowEvent::Resized(physical_size) => {
                            state.resize((*physical_size).into());
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            state.resize((**new_inner_size).into());
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.reconfigure_surface();
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
