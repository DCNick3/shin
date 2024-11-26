use anyhow::Result;
use glam::vec2;
use shin_core::format::picture::SimpleMergedPicture;

use crate::asset::{Asset, AssetDataAccessor};

/// A Picture, uploaded to GPU on demand (because doing it in the asset loading context is awkward)
pub struct Picture {
    // picture: LazyGpuImage,
}

impl Picture {
    // pub fn gpu_image(&self, resources: &GpuCommonResources) -> &GpuImage {
    //     self.picture.gpu_image(resources)
    // }
}

impl Asset for Picture {
    async fn load(data: AssetDataAccessor) -> Result<Self> {
        let picture = shin_core::format::picture::read_picture::<SimpleMergedPicture>(
            &data.read_all().await,
            (),
        )?;
        let picture_id = picture.picture_id;

        todo!()

        // let picture = LazyGpuImage::new(
        //     picture.image,
        //     vec2(picture.origin_x as f32, picture.origin_y as f32),
        //     Some(&format!("Picture {:08x}", picture_id)),
        // );
        //
        // Ok(Self { picture })
    }
}
