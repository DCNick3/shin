use std::{fmt::Debug, sync::Arc};

use derive_where::derive_where;
use shin_input::RawInputState;
use shin_render::{
    init::{RenderResources, WgpuInitResult},
    render_pass::RenderPass,
    resize::{CanvasSize, ResizeHandle, SurfaceResizeSource, SurfaceSize, ViewportParams},
};
use shin_tasks::Task;
use tracing::{info, warn};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopClosed, EventLoopProxy},
    window::{Window, WindowId},
};

pub struct AppContext<'a, A: ShinApp> {
    pub event_loop_proxy: &'a ShinEventLoopProxy<A::EventType>,
    pub input: &'a RawInputState,
    pub winit: &'a mut WindowState,
    pub render: &'a mut RenderResources,
}

pub trait ShinApp: Sized {
    type Parameters;
    type EventType: Send + Debug + 'static;

    fn init(context: AppContext<Self>, parameters: Self::Parameters) -> Self;

    fn map_canvas_size(window_size: PhysicalSize<u32>) -> ViewportParams {
        ViewportParams::both(window_size)
    }

    fn custom_event(&mut self, context: AppContext<Self>, event: Self::EventType);

    fn update(&mut self, context: AppContext<Self>);
    // can't pass context here because `RenderPass` borrows a bunch of stuff from there
    // let's hope it won't be an issue ;)
    fn render(&mut self, pass: &mut RenderPass);
}

#[derive_where(Clone)]
pub struct ShinEventLoopProxy<E: Send + Debug + 'static> {
    proxy: EventLoopProxy<ShinAppEventImpl<E>>,
}

impl<E: Send + Debug + 'static> ShinEventLoopProxy<E> {
    pub fn send_event(&self, event: E) -> Result<(), EventLoopClosed<E>> {
        self.proxy
            .send_event(ShinAppEventImpl::Custom(event))
            .map_err(|e| {
                let EventLoopClosed(ShinAppEventImpl::Custom(e)) = e else {
                    unreachable!()
                };
                EventLoopClosed(e)
            })
    }
}

pub struct WindowState {
    pub window: Arc<Window>,
    pub resize_source: SurfaceResizeSource,
}

impl WindowState {
    pub fn new<A: ShinApp>(event_loop: &ActiveEventLoop) -> Self {
        #[allow(unused_mut)]
        let mut attributes = Window::default_attributes();

        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        {
            use winit::dpi::{LogicalPosition, LogicalSize};
            attributes = attributes
                .with_inner_size(LogicalSize::new(1920, 1080))
                .with_maximized(false)
                .with_position(LogicalPosition::new(1080, 0));
        }

        #[cfg(target_arch = "wasm32")]
        {
            use anyhow::Context;
            use web_sys::wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
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

        let size = A::map_canvas_size(window.inner_size());

        info!("Created a window with size: {:?}", size);

        let window_resize_source = SurfaceResizeSource::new(size);

        Self {
            window,
            resize_source: window_resize_source,
        }
    }
}

#[derive(Debug)]
enum ShinAppEventImpl<E> {
    WgpuInitDone(anyhow::Result<WgpuInitResult<'static>>),
    Custom(E),
}

fn start_wgpu_init<A: ShinApp>(
    window: Arc<Window>,
    surface_resize_handle: ResizeHandle<SurfaceSize>,
    proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
) -> Task<()> {
    shin_tasks::IoTaskPool::get().spawn(async move {
        let result = shin_render::init::init_wgpu(window, surface_resize_handle, None).await;

        proxy
            .send_event(ShinAppEventImpl::WgpuInitDone(result))
            .unwrap();
    })
}

#[derive(Default)]
enum WinitAppState<A: ShinApp> {
    WaitingForInitialResume {
        // unfortunately we have to weave in the proxy from outside of the loop until winit 0.31: https://github.com/rust-windowing/winit/pull/3764
        proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
        input_state: RawInputState,
        params: A::Parameters,
    },
    /// On web platform the initial winit canvas size is zero, so we have to wait for a non-zero size
    WaitingForNonzeroSize {
        proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
        input_state: RawInputState,
        params: A::Parameters,
        winit: WindowState,
        surface_resize_handle: ResizeHandle<SurfaceSize>,
    },
    WaitingForWgpuInit {
        proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
        input_state: RawInputState,
        params: A::Parameters,
        winit: WindowState,
        #[allow(unused)] // this is just to keep the task alive
        task: Task<()>,
    },
    Operational {
        proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
        shin_proxy: ShinEventLoopProxy<A::EventType>,
        input_state: RawInputState,
        winit: WindowState,
        render: RenderResources,
        app: A,
    },
    #[default]
    Poison,
}

impl<A: ShinApp> WinitAppState<A> {
    fn new(proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>, params: A::Parameters) -> Self {
        Self::WaitingForInitialResume {
            proxy,
            params,
            input_state: RawInputState::new(),
        }
    }

