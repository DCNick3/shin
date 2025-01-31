use shin_core::format::scenario::Scenario;

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

impl Asset for Scenario {
    type Args = ();

    async fn load(
        _context: &AssetLoadContext,
        _args: (),
        _name: &str,
        data: AssetDataAccessor,
    ) -> anyhow::Result<Self> {
        Scenario::new(data.read_all().await.into())
    }
}
