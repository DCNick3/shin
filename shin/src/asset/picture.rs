use crate::asset::Asset;
use anyhow::Result;
use glam::vec2;
use shin_core::format::picture::SimpleMergedPicture;
use shin_render::GpuCommonResources;
use shin_render::{GpuImage, LazyGpuImage};

/// A Picture, uploaded to GPU on demand (because doing it in the asset loading context is awkward)
pub struct Picture {
    picture: LazyGpuImage,
}

impl Picture {
    pub fn gpu_image(&self, resources: &GpuCommonResources) -> &GpuImage {
        self.picture.gpu_image(resources)
    }
}

impl Asset for Picture {
    fn load_from_bytes(data: Vec<u8>) -> Result<Self> {
        let picture = shin_core::format::picture::read_picture::<SimpleMergedPicture>(&data, ())?;
        let picture_id = picture.picture_id;
        let picture = LazyGpuImage::new(
            picture.image,
            vec2(picture.origin_x as f32, picture.origin_y as f32),
            Some(&format!("Picture {:08x}", picture_id)),
        );

        Ok(Self { picture })
    }
}