    pub fn input_mut(&mut self) -> &mut RawInputState {
        match self {
            WinitAppState::WaitingForInitialResume { input_state, .. } => input_state,
            WinitAppState::WaitingForNonzeroSize { input_state, .. } => input_state,
            WinitAppState::WaitingForWgpuInit { input_state, .. } => input_state,
            WinitAppState::Operational { input_state, .. } => input_state,
            WinitAppState::Poison => {
                unreachable!()
            }
        }
    }

    pub fn winit(&self) -> Option<&WindowState> {
        match self {
            WinitAppState::WaitingForInitialResume { .. } => None,
            WinitAppState::WaitingForNonzeroSize { winit, .. } => Some(winit),
            WinitAppState::WaitingForWgpuInit { winit, .. } => Some(winit),
            WinitAppState::Operational { winit, .. } => Some(winit),
            WinitAppState::Poison => {
                unreachable!()
            }
        }
    }
}

impl<A: ShinApp> ApplicationHandler<ShinAppEventImpl<A::EventType>> for WinitAppState<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match std::mem::take(self) {
            WinitAppState::WaitingForInitialResume {
                proxy,
                input_state,
                params,
            } => {
                let winit = WindowState::new::<A>(event_loop);

                let mut surface_resize_handle = winit.resize_source.surface_handle();
                let current_size = surface_resize_handle.get();

                if current_size != SurfaceSize::default() {
                    let task = start_wgpu_init::<A>(
                        winit.window.clone(),
                        surface_resize_handle,
                        proxy.clone(),
                    );

                    *self = WinitAppState::WaitingForWgpuInit {
                        proxy,
                        input_state,
                        params,
                        winit,
                        task,
                    };
                } else {
                    *self = WinitAppState::WaitingForNonzeroSize {
                        proxy,
                        input_state,
                        params,
                        winit,
                        surface_resize_handle,
                    };
                }
            }
            WinitAppState::WaitingForNonzeroSize { .. } => {
                // this probably shouldn't happen
                todo!()
            }
            WinitAppState::WaitingForWgpuInit { .. } => {
                // this shouldn't happen.. I think
                // TODO: figure out this out better when porting to Android or smth
                todo!()
            }
            WinitAppState::Operational {
                proxy,
                shin_proxy,
                input_state,
                winit,
                mut render,
                app,
            } => {
                render.surface = shin_render::init::surface_reinit(
                    &render.instance,
                    render.device.clone(),
                    winit.window.clone(),
                    winit.resize_source.handle::<SurfaceSize>(),
                    render.surface_texture_format,
                )
                .expect("surface reinit failed");

                *self = WinitAppState::Operational {
                    proxy,
                    shin_proxy,
                    input_state,
                    winit,
                    render,
                    app,
                };
            }
            WinitAppState::Poison => unreachable!(),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ShinAppEventImpl<A::EventType>) {
        match event {
            ShinAppEventImpl::WgpuInitDone(result) => {
                let wgpu = result.expect("wgpu init failed");

                let WinitAppState::WaitingForWgpuInit {
                    proxy,
                    input_state,
                    params,
                    mut winit,
                    task: _,
                } = std::mem::take(self)
                else {
                    unreachable!()
                };

                // finish render init
                // the depth stencil should be the same size as the surface, even though it's a bit wasteful
                let mut render = RenderResources::new(
                    wgpu,
                    winit.resize_source.handle::<SurfaceSize>(),
                    winit.resize_source.handle::<CanvasSize>(),
                );

                let shin_proxy = ShinEventLoopProxy {
                    proxy: proxy.clone(),
                };

                let context = AppContext {
                    event_loop_proxy: &shin_proxy,
                    input: &input_state,
                    winit: &mut winit,
                    render: &mut render,
                };

                let app = A::init(context, params);

                *self = WinitAppState::Operational {
                    proxy,
                    shin_proxy,
                    input_state,
                    winit,
                    render,
                    app,
                };
            }
            ShinAppEventImpl::Custom(e) => {
                let WinitAppState::Operational {
                    proxy: _,
                    shin_proxy,
                    input_state,
                    winit,
                    render,
                    app,
                } = self
                else {
                    warn!("Received custom event before app was initialized");
                    return;
                };

                app.custom_event(
                    AppContext {
                        event_loop_proxy: shin_proxy,
                        input: input_state,
                        winit,
                        render,
                    },
                    e,
                );
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.input_mut().on_winit_event(&event);

        let Some(winit) = self.winit() else {
            return;
        };

        if window_id != winit.window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                winit
                    .resize_source
                    .resize(A::map_canvas_size(physical_size));
                winit.window.request_redraw();

                // this is a mouthful...
                match std::mem::take(self) {
                    WinitAppState::WaitingForNonzeroSize {
                        proxy,
                        input_state,
                        params,
                        mut surface_resize_handle,
                        winit,
                    } => {
                        if surface_resize_handle.get() != SurfaceSize::default() {
                            let task = start_wgpu_init::<A>(
                                winit.window.clone(),
                                surface_resize_handle.clone(),
                                proxy.clone(),
                            );

                            *self = WinitAppState::WaitingForWgpuInit {
                                proxy,
                                input_state,
                                params,
                                winit,
                                task,
                            };
                        } else {
                            *self = WinitAppState::WaitingForNonzeroSize {
                                proxy,
                                input_state,
                                params,
                                winit,
                                surface_resize_handle,
                            };
                        }
                    }
                    state => *self = state,
                }
            }
            WindowEvent::RedrawRequested => {
                let WinitAppState::Operational {
                    proxy: _,
                    input_state,
                    shin_proxy,
                    winit,
                    render,
                    app,
                } = self
                else {
                    return;
                };

                app.update(AppContext {
                    event_loop_proxy: shin_proxy,
                    input: input_state,
                    winit,
                    render,
                });

                let mut encoder = render
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                let (viewport, surface_texture) = render.surface.get_current_texture().unwrap();

                let mut pass = RenderPass::new(
                    &mut render.pipelines,
                    &mut render.dynamic_buffer,
                    &render.device,
                    &mut encoder,
                    &surface_texture.view,
                    &render.surface_depth_stencil_buffer.get_view(),
                    viewport,
                );

                app.render(&mut pass);

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

pub fn run_window<A: ShinApp>(
    parameters: A::Parameters,
    #[cfg(target_os = "android")] android_app: winit::platform::android::activity::AndroidApp,
) {
    #[allow(unused_mut)]
    let mut event_loop_builder = EventLoop::<ShinAppEventImpl<A::EventType>>::with_user_event();

    #[cfg(target_os = "android")]
    {
        event_loop_builder = event_loop_builder.android_app(android_app);
    }

    let event_loop = event_loop_builder.build().unwrap();
    let proxy = event_loop.create_proxy();

    event_loop
        .run_app(&mut WinitAppState::<A>::new(proxy, parameters))
        .unwrap();
}
