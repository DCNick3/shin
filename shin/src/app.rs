use std::{sync::Arc, time::Duration};

use anyhow::Context;
use enum_map::EnumMap;
use shin_audio::AudioManager;
use shin_input::{ActionState, DummyAction};
use shin_render::render_pass::RenderPass;
use shin_window::{AppContext, ShinApp};
use tracing::debug;

use crate::{
    asset::{
        picture::Picture,
        system::{locate_assets, AssetLoadContext, AssetServer},
    },
    cli::Cli,
};

pub struct App {
    audio_manager: Arc<AudioManager>,
    asset_server: Arc<AssetServer>,
}

impl ShinApp for App {
    type Parameters = Cli;
    type EventType = ();
    type ActionType = DummyAction;

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

        Ok(Self {
            audio_manager,
            asset_server,
        })
    }

    fn custom_event(&mut self, context: AppContext<Self>, event: Self::EventType) {
        todo!()
    }

    fn update(
        &mut self,
        context: AppContext<Self>,
        input: EnumMap<Self::ActionType, ActionState>,
        elapsed_time: Duration,
    ) {
        // todo!()
    }

    fn render(&mut self, pass: &mut RenderPass) {
        // todo!()
    }
}
