pub mod bustup;
mod font;
pub mod gpu_image;
pub mod picture;
mod scenario;
mod server;
pub mod texture_archive;

pub mod asset_paths {
    pub const SCENARIO: &str = "/main.snr";
    pub const SYSTEM_FNT: &str = "/system.fnt";
    pub const MSGTEX: &str = "/msgtex.txa";
    pub const NEWRODIN_MEDIUM_FNT: &str = "/newrodin-medium.fnt";
    pub const NEWRODIN_BOLD_FNT: &str = "/newrodin-bold.fnt";
}

pub use server::{AnyAssetServer, Asset, AssetIo, AssetServer, DirAssetIo, RomAssetIo};
