use crate::asset::gpu_image::LazyGpuTexture;
use crate::asset::texture_archive::TextureArchive;

#[derive(TextureArchive)]
pub struct MessageboxTextures {
    #[txa(name = "keywait")]
    pub keywait: LazyGpuTexture,
    #[txa(name = "select")]
    pub select: LazyGpuTexture,
    #[txa(name = "select_cur")]
    pub select_cursor: LazyGpuTexture,

    #[txa(name = "msgwnd1")]
    pub message_window_1: LazyGpuTexture,
    #[txa(name = "msgwnd2")]
    pub message_window_2: LazyGpuTexture,
    #[txa(name = "msgwnd3")]
    pub message_window_3: LazyGpuTexture,
}
