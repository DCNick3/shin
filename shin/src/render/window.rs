use tracing::warn;

use shin_core::vm::command::layer::LayerProperty;
use winit::dpi::LogicalSize;
use winit::window::Fullscreen;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use super::pipelines::Pipelines;

use crate::asset::picture::GpuPicture;
use crate::interpolator::Easing;
use crate::layer::{Layer, PictureLayer};
use crate::render::bind_groups::BindGroupLayouts;
use crate::render::camera::Camera;
use crate::render::common_resources::GpuCommonResources;
use crate::render::pillarbox::Pillarbox;
use crate::render::pipelines::CommonBinds;
use crate::render::Renderable;
use crate::update::{Ticks, Updatable, UpdateContext};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

struct State {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    resources: GpuCommonResources,
    // TODO: do we want to pull the bevy deps?
    time: bevy_time::Time,
    camera: Camera,
    pillarbox: Pillarbox,
    bg_pic: PictureLayer,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::PUSH_CONSTANTS,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: wgpu::Limits {
                        max_texture_dimension_2d: 4096,
                        max_push_constant_size: 64,

                        ..wgpu::Limits::downlevel_webgl2_defaults()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        // TODO: make a better selection?
        let texture_format = surface.get_supported_formats(&adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let bind_group_layouts = BindGroupLayouts::new(&device);
        let pipelines = Pipelines::new(&device, &bind_group_layouts, texture_format);

        let window_size = window.inner_size();
        let camera = Camera::new(
            &device,
            &bind_group_layouts,
            (window_size.width, window_size.height),
        );

        let resources = GpuCommonResources {
            device,
            queue,
            texture_format,
            bind_group_layouts,
            pipelines,
            common_binds: CommonBinds {
                camera: camera.bind_group().clone(),
            },
        };

        let pillarbox = Pillarbox::new(&resources);

        let bg_pic = std::fs::read("assets/ship_p1a.pic").unwrap();
        let bg_pic = crate::asset::picture::load_picture(&bg_pic).unwrap();
        let bg_pic = GpuPicture::load(&resources, bg_pic);
        let mut bg_pic = PictureLayer::new(&resources, bg_pic);

        // test the interpolators
        let props = bg_pic.properties_mut();
        props.set_property(LayerProperty::Rotation, 400.0, Ticks(180.0), Easing::EaseIn);
        props.set_property(
            LayerProperty::Rotation,
            -400.0,
            Ticks(240.0),
            Easing::Identity,
        );
        props.set_property(LayerProperty::Rotation, 0.0, Ticks(180.0), Easing::EaseOut);

        Self {
            surface,
            config,
            size,
            resources,
            time: bevy_time::Time::default(),
            camera,
            pillarbox,
            bg_pic,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.resources.device, &self.config);

            let new_size = (new_size.width, new_size.height);

            self.camera
                .resize(&self.resources.device, &mut self.resources.queue, new_size);

            // self.pillarbox.resize(&gpu_resources, new_size);
            self.bg_pic.resize(&self.resources, new_size);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        self.time.update();

        let update_context = UpdateContext {
            time: &self.time,
            gpu_resources: &self.resources,
        };

        self.bg_pic.update(&update_context);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.resources.start_encoder();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Root Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.bg_pic.render(&self.resources, &mut render_pass);
            // self.pillarbox.render(&mut render_context);
        }

        drop(encoder);

        output.present();

        Ok(())
    }
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

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1920, 1080))
        .with_maximized(true)
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
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            state.resize(**new_inner_size);
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
                        state.resize(state.size)
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
