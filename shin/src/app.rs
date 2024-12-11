use std::{sync::Arc, time::Duration};

use anyhow::Context;
use enum_map::{Enum, EnumMap};
use glam::vec4;
use shin_audio::AudioManager;
use shin_core::vm::command::types::{LayerId, LayerProperty, LayerbankId, PlaneId};
use shin_input::{Action, ActionState, RawInputState};
use shin_render::{render_pass::RenderPass, shaders::types::vertices::FloatColor4};
use shin_window::{AppContext, RenderContext, ShinApp};
use tracing::debug;
use winit::keyboard::KeyCode;

use crate::{
    asset::{
        picture::Picture,
        system::{locate_assets, AssetLoadContext, AssetServer},
    },
    cli::Cli,
    layer::{
        render_layer,
        render_params::TransformParams,
        user::{PictureLayer, TileLayer},
        DrawableLayer, Layer as _, PageLayer, PreRenderContext,
    },
};

#[derive(Debug, Enum)]
pub enum AppAction {
    ToggleFullscreen,
}

impl Action for AppAction {
    fn lower(raw_input_state: &RawInputState) -> EnumMap<Self, bool> {
        EnumMap::from_fn(|action| match action {
            AppAction::ToggleFullscreen => raw_input_state.keyboard.contains(&KeyCode::F11),
        })
    }
}

pub struct App {
    audio_manager: Arc<AudioManager>,
    asset_server: Arc<AssetServer>,
    page_layer: PageLayer,
}

impl ShinApp for App {
    type Parameters = Cli;
    type EventType = ();
    type ActionType = AppAction;

    fn init(context: AppContext<Self>, cli: Self::Parameters) -> anyhow::Result<Self> {
        let audio_manager = Arc::new(AudioManager::new());

        let asset_io = locate_assets(cli.assets_dir.as_deref()).context("Failed to locate assets. Consult the README for instructions on how to set up the game.")?;

        debug!("Asset IO: {:#?}", asset_io);

        let asset_server = Arc::new(AssetServer::new(
            asset_io.into(),
            AssetLoadContext {
                wgpu_device: context.wgpu.device.clone(),
                wgpu_queue: context.wgpu.queue.clone(),
            },
        ));

        let picture_name = "/picture/text001.pic";

        let picture = asset_server.load_sync::<Picture>(picture_name).unwrap();

        let mut tile_layer_bottom = TileLayer::new(
            FloatColor4::PASTEL_PINK,
            vec4(-1088.0, -612.0, 2176.0, 1224.0),
        );
        tile_layer_bottom
            .properties_mut()
            .set_layer_id(LayerId::new(1));

        let mut tile_layer_top = TileLayer::new(
            FloatColor4::PASTEL_GREEN,
            vec4(-300.0, -300.0, 600.0, 600.0),
        );
        {
            let props = tile_layer_top.properties_mut();
            props.set_layer_id(LayerId::new(3));
            props
                .property_tweener_mut(LayerProperty::MulColorAlpha)
                .fast_forward_to(200.0);
        }

        let mut picture_layer = PictureLayer::new(picture, Some(picture_name.to_string()));
        {
            let props = picture_layer.properties_mut();
            props.set_layer_id(LayerId::new(2));
            props
                .property_tweener_mut(LayerProperty::TranslateZ)
                .fast_forward_to(1500.0);
            props
                .property_tweener_mut(LayerProperty::MulColorAlpha)
                .fast_forward_to(800.0);
        }

        let mut page_layer = PageLayer::new(4, None);

        let layer_group = page_layer.get_plane_mut(PlaneId::new(0));
        layer_group.add_layer(LayerbankId::new(1), tile_layer_bottom.into());
        layer_group.add_layer(LayerbankId::new(0), picture_layer.into());

        let layer_group = page_layer.get_plane_mut(PlaneId::new(1));
        layer_group.add_layer(LayerbankId::new(2), tile_layer_top.into());

        Ok(Self {
            audio_manager,
            asset_server,
            page_layer,
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

        let transform = TransformParams::default();

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

        self.page_layer
            .pre_render(&mut pre_render_context, &transform);

        context.wgpu.queue.submit(std::iter::once(encoder.finish()));
    }

    fn render(&mut self, _context: RenderContext, pass: &mut RenderPass) {
        let transform = TransformParams::default();

        render_layer(pass, &transform, &self.page_layer, FloatColor4::BLACK, 0);
    }
}
