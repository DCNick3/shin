use crate::asset::gpu_image::{GpuImage, LazyGpuImage};
use crate::asset::Asset;
use crate::render::GpuCommonResources;
use anyhow::{Context, Result};
use bevy_utils::HashMap;
use cgmath::Vector2;

struct BustupExpression {
    face_picture: LazyGpuImage,

    mouth_pictures: Vec<LazyGpuImage>,
}

pub struct Bustup {
    base_picture: LazyGpuImage,
    emotions: HashMap<String, BustupExpression>,
}

impl Bustup {
    pub fn base_gpu_image(&self, resources: &GpuCommonResources) -> &GpuImage {
        self.base_picture.gpu_image(resources)
    }

    pub fn face_gpu_image(&self, resources: &GpuCommonResources, emotion: &str) -> &GpuImage {
        self.emotions
            .get(emotion)
            .with_context(|| format!("No emotion {} in bustup", emotion))
            .unwrap()
            .face_picture
            .gpu_image(resources)
    }

    pub fn mouth_gpu_image(
        &self,
        resources: &GpuCommonResources,
        emotion: &str,
        mouth_intensity: f32,
    ) -> Option<&GpuImage> {
        let emotion = self
            .emotions
            .get(emotion)
            .with_context(|| format!("No emotion {} in bustup", emotion))
            .unwrap();

        if emotion.mouth_pictures.is_empty() {
            return None;
        }

        let mouth_intensity = mouth_intensity.clamp(0.0, 1.0);
        let mouth_index =
            ((emotion.mouth_pictures.len() - 1) as f32 * mouth_intensity).round() as usize;

        Some(emotion.mouth_pictures[mouth_index].gpu_image(resources))
    }
}

impl Asset for Bustup {
    fn load_from_bytes(data: Vec<u8>) -> Result<Self> {
        let bustup = shin_core::format::bustup::read_bustup(&data)?;

        let origin = Vector2::new(bustup.origin.0 as f32, bustup.origin.1 as f32);

        Ok(Self {
            base_picture: LazyGpuImage::new(bustup.base_image, origin, Some("Bustup Base")),
            emotions: bustup
                .expressions
                .into_iter()
                .map(|(name, expression)| {
                    fn chunk_to_gpu_image(
                        chunk: shin_core::format::picture::PictureChunk,
                        origin: Vector2<f32>,
                        label: &str,
                    ) -> LazyGpuImage {
                        LazyGpuImage::new(
                            chunk.data,
                            origin - Vector2::new(chunk.offset_x as f32, chunk.offset_y as f32),
                            Some(label),
                        )
                    }

                    let image = chunk_to_gpu_image(
                        expression.face_chunk,
                        origin,
                        &format!("Bustup Expression {}", name),
                    );

                    let mouth_images = expression
                        .mouth_chunks
                        .into_iter()
                        .enumerate()
                        .map(|(i, chunk)| {
                            chunk_to_gpu_image(
                                chunk,
                                origin,
                                &format!("Bustup Expression {} Mouth {}", name, i),
                            )
                        })
                        .collect();

                    (
                        name,
                        BustupExpression {
                            face_picture: image,
                            mouth_pictures: mouth_images,
                        },
                    )
                })
                .collect(),
        })
    }
}
