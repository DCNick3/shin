use glam::{Mat4, Vec4};
use shin_render::{
    BindGroupLayouts, Camera, GpuCommonResources, Pipelines, RenderTarget, SpriteVertexBuffer,
};
use shin_video::mp4::Mp4;
use shin_video::{H264Decoder, Mp4BitstreamConverter, YuvTexture};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::info;
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

    let file = File::open("ship1.mp4").unwrap();
    let mut mp4 = Mp4::new(file).unwrap();

    let mut decoder = H264Decoder::new().unwrap();
    let mut conv = mp4
        .video_track
        .get_mp4_track_info(Mp4BitstreamConverter::for_mp4_track);
    let mut buffer = Vec::new();

    for _ in 0..3 {
        let mp4_sample = mp4.video_track.next_sample().unwrap().unwrap();
        conv.convert_packet(&mp4_sample.bytes, &mut buffer);
        decoder.push_packet(&buffer).unwrap();
    }

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
            &buffer,
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

                {
                    if let Some(yuv_frame) = decoder.read_frame().unwrap() {
                        yuv_texture.write_data(&yuv_frame, &resources.queue);
                    } else {
                        info!("EOF");
                        *control_flow = ControlFlow::ExitWithCode(0);
                    }

                    if let Some(mp4_sample) = mp4.video_track.next_sample().unwrap() {
                        info!("video frame {:05}: {}", i, mp4_sample.start_time);

                        conv.convert_packet(&mp4_sample.bytes, &mut buffer);
                        decoder.push_packet(&buffer).unwrap();
                    } else {
                        decoder.mark_eof();
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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1920.0, 1080.0))
        .build(&event_loop)
        .unwrap();
    tracing_subscriber::fmt::init();

    pollster::block_on(run(event_loop, window));
}
