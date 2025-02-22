use std::{
    fmt::Debug,
    sync::Arc,
    time::{Duration, Instant},
};

use cfg_if::cfg_if;
use derive_where::derive_where;
use enum_map::EnumMap;
use shin_input::{Action, ActionState, ActionsState, RawInputAccumulator};
use shin_render::{
    init::{RenderResources, WgpuInitResult, WgpuResources},
    render_pass::RenderPass,
    resize::{CanvasSize, ResizeHandle, SurfaceResizeSource, SurfaceSize, ViewportParams},
    shaders::types::texture::{DepthStencilTarget, TextureTarget, TextureTargetKind},
};
use shin_tasks::AsyncTask;
use tracing::{info, warn};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopClosed, EventLoopProxy},
    window::{Window, WindowId},
};

pub struct AppContext<'a, A: ShinApp> {
    pub event_loop: &'a ActiveEventLoop,
    pub event_loop_proxy: &'a ShinEventLoopProxy<A::EventType>,
    pub winit: &'a mut WindowState,
    pub wgpu: &'a WgpuResources,
    pub render: &'a mut RenderResources,
}

pub struct RenderContext<'a> {
    pub winit: &'a WindowState,
    pub wgpu: &'a WgpuResources,
}

pub struct AppContextOwned<A: ShinApp> {
    pub event_loop_proxy: ShinEventLoopProxy<A::EventType>,
    pub winit: WindowState,
    pub wgpu: WgpuResources,
    pub render: RenderResources,
}

pub trait ShinApp: Sized {
    type Parameters;
    type EventType: Send + Debug + 'static;
    type ActionType: Action;

    fn init(context: AppContext<Self>, parameters: Self::Parameters) -> anyhow::Result<Self>;

    fn map_canvas_size(window_size: PhysicalSize<u32>) -> ViewportParams {
        ViewportParams::with_aspect_ratio(window_size, 16.0 / 9.0)
    }

    fn custom_event(&mut self, context: AppContext<Self>, event: Self::EventType);

    fn update(
        &mut self,
        context: AppContext<Self>,
        input: EnumMap<Self::ActionType, ActionState>,
        elapsed_time: Duration,
    );
    // can't pass context here because `RenderPass` borrows a bunch of stuff from there
    // let's hope it won't be an issue ;)
    fn render(&mut self, context: RenderContext, pass: &mut RenderPass);
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

    pub fn toggle_fullscreen(&self) {
        let window = &self.window;

        if window.fullscreen().is_some() {
            info!("Exiting fullscreen mode");
            window.set_fullscreen(None);
        } else if let Some(monitor) = window.current_monitor() {
            if let Some(video_mode) = monitor.video_modes().next() {
                info!(
                    "Attempting to enter exclusive fullscreen mode {}",
                    video_mode
                );
                window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(video_mode)));
            }
            if window.fullscreen().is_none() {
                info!(
                    "Attempting to enter non-exclusive fullscreen mode on {}",
                    monitor
                        .name()
                        .unwrap_or_else(|| "unknown monitor".to_string())
                );
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(monitor))));
            }
        }
    }
}

#[derive(Debug)]
enum ShinAppEventImpl<E> {
    #[allow(dead_code)]
    WgpuInitDone(anyhow::Result<WgpuInitResult<'static>>),
    Custom(E),
}

#[inline]
fn start_wgpu_init<A: ShinApp>(
    #[allow(unused)] event_loop: &ActiveEventLoop,
    proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
    raw_input_state: RawInputAccumulator,
    params: A::Parameters,
    winit: WindowState,
    // window: Arc<Window>,
    // surface_resize_handle: ResizeHandle<SurfaceSize>,
) -> WinitAppState<A> {
    let window = winit.window.clone();
    let surface_resize_handle = winit.resize_source.handle();

    cfg_if! {
        if #[cfg(not(windows))] {
            // async init
            // this is not necessarily required anywhere besides wasm (we can't do sync init there)
            // however if we have to choose async init is a bit nicer because we can continue handling events
            let proxy_clone = proxy.clone();
            let task = shin_tasks::async_io::spawn(async move {
                let result = shin_render::init::init_wgpu(window, surface_resize_handle, None).await;

                proxy_clone
                    .send_event(ShinAppEventImpl::WgpuInitDone(result))
                    .unwrap();
            });

            WinitAppState::WaitingForWgpuInit {
                proxy,
                raw_input_state,
                params,
                winit,
                task,
            }
        } else {
            // sync init
            // windows want accesses to window handles to be on the main thread, so we have to do this synchronously
            let result = shin_tasks::block_on(shin_render::init::init_wgpu(
                window,
                surface_resize_handle,
                None,
            ));
            finish_wgpu_init(event_loop, proxy, raw_input_state, params, winit, result)
        }
    }
}

