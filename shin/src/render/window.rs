use std::path::PathBuf;
use tracing::{debug, warn};

use shin_core::format::scenario::Scenario;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::Fullscreen;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use super::pipelines::Pipelines;

use crate::adv::Adv;
use crate::game_data::GameData;
use crate::render::bind_groups::BindGroupLayouts;
use crate::render::camera::Camera;
use crate::render::common_resources::GpuCommonResources;
use crate::render::pillarbox::Pillarbox;
use crate::render::{RenderTarget, Renderable, SpriteVertexBuffer};
use crate::update::{Updatable, UpdateContext};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

struct State {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    window_size: (u32, u32),
    resources: GpuCommonResources,
    // TODO: do we want to pull the bevy deps?
    time: bevy_time::Time,
    vertices: SpriteVertexBuffer,
    render_target: RenderTarget,
    pillarbox: Pillarbox,
    game_data: GameData,
    adv: Adv,
}

impl State {
    async fn new(window: &Window) -> Self {
        let window_size = window.inner_size();
        let window_size = (window_size.width, window_size.height);

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
                        // TODO: maybe we should use uniform buffers more...
                        max_push_constant_size: 256,

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

        let resources = GpuCommonResources {
            device,
            queue,
            bind_group_layouts,
            pipelines,
            camera,
        };

        let vertices = SpriteVertexBuffer::new_fullscreen(&resources);
        let render_target = RenderTarget::new(
            &resources,
            resources.current_render_buffer_size(),
            Some("Window RenderTarget"),
        );

        let pillarbox = Pillarbox::new(&resources);

        // let bg_pic = std::fs::read("assets/ship_p1a.pic").unwrap();
        // let bg_pic = crate::asset::picture::load_picture(&bg_pic).unwrap();
        // let bg_pic = GpuPicture::load(&resources, bg_pic);
        // let mut bg_pic = PictureLayer::new(&resources, bg_pic);
        //
        // // test the interpolators
        // let props = bg_pic.properties_mut();
        // props.set_property(LayerProperty::Rotation, 400.0, Ticks(180.0), Easing::EaseIn);
        // props.set_property(
        //     LayerProperty::Rotation,
        //     -400.0,
        //     Ticks(240.0),
        //     Easing::Identity,
        // );
        // props.set_property(LayerProperty::Rotation, 0.0, Ticks(180.0), Easing::EaseOut);
        //
        // let mut layer_group = LayerGroup::new(&resources);
        // layer_group.add_layer(LayerId::new(1), bg_pic.into());
        //
        // let props = layer_group.properties_mut();
        // props.set_property(
        //     LayerProperty::TranslateY,
        //     400.0,
        //     Ticks(180.0),
        //     Easing::EaseIn,
        // );
        // props.set_property(
        //     LayerProperty::TranslateY,
        //     -400.0,
        //     Ticks(240.0),
        //     Easing::Identity,
        // );
        // props.set_property(
        //     LayerProperty::TranslateY,
        //     0.0,
        //     Ticks(180.0),
        //     Easing::EaseOut,
        // );

        let game_data = GameData::new(PathBuf::from("assets/data"));

        let scenario = game_data.read_file("/main.snr");
        let scenario = Scenario::new(scenario.into()).expect("Parsing scenario");
        let adv = Adv::new(&resources, scenario, 0, 42);

        Self {
            surface,
            surface_config: config,
            window_size,
            resources,
            time: bevy_time::Time::default(),
            vertices,
            render_target,
            pillarbox,
            game_data,
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

            self.resources.camera.resize(new_size);
            self.render_target
                .resize(&self.resources, self.resources.current_render_buffer_size());

            debug!(
                "Window resized to {:?}, new render buffer size is {:?}",
                new_size,
                self.resources.current_render_buffer_size()
            );

            self.pillarbox.resize(&self.resources);
            self.adv.resize(&self.resources);
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
            game_data: &self.game_data,
        };

        self.adv.update(&update_context);
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
                self.resources.projection_matrix(),
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
                self.vertices.vertex_source(),
                self.render_target.bind_group(),
                self.resources.camera.screen_projection_matrix(),
            );
            self.pillarbox.render(
                &self.resources,
                &mut render_pass,
                self.resources.camera.screen_projection_matrix(),
            );
        }

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
