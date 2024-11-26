use shin::asset::AssetDataAccessor;
use shin_core::format::scenario::Scenario;

use crate::asset::Asset;

impl Asset for Scenario {
    async fn load(data: AssetDataAccessor) -> anyhow::Result<Self> {
        Scenario::new(data.read_all().await.into())
    }
}
