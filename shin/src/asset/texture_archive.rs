use anyhow::Result;
pub use shin_derive::TextureArchive;

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

// TODO: add strong typing with derive or smth

pub trait TextureArchiveBuilder {
    type Output;

    fn new() -> Self;
    fn add_texture(&mut self, name: &str, texture: ());
    fn build(self) -> Self::Output;
}

pub trait TextureArchive: Sync + Send + 'static {
    type Builder: TextureArchiveBuilder<Output = Self>;
}

impl<T: TextureArchive> Asset for T {
    async fn load(_context: &AssetLoadContext, data: AssetDataAccessor) -> Result<Self> {
        let archive =
            shin_core::format::texture_archive::read_texture_archive(&data.read_all().await)?;

        let mut builder = T::Builder::new();
        let mut textures = archive.textures.into_iter().map(Some).collect::<Vec<_>>();

        todo!()

        // for (name, index) in archive.name_to_index.into_iter() {
        //     let texture = textures[index].take().unwrap();
        //     let image = LazyGpuTexture::new(texture, Some(&format!("TextureArchive[{:?}]", name)));
        //     builder.add_texture(&name, image);
        // }
        //
        // Ok(builder.build())
    }
}
