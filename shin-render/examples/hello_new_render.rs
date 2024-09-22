use glam::{Mat4, Vec3};
use shin_render::new_render::{
    pipelines::{PipelineStorage, DEPTH_STENCIL_FORMAT},
    render_pass::RenderPass,
    resize::SurfaceResizeSource,
    resizeable_texture::ResizeableTexture,
    DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder,
};
use shin_render_shader_types::{
    buffer::{BytesAddress, DynamicBuffer, VertexSource},
    texture::TextureBindGroupLayout,
    vertices::{PosColVertex, UnormColor},
};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1920, 1080))
        .with_maximized(false)
        .with_position(LogicalPosition::new(1080, 0))
        .build(&event_loop)
        .unwrap();

    let window_resize_source = SurfaceResizeSource::new(window.inner_size().into());

    let mut wgpu =
        shin_render::new_render::init::init(&window, window_resize_source.handle(), None)
            .await
            .expect("wgpu init failed");

    let mut dynamic_buffer = DynamicBuffer::new(
        &wgpu.device,
        wgpu.queue.clone(),
        BytesAddress::new(1024 * 1024),
    );
    let texture_bind_group_layout = TextureBindGroupLayout::new(&wgpu.device);

    let mut pipelines = PipelineStorage::new(
        wgpu.device.clone(),
        wgpu.surface_texture_format,
        &texture_bind_group_layout,
    );

    // prevent a move
    let window = &window;

    let mut depth_stencil = ResizeableTexture::new(
        wgpu.device.clone(),
        DEPTH_STENCIL_FORMAT,
        window_resize_source.handle(),
    );

    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                &WindowEvent::Resized(physical_size) => {
                    window_resize_source.resize(physical_size.into());
                }
                WindowEvent::RedrawRequested => {
                    let mut encoder = wgpu
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    let surface_texture = wgpu.surface.get_current_texture().unwrap();

                    let vertices = [
                        PosColVertex {
                            position: Vec3::new(0.0, 0.5, 0.0),
                            color: UnormColor::RED,
                        },
                        PosColVertex {
                            position: Vec3::new(-0.5, -0.5, 0.0),
                            color: UnormColor::GREEN,
                        },
                        PosColVertex {
                            position: Vec3::new(0.5, -0.5, 0.0),
                            color: UnormColor::BLUE,
                        },
                    ];
                    let vertices = dynamic_buffer.get_vertex_with_data(&vertices);

                    let mut pass = RenderPass::new(
                        &mut pipelines,
                        &mut dynamic_buffer,
                        &wgpu.device,
                        &mut encoder,
                        &surface_texture.view,
                        &depth_stencil.get_view(),
                    );
                    pass.run(RenderRequestBuilder::new().build(
                        RenderProgramWithArguments::Fill {
                            vertices: VertexSource::VertexBuffer {
                                vertex_buffer: vertices.as_buffer_ref(),
                            },
                            transform: Mat4::IDENTITY,
                        },
                        DrawPrimitive::Triangles,
                    ));

                    drop(pass);

                    wgpu.queue.submit(std::iter::once(encoder.finish()));

                    surface_texture.texture.present();

                    window.request_redraw();
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}

fn main() {
    tracing_subscriber::fmt::init();
    pollster::block_on(run());
}
