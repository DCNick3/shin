use std::sync::Arc;

use shin_core::format::scenario::Scenario;

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

impl Asset for Scenario {
    type Args = ();

    async fn load(
        _context: &Arc<AssetLoadContext>,
        _args: (),
        _name: &str,
        data: AssetDataAccessor,
    ) -> anyhow::Result<Self> {
        Scenario::new(data.read_all().await.into())
    }
}
