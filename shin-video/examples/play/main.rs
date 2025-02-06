use std::{
    fs::File,
    path::Path,
    sync::{Arc, RwLock},
    time::Instant,
};

use glam::Mat4;
use shin_audio::AudioManager;
use shin_core::time::Ticks;
use shin_render::{
    BindGroupLayouts, Camera, GpuCommonResources, Pipelines, RenderTarget, Renderable,
};
use shin_video::{mp4::Mp4, VideoPlayer};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    });

    let surface = instance.create_surface(&window).unwrap();

    let adapter = wgpu::util::initialize_adapter_from_env_or_default(
        &instance,
        // NOTE: this select the low-power GPU by default
        // it's fine, but if we want to use the high-perf one in the future we will have to ditch this function
        Some(&surface),
    )
    .await
    .unwrap();

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::PUSH_CONSTANTS,
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits {
                    max_push_constant_size: 256,
                    ..wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
                },
                memory_hints: Default::default(),
            },
            Some(Path::new("wgpu_trace")),
        )
        .await
        .expect("Failed to create device");

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    let bind_group_layouts = BindGroupLayouts::new(&device);
    let pipelines = Pipelines::new(&device, &bind_group_layouts, swapchain_format);

    let window_size = (window.inner_size().width, window.inner_size().height);
    let mut camera = Camera::new(window_size);

    let resources = Arc::new(GpuCommonResources {
        device,
        queue,
        render_buffer_size: RwLock::new(camera.render_buffer_size()),
        bind_group_layouts,
        pipelines,
    });

    let audio_manager = AudioManager::new();

    // let file = File::open("ship1.mp4").unwrap();
    let file = File::open("op1.mp4").unwrap();
    let mp4 = Mp4::new(file).unwrap();
    let mut video_player = VideoPlayer::new(&resources, &audio_manager, mp4).unwrap();

    let render_target = RenderTarget::new(
        &resources,
        camera.render_buffer_size(),
        Some("Window RenderTarget"),
    );

    let mut time = Instant::now();

    // don't move it pls
    let window = &window;

    event_loop
        .run(move |event, target| {
            // Have the closure take ownership of the resources.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.
            let _ = (
                &instance,
                &adapter,
                &video_player,
                &resources,
                &render_target,
                &time,
                &audio_manager,
            );

            target.set_control_flow(ControlFlow::Wait);
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    // Reconfigure the surface with the new size
                    config.width = size.width;
                    config.height = size.height;
                    camera.resize((size.width, size.height));
                    surface.configure(&resources.device, &config);
                    // On macos the window needs to be redrawn manually after resizing
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let time_now = Instant::now();
                    let delta_time = time_now - time;
                    time = time_now;

                    if video_player.is_finished() {
                        target.exit();
                    }

                    video_player.update(Ticks::from_duration(delta_time), &resources.queue);

                    let frame = surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = resources.start_encoder();
                    {
                        let mut rpass = render_target.begin_raw_render_pass(&mut encoder, None);
                        let proj = render_target.projection_matrix();

                        video_player.render(&resources, &mut rpass, Mat4::IDENTITY, proj);
                    }

                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        let proj = camera.screen_projection_matrix();

                        resources.pipelines.sprite_screen.draw(
                            &mut rpass,
                            render_target.vertex_source(),
                            render_target.bind_group(),
                            proj,
                        );
                    }

                    drop(encoder);

                    frame.present();
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => target.exit(),
                _ => {}
            }
        })
        .unwrap();
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1920.0, 1080.0))
        .build(&event_loop)
        .unwrap();
    tracing_subscriber::fmt::init();
    shin_tasks::create_task_pools();

    pollster::block_on(run(event_loop, window));
}
