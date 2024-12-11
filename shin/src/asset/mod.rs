mod audio;
#[expect(unused)]
pub mod bustup;
mod font;
pub mod movie;
pub mod picture;
mod scenario;
pub mod system;
#[expect(unused)]
pub mod texture_archive;

/// Common asset paths used in the game.
pub mod asset_paths {
    pub const SCENARIO: &str = "/main.snr";
    pub const SYSTEM_FNT: &str = "/system.fnt";
    #[expect(unused)]
    pub const MSGTEX: &str = "/msgtex.txa";
    pub const NEWRODIN_MEDIUM_FNT: &str = "/newrodin-medium.fnt";
    pub const NEWRODIN_BOLD_FNT: &str = "/newrodin-bold.fnt";
}
