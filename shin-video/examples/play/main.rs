use glam::{Mat4, Vec4};
use shin_render::{
    BindGroupLayouts, Camera, GpuCommonResources, Pipelines, RenderTarget, SpriteVertexBuffer,
};
use shin_video::mp4::Mp4;
use shin_video::{H264Decoder, Mp4BitstreamConverter, YuvTexture};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::{info, trace};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = wgpu::util::initialize_adapter_from_env_or_default(
        &instance,
        backends,
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
                features: wgpu::Features::PUSH_CONSTANTS,
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits {
                    max_push_constant_size: 256,
                    ..wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
                },
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

    // let file = File::open("ship1.mp4").unwrap();
    let file = File::open("op1.mp4").unwrap();
    let mut mp4 = Mp4::new(file).unwrap();

    let mut decoder = H264Decoder::new(mp4.video_track).unwrap();

    let yuv_texture = {
        let frame_info = decoder.info().unwrap();

        YuvTexture::new(&resources, frame_info)
    };

    // it's a hack, I just want to ignore the camera for now
    let vertex_buffer = SpriteVertexBuffer::new(&resources, (-1.0, 1.0, 1.0, -1.0), Vec4::ONE);

    let render_target = RenderTarget::new(
        &resources,
        camera.render_buffer_size(),
        Some("Window RenderTarget"),
    );

    let mut i = 0;

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (
            &instance,
            &adapter,
            &decoder,
            &yuv_texture,
            &resources,
            &yuv_texture,
            &render_target,
            &i,
        );

        *control_flow = ControlFlow::Wait;
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
            Event::RedrawRequested(_) => {
                i += 1;

                if i % 2 == 0 {
                    if let Some(yuv_frame) = decoder.read_frame().unwrap() {
                        yuv_texture.write_data(&yuv_frame, &resources.queue);
                    } else {
                        info!("EOF");
                        *control_flow = ControlFlow::ExitWithCode(0);
                    }
                }

                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = resources.start_encoder();
                {
                    let mut rpass = render_target.begin_raw_render_pass(&mut encoder, None);
                    // let proj = camera.screen_projection_matrix();
                    resources.draw_yuv_sprite(
                        &mut rpass,
                        vertex_buffer.vertex_source(),
                        yuv_texture.bind_group(),
                        Mat4::IDENTITY,
                    );
                }

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    resources.pipelines.sprite_screen.draw(
                        &mut rpass,
                        vertex_buffer.vertex_source(),
                        render_target.bind_group(),
                        Mat4::IDENTITY,
                    );
                }

                drop(encoder);

                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1920.0, 1080.0))
        .build(&event_loop)
        .unwrap();
    tracing_subscriber::fmt::init();
    create_task_pools();

    pollster::block_on(run(event_loop, window));
}
