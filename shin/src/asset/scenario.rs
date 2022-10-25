use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bytes::Bytes;
use std::sync::Arc;

#[derive(Clone, TypeUuid)]
#[uuid = "dcb513fc-b0cd-4fde-9567-b5f65f570231"]
pub struct Scenario(pub Arc<shin_core::format::scenario::Scenario>);

pub struct ScenarioLoader;

impl AssetLoader for ScenarioLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let scenario =
                shin_core::format::scenario::Scenario::new(Bytes::copy_from_slice(bytes))?;
            load_context.set_default_asset(LoadedAsset::new(Scenario(Arc::new(scenario))));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["snr"]
    }
}

pub struct ScenarioPlugin;

impl Plugin for ScenarioPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Scenario>().add_asset_loader(ScenarioLoader);
    }
}
