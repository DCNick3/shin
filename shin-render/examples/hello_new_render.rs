use std::sync::Arc;

use glam::{Mat4, Vec3};
use shin_render::new_render::{
    init::{RenderResources, WgpuInitResult},
    render_pass::RenderPass,
    resize::{SurfaceResizeHandle, SurfaceResizeSource},
    DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder,
};
use shin_render_shader_types::{
    buffer::VertexSource,
    vertices::{PosColVertex, UnormColor},
};
use shin_tasks::Task;
use tracing::info;
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
    /// On web platform the initial winit canvas size is zero, so we have to wait for a non-zero size
    WaitingForNonzeroSize {
        proxy: EventLoopProxy<MyUserEvent>,
        winit: WindowState,
        resize_handle: SurfaceResizeHandle,
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
            State::WaitingForNonzeroSize { winit, .. } => Some(winit),
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
        let mut attributes = Window::default_attributes();

        #[cfg(not(target_arch = "wasm32"))]
        {
            attributes = attributes
                .with_inner_size(LogicalSize::new(1920, 1080))
                .with_maximized(false)
                .with_position(LogicalPosition::new(1080, 0));
        }

        #[cfg(target_arch = "wasm32")]
        {
            use anyhow::Context;
            use web_sys::wasm_bindgen::JsCast;
            use winit::platform::web::{WindowAttributesExtWebSys, WindowExtWebSys};
            attributes = web_sys::window()
                .context("Couldn't get window")
                .and_then(|win| win.document().context("Couldn't get document"))
                .and_then(|doc| {
                    let canvas = doc
                        .get_element_by_id("canvas")
                        .context("Couldn't find the canvas element element")?
                        .dyn_into::<web_sys::HtmlCanvasElement>()
                        .map_err(|e| {
                            anyhow::anyhow!(
                                "Couldn't cast the element to a canvas element: {:?}",
                                e
                            )
                        })
                        .context("Couldn't cast the element to a canvas element")?;

                    Ok(attributes.with_canvas(Some(canvas)))
                })
                .expect("Couldn't attach canvas to winit window");
        }

        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let size = window.inner_size().into();

        info!("Created a window with size: {:?}", size);

        let window_resize_source = SurfaceResizeSource::new(size);

        Self {
            window,
            resize_source: window_resize_source,
        }
    }
}

fn start_wgpu_init(
    window: Arc<Window>,
    resize_handle: SurfaceResizeHandle,
    proxy: EventLoopProxy<MyUserEvent>,
) -> Task<()> {
    shin_tasks::IoTaskPool::get().spawn(async move {
        let result = shin_render::new_render::init::init_wgpu(window, resize_handle, None).await;

        proxy.send_event(MyUserEvent::WgpuInitDone(result)).unwrap();
    })
}

impl ApplicationHandler<MyUserEvent> for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match std::mem::take(self) {
            State::WaitingForInitialResume { proxy } => {
                let winit = WindowState::new(event_loop);

                let mut resize_handle = winit.resize_source.handle();
                let current_size = resize_handle.get();

                if !current_size.is_empty() {
                    let task = start_wgpu_init(winit.window.clone(), resize_handle, proxy.clone());

                    *self = State::WaitingForWgpuInit { proxy, winit, task };
                } else {
                    *self = State::WaitingForNonzeroSize {
                        proxy,
                        winit,
                        resize_handle,
                    };
                }
            }
            State::WaitingForNonzeroSize { .. } => {
                // this probably shouldn't happen
                todo!()
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

                if let State::WaitingForNonzeroSize {
                    proxy,
                    mut resize_handle,
                    winit,
                } = std::mem::take(self)
                {
                    if !resize_handle.get().is_empty() {
                        let task = start_wgpu_init(
                            winit.window.clone(),
                            resize_handle.clone(),
                            proxy.clone(),
                        );

                        *self = State::WaitingForWgpuInit { proxy, winit, task };
                    }
                }
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
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    #[cfg(target_arch = "wasm32")]
    tracing_wasm::set_as_global_default();

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    shin_tasks::create_task_pools();

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    event_loop.run_app(&mut State::new(proxy)).unwrap();
}