#[inline]
fn finish_wgpu_init<A: ShinApp>(
    event_loop: &ActiveEventLoop,
    proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
    raw_input_state: RawInputAccumulator,
    params: A::Parameters,
    mut winit: WindowState,
    result: anyhow::Result<WgpuInitResult<'static>>,
) -> WinitAppState<A> {
    let wgpu = result.expect("wgpu init failed");

    // finish render init
    // the depth stencil should be the same size as the surface, even though it's a bit wasteful
    let (wgpu, mut render) = wgpu.into_resources(
        winit.resize_source.handle::<SurfaceSize>(),
        winit.resize_source.handle::<CanvasSize>(),
    );

    let shin_proxy = ShinEventLoopProxy {
        proxy: proxy.clone(),
    };

    let context = AppContext {
        event_loop,
        event_loop_proxy: &shin_proxy,
        winit: &mut winit,
        wgpu: &wgpu,
        render: &mut render,
    };

    let app = A::init(context, params)
        // TODO: report this error to the user better than just panicking
        .expect("App initialization failed");

    let context = AppContextOwned {
        event_loop_proxy: shin_proxy,
        winit,
        wgpu,
        render,
    };

    WinitAppState::Operational {
        proxy,
        last_update: Instant::now(),
        raw_input_state,
        input_state: ActionsState::new(),
        context,
        app,
    }
}

