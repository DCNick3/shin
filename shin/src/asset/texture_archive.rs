use anyhow::Result;
pub use shin_derive::TextureArchive;
use shin_render::gpu_texture::GpuTexture;

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

pub trait TextureArchiveBuilder {
    type Output;

    fn new() -> Self;
    fn add_texture(&mut self, name: &str, texture: GpuTexture);
    fn build(self) -> Self::Output;
}

pub trait TextureArchive: Sync + Send + 'static {
    type Builder: TextureArchiveBuilder<Output = Self>;
}

impl<T: TextureArchive> Asset for T {
    type Args = ();

    async fn load(
        context: &AssetLoadContext,
        _args: (),
        txa_name: &str,
        data: AssetDataAccessor,
    ) -> Result<Self> {
        let archive =
            shin_core::format::texture_archive::read_texture_archive(&data.read_all().await)?;

        let mut builder = T::Builder::new();

        for (name, index) in archive.name_to_index {
            let texture = &archive.textures[index];

            let texture = GpuTexture::new_static_from_rgba_image(
                &context.wgpu_device,
                &context.wgpu_queue,
                Some(&format!("TXA[{}]/{}", txa_name, name)),
                texture,
            );

            builder.add_texture(&name, texture);
        }

        Ok(builder.build())
    }
}
