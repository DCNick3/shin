use std::sync::Arc;

use glam::{Mat4, Vec3};
use shin_render::new_render::{
    init::{RenderResources, WgpuInitResult},
    render_pass::RenderPass,
    resize::SurfaceResizeSource,
    DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder,
};
use shin_render_shader_types::{
    buffer::VertexSource,
    vertices::{PosColVertex, UnormColor},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

#[derive(Debug)]
enum MyUserEvent {
    WgpuInitDone(anyhow::Result<WgpuInitResult<'static>>),
}

#[derive(Default)]
enum State {
    WaitingForInitialResume {
        // unfortunately we have to weave in the proxy from outside of the loop until winit 0.31: https://github.com/rust-windowing/winit/pull/3764
        proxy: EventLoopProxy<MyUserEvent>,
    },
    WaitingForWgpuInit {
        proxy: EventLoopProxy<MyUserEvent>,
        winit: WindowState,
        task: shin_tasks::Task<()>,
    },
    Operational {
        proxy: EventLoopProxy<MyUserEvent>,
        winit: WindowState,
        render: RenderResources,
    },
    #[default]
    Poison,
}

impl State {
    pub fn new(proxy: EventLoopProxy<MyUserEvent>) -> Self {
        Self::WaitingForInitialResume { proxy }
    }

    pub fn winit(&self) -> Option<&WindowState> {
        match self {
            State::WaitingForInitialResume { .. } => None,
            State::WaitingForWgpuInit { winit, .. } => Some(winit),
            State::Operational { winit, .. } => Some(winit),
            State::Poison => {
                unreachable!()
            }
        }
    }
}

struct WindowState {
    pub window: Arc<Window>,
    pub resize_source: SurfaceResizeSource,
}

impl WindowState {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(1920, 1080))
            .with_maximized(false)
            .with_position(LogicalPosition::new(1080, 0));
        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let window_resize_source = SurfaceResizeSource::new(window.inner_size().into());

        Self {
            window,
            resize_source: window_resize_source,
        }
    }
}

impl ApplicationHandler<MyUserEvent> for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match std::mem::take(self) {
            State::WaitingForInitialResume { proxy } => {
                let winit = WindowState::new(event_loop);

                let (window_copy, resize_handle, proxy_copy) = (
                    winit.window.clone(),
                    winit.resize_source.handle(),
                    proxy.clone(),
                );

                let task = shin_tasks::IoTaskPool::get().spawn(async move {
                    let result =
                        shin_render::new_render::init::init_wgpu(window_copy, resize_handle, None)
                            .await;

                    proxy_copy
                        .send_event(MyUserEvent::WgpuInitDone(result))
                        .unwrap();
                });

                *self = State::WaitingForWgpuInit { proxy, winit, task };
            }
            State::WaitingForWgpuInit { .. } => {
                // this shouldn't happen.. I think
                // TODO: figure out this out better when porting to Android or smth
                todo!()
            }
            State::Operational {
                proxy,
                winit,
                mut render,
            } => {
                render.surface = shin_render::new_render::init::surface_reinit(
                    &render.instance,
                    render.device.clone(),
                    winit.window.clone(),
                    winit.resize_source.handle(),
                    render.surface_texture_format,
                )
                .expect("surface reinit failed");

                *self = State::Operational {
                    proxy,
                    winit,
                    render,
                };
            }
            State::Poison => unreachable!(),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: MyUserEvent) {
        match event {
            MyUserEvent::WgpuInitDone(result) => {
                let wgpu = result.expect("wgpu init failed");

                let State::WaitingForWgpuInit {
                    proxy,
                    winit,
                    task: _,
                } = std::mem::take(self)
                else {
                    unreachable!()
                };

                // finish render init
                let render = RenderResources::new(wgpu, winit.resize_source.handle());

                *self = State::Operational {
                    proxy,
                    winit,
                    render,
                };
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(winit) = self.winit() else {
            return;
        };

        if window_id != winit.window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                winit.resize_source.resize(physical_size.into());
                winit.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let State::Operational {
                    proxy: _,
                    winit,
                    render,
                } = self
                else {
                    return;
                };

                let mut encoder = render
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let surface_texture = render.surface.get_current_texture().unwrap();

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
                let vertices = render.dynamic_buffer.get_vertex_with_data(&vertices);

                let mut pass = RenderPass::new(
                    &mut render.pipelines,
                    &mut render.dynamic_buffer,
                    &render.device,
                    &mut encoder,
                    &surface_texture.view,
                    &render.depth_stencil_buffer.get_view(),
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

                render.queue.submit(std::iter::once(encoder.finish()));

                winit.window.pre_present_notify();
                surface_texture.texture.present();

                winit.window.request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(winit) = self.winit() else {
            return;
        };

        winit.window.request_redraw();
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    shin_tasks::create_task_pools();

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    event_loop.run_app(&mut State::new(proxy)).unwrap();
}