#[derive(Default)]
enum WinitAppState<A: ShinApp> {
    WaitingForInitialResume {
        // unfortunately we have to weave in the proxy from outside of the loop until winit 0.31: https://github.com/rust-windowing/winit/pull/3764
        proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
        raw_input_state: RawInputAccumulator,
        params: A::Parameters,
    },
    /// On web platform the initial winit canvas size is zero, so we have to wait for a non-zero size
    WaitingForNonzeroSize {
        proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
        raw_input_state: RawInputAccumulator,
        params: A::Parameters,
        winit: WindowState,
        surface_resize_handle: ResizeHandle<SurfaceSize>,
    },
    #[allow(dead_code)]
    WaitingForWgpuInit {
        proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
        raw_input_state: RawInputAccumulator,
        params: A::Parameters,
        winit: WindowState,
        #[allow(unused)] // this is just to keep the task alive
        task: AsyncTask<()>,
    },
    Operational {
        proxy: EventLoopProxy<ShinAppEventImpl<A::EventType>>,
        last_update: Instant,
        raw_input_state: RawInputAccumulator,
        input_state: ActionsState<A::ActionType>,
        context: AppContextOwned<A>,
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
            raw_input_state: RawInputAccumulator::new(),
        }
    }

    pub fn input_mut(&mut self) -> &mut RawInputAccumulator {
        match self {
            WinitAppState::WaitingForInitialResume {
                raw_input_state: input_state,
                ..
            } => input_state,
            WinitAppState::WaitingForNonzeroSize {
                raw_input_state: input_state,
                ..
            } => input_state,
            WinitAppState::WaitingForWgpuInit {
                raw_input_state: input_state,
                ..
            } => input_state,
            WinitAppState::Operational {
                raw_input_state: input_state,
                ..
            } => input_state,
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
            WinitAppState::Operational { context, .. } => Some(&context.winit),
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
                raw_input_state: input_state,
                params,
            } => {
                let winit = WindowState::new::<A>(event_loop);

                let mut surface_resize_handle = winit.resize_source.surface_handle();
                let current_size = surface_resize_handle.get();

                if current_size != SurfaceSize::default() {
                    *self = start_wgpu_init::<A>(event_loop, proxy, input_state, params, winit);
                } else {
                    *self = WinitAppState::WaitingForNonzeroSize {
                        proxy,
                        raw_input_state: input_state,
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
                last_update,
                raw_input_state,
                input_state,
                mut context,
                app,
            } => {
                context.render.surface = shin_render::init::surface_reinit(
                    &context.wgpu.instance,
                    context.wgpu.device.clone(),
                    context.winit.window.clone(),
                    context.winit.resize_source.handle::<SurfaceSize>(),
                    context.render.surface_texture_format,
                )
                .expect("surface reinit failed");

                *self = WinitAppState::Operational {
                    proxy,
                    last_update,
                    raw_input_state,
                    input_state,
                    context,
                    app,
                };
            }
            WinitAppState::Poison => unreachable!(),
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ShinAppEventImpl<A::EventType>) {
        match event {
            ShinAppEventImpl::WgpuInitDone(result) => {
                let WinitAppState::WaitingForWgpuInit {
                    proxy,
                    raw_input_state,
                    params,
                    winit,
                    task: _,
                } = std::mem::take(self)
                else {
                    unreachable!()
                };
                *self = finish_wgpu_init::<A>(
                    event_loop,
                    proxy,
                    raw_input_state,
                    params,
                    winit,
                    result,
                );
            }
            ShinAppEventImpl::Custom(e) => {
                let WinitAppState::Operational {
                    proxy: _,
                    last_update: _,
                    raw_input_state: _,
                    input_state: _,
                    context:
                        AppContextOwned {
                            event_loop_proxy,
                            winit,
                            wgpu,
                            render,
                        },
                    app,
                } = self
                else {
                    warn!("Received custom event before app was initialized");
                    return;
                };

                app.custom_event(
                    AppContext {
                        event_loop,
                        event_loop_proxy,
                        wgpu,
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
                        raw_input_state: input_state,
                        params,
                        mut surface_resize_handle,
                        winit,
                    } => {
                        if surface_resize_handle.get() != SurfaceSize::default() {
                            *self =
                                start_wgpu_init::<A>(event_loop, proxy, input_state, params, winit);
                        } else {
                            *self = WinitAppState::WaitingForNonzeroSize {
                                proxy,
                                raw_input_state: input_state,
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
                    last_update,
                    raw_input_state,
                    input_state,
                    context:
                        AppContextOwned {
                            event_loop_proxy,
                            winit,
                            wgpu,
                            render,
                        },
                    app,
                } = self
                else {
                    return;
                };

                let now_update = Instant::now();
                let elapsed = now_update.duration_since(*last_update);
                *last_update = now_update;

                let raw_instantaneous_input_state = raw_input_state.start_frame();
                let instantaneous_input_state =
                    A::ActionType::lower(&raw_instantaneous_input_state);
                // NOTE: this interface does not expose analog stick positions, as well as mouse wheel (and buttons...)
                // these should probably be passed directly or some other abstraction should be devised
                let input_state = input_state.update(instantaneous_input_state, elapsed);

                app.update(
                    AppContext {
                        event_loop,
                        event_loop_proxy,
                        winit,
                        wgpu,
                        render,
                    },
                    input_state,
                    elapsed,
                );
                raw_input_state.finish_frame();

                let mut encoder = wgpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                let (viewport, surface_texture) = render.surface.get_current_texture().unwrap();

                let mut pass = RenderPass::new(
                    &mut render.pipelines,
                    &mut render.dynamic_buffer,
                    &render.sampler_store,
                    &wgpu.device,
                    &mut encoder,
                    TextureTarget {
                        kind: TextureTargetKind::Screen,
                        view: &surface_texture.view,
                    },
                    DepthStencilTarget {
                        view: render.surface_depth_stencil_buffer.resize_and_get_view(),
                    },
                    Some(viewport),
                );

                let context = RenderContext { winit, wgpu };

                app.render(context, &mut pass);

                drop(pass);

                wgpu.queue.submit(std::iter::once(encoder.finish()));

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
