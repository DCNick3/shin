//! Contains functionality pertaining to the asset system itself.

pub mod cache;
mod locate;
mod server;

pub use self::{
    locate::locate_assets,
    server::{
        Asset, AssetDataAccessor, AssetDataCursor, AssetLoadContext, AssetServer, LayeredAssetIo,
    },
};
