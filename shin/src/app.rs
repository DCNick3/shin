use std::{sync::Arc, time::Duration};

use anyhow::Context;
use enum_map::{Enum, EnumMap};
use shin_audio::AudioManager;
use shin_core::{
    format::scenario::instruction_elements::CodeAddress, primitives::update::FrameId, time::Ticks,
    vm::Scripter,
};
use shin_input::{Action, ActionState, RawInputState, inputs::MouseButton};
use shin_render::render_pass::RenderPass;
use shin_window::{AppContext, RenderContext, ShinApp};
use tracing::debug;
use winit::keyboard::KeyCode;

use crate::{
    adv::{Adv, assets::AdvAssets},
    asset::system::{AssetLoadContext, AssetServer, cache::AssetCache, locate_assets},
    cli::Cli,
    layer::PreRenderContext,
    update::UpdateContext,
};

#[derive(Debug, Enum)]
pub enum AppAction {
    ToggleFullscreen,
    Act,
    Enter,
    Cancel,
    AnyDown,
    HoldSkip,
}

impl Action for AppAction {
    fn lower(raw_input_state: &RawInputState) -> EnumMap<Self, bool> {
        EnumMap::from_fn(|action| match action {
            AppAction::ToggleFullscreen => raw_input_state.keyboard.contains(&KeyCode::F11),
            AppAction::Act => {
                raw_input_state.keyboard.contains(&KeyCode::Space)
                    || raw_input_state.mouse.buttons[MouseButton::Left]
            }
            AppAction::Enter => {
                raw_input_state.keyboard.contains(&KeyCode::Space)
                    | raw_input_state.keyboard.contains(&KeyCode::Enter)
            }
            AppAction::Cancel => raw_input_state.keyboard.contains(&KeyCode::Backspace),
            AppAction::AnyDown => raw_input_state.keyboard.contains(&KeyCode::ArrowDown),
            AppAction::HoldSkip => raw_input_state.keyboard.contains(&KeyCode::ControlLeft),
        })
    }
}

pub struct App {
    frame_id: FrameId,
    #[expect(unused)] // for future stuff
    audio_manager: Arc<AudioManager>,
    asset_server: Arc<AssetServer>,
    adv: Adv,
}

impl ShinApp for App {
    type Parameters = Cli;
    type EventType = ();
    type ActionType = AppAction;

    fn init(context: AppContext<Self>, cli: Self::Parameters) -> anyhow::Result<Self> {
        let audio_manager = Arc::new(AudioManager::new());

        let asset_io = locate_assets(cli.assets_dir.as_deref()).context("Failed to locate assets. Consult the README for instructions on how to set up the game.")?;

        debug!("Asset IO: {:#?}", asset_io);

        let asset_server = Arc::new(AssetServer::new(asset_io.into(), AssetLoadContext {
            wgpu_device: context.wgpu.device.clone(),
            wgpu_queue: context.wgpu.queue.clone(),
            bustup_cache: AssetCache::new(),
        }));

        // TODO: do not block the game loop (?)
        let adv_assets = shin_tasks::block_on(AdvAssets::load(&asset_server)).unwrap();

        let mut scripter = Scripter::new(&adv_assets.scenario, 0, 42);

        if let Some(addr) = cli.unsafe_entry_point {
            debug!("Starting execution from 0x{:x}", addr);
            scripter.unsafe_set_position(CodeAddress(addr));
        }

        let mut adv = Adv::new(audio_manager.clone(), adv_assets, scripter);

        if let Some(addr) = cli.fast_forward_to {
            debug!("Fast forwarding to 0x{:x}", addr);
            adv.fast_forward_to(CodeAddress(addr));
        }

        // let picture_name = "/picture/text001.pic";
        //
        // let picture = asset_server.load_sync::<Picture>(picture_name).unwrap();
        //
        // let mut tile_layer_bottom = TileLayer::new(
        //     FloatColor4::PASTEL_PINK,
        //     vec4(-1088.0, -612.0, 2176.0, 1224.0),
        // );
        // tile_layer_bottom
        //     .properties_mut()
        //     .set_layer_id(LayerId::new(1));
        //
        // let mut tile_layer_top = TileLayer::new(
        //     FloatColor4::PASTEL_GREEN,
        //     vec4(-300.0, -300.0, 600.0, 600.0),
        // );
        // {
        //     let props = tile_layer_top.properties_mut();
        //     props.set_layer_id(LayerId::new(3));
        //     props
        //         .property_tweener_mut(LayerProperty::MulColorAlpha)
        //         .fast_forward_to(200.0);
        // }
        //
        // let mut picture_layer = PictureLayer::new(picture, Some(picture_name.to_string()));
        // {
        //     let props = picture_layer.properties_mut();
        //     props.set_layer_id(LayerId::new(2));
        //     props
        //         .property_tweener_mut(LayerProperty::TranslateZ)
        //         .fast_forward_to(1500.0);
        //     props
        //         .property_tweener_mut(LayerProperty::MulColorAlpha)
        //         .fast_forward_to(800.0);
        // }
        //
        // let mut root_layer_group = RootLayerGroup::new();
        //
        // let screen_layer = root_layer_group.screen_layer_mut();
        //
        // let page_layer = screen_layer.page_layer_mut();
        //
        // let layer_group = page_layer.get_plane_mut(PlaneId::new(0));
        // layer_group.add_layer(LayerbankId::new(1), tile_layer_bottom.into());
        // layer_group.add_layer(LayerbankId::new(0), picture_layer.into());
        // layer_group.add_layer(LayerbankId::new(2), tile_layer_top.into());
        //
        // {
        //     let tweener = page_layer
        //         .properties_mut()
        //         .property_tweener_mut(LayerProperty::MulColorRed);
        //
        //     tweener.enqueue(2000.0, Tween::linear(Ticks::from_seconds(0.5)));
        //     tweener.enqueue(1000.0, Tween::linear(Ticks::from_seconds(0.5)));
        //     tweener.enqueue(2000.0, Tween::linear(Ticks::from_seconds(0.5)));
        //     tweener.enqueue(1000.0, Tween::linear(Ticks::from_seconds(0.5)));
        //
        //     // tweener.enqueue(2000.0, Tween::linear(Ticks::from_seconds(1.0)));
        //     // tweener.enqueue(0.0, Tween::linear(Ticks::from_seconds(1.0)));
        //     // tweener.enqueue(2000.0, Tween::linear(Ticks::from_seconds(1.0)));
        //     // tweener.enqueue(0.0, Tween::linear(Ticks::from_seconds(1.0)));
        //     // tweener.enqueue(1000.0, Tween::linear(Ticks::from_seconds(0.5)));
        // }

        Ok(Self {
            frame_id: FrameId::default(),
            audio_manager,
            asset_server,
            adv,
        })
    }

