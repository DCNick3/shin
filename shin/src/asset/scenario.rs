use shin_core::format::scenario::Scenario;

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

impl Asset for Scenario {
    async fn load(_context: &AssetLoadContext, data: AssetDataAccessor) -> anyhow::Result<Self> {
        Scenario::new(data.read_all().await.into())
    }
}
