use crate::asset::{asset_paths, AnyAssetServer};
use crate::layer::MessageboxTextures;
use anyhow::Result;
use futures::try_join;
use shin_core::format::font::LazyFont;
use shin_core::format::scenario::Scenario;
use std::sync::Arc;

// TODO: this can be done with a macro
#[derive(Clone)]
pub struct AdvAssets {
    pub scenario: Arc<Scenario>,
    pub fonts: AdvFonts,
    pub messagebox_textures: Arc<MessageboxTextures>,
}

#[derive(Clone)]
pub struct AdvFonts {
    pub system_font: Arc<LazyFont>,
    pub medium_font: Arc<LazyFont>,
    pub bold_font: Arc<LazyFont>,
}

impl AdvAssets {
    pub async fn load(asset_server: &AnyAssetServer) -> Result<Self> {
        let result = try_join!(
            asset_server.load(asset_paths::SCENARIO),
            AdvFonts::load(asset_server),
            asset_server.load(asset_paths::MSGTEX),
        )?;

        Ok(Self {
            scenario: result.0,
            fonts: result.1,
            messagebox_textures: result.2,
        })
    }
}

impl AdvFonts {
    pub async fn load(asset_server: &AnyAssetServer) -> Result<Self> {
        let result = try_join!(
            asset_server.load(asset_paths::SYSTEM_FNT),
            asset_server.load(asset_paths::NEWRODIN_MEDIUM_FNT),
            asset_server.load(asset_paths::NEWRODIN_BOLD_FNT),
        )?;

        Ok(Self {
            system_font: result.0,
            medium_font: result.1,
            bold_font: result.2,
        })
    }
}