    fn custom_event(&mut self, _context: AppContext<Self>, _event: Self::EventType) {
        todo!()
    }

    fn update(
        &mut self,
        context: AppContext<Self>,
        input: EnumMap<Self::ActionType, ActionState>,
        elapsed_time: Duration,
    ) {
        if input[AppAction::ToggleFullscreen].is_clicked {
            context.winit.toggle_fullscreen();
        }

        // if input[AppAction::Act].is_clicked {
        //     let screen_layer = self.root_layer_group.screen_layer_mut();
        //
        //     screen_layer.pageback(false);
        //     screen_layer
        //         .page_layer_mut()
        //         .properties_mut()
        //         .property_tweener_mut(LayerProperty::MulColorGreen)
        //         .fast_forward_to(2000.0);
        //     screen_layer.apply_transition(Some(DefaultWiper::new(Ticks::from_seconds(1.0)).into()));
        // }

        let mut encoder =
            context
                .wgpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("App::update"),
                });

        let mut pre_render_context = PreRenderContext {
            device: &context.wgpu.device,
            queue: &context.wgpu.queue,
            resize_source: &context.winit.resize_source,
            sampler_store: &context.render.sampler_store,
            depth_stencil: context.render.canvas_depth_stencil_buffer.get_target_view(),

            pipeline_storage: &mut context.render.pipelines,
            dynamic_buffer: &mut context.render.dynamic_buffer,
            encoder: &mut encoder,
        };

        let mut update_context = UpdateContext {
            frame_id: self.frame_id,
            delta_ticks: Ticks::from_duration(elapsed_time),
            asset_server: &self.asset_server,
            pre_render: &mut pre_render_context,
        };

        self.adv.update(&mut update_context, input);

        // let update_context = AdvUpdateContext {
        //     delta_time: Ticks::from_duration(elapsed_time),
        //     asset_server: &self.asset_server,
        //     are_animations_allowed: true,
        // };
        //
        // self.root_layer_group.update(&update_context);
        //
        // let transform = TransformParams::default();
        //
        // let mut encoder =
        //     context
        //         .wgpu
        //         .device
        //         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        //             label: Some("App::update"),
        //         });
        //
        // let mut pre_render_context = PreRenderContext {
        //     device: &context.wgpu.device,
        //     queue: &context.wgpu.queue,
        //     resize_source: &context.winit.resize_source,
        //     sampler_store: &context.render.sampler_store,
        //     depth_stencil: context.render.canvas_depth_stencil_buffer.get_target_view(),
        //
        //     pipeline_storage: &mut context.render.pipelines,
        //     dynamic_buffer: &mut context.render.dynamic_buffer,
        //     encoder: &mut encoder,
        // };
        //
        // self.root_layer_group
        //     .pre_render(&mut pre_render_context, &transform);

        context.wgpu.queue.submit(std::iter::once(encoder.finish()));

        self.frame_id.advance();
    }

    fn render(&mut self, _context: RenderContext, pass: &mut RenderPass) {
        self.adv.render(pass);

        // render_layer(pass, &transform, &self.adv, FloatColor4::BLACK, 0);
    }
}
