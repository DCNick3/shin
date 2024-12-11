//! Contains functionality pertaining to the asset system itself.

mod locate;
mod server;

pub use self::{
    locate::locate_assets,
    server::{
        Asset, AssetDataAccessor, AssetDataCursor, AssetLoadContext, AssetServer, LayeredAssetIo,
    },
};
