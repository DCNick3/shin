use bitvec::bitbox;
use image::RgbaImage;
use indexmap::IndexMap;

use crate::format::{
    bustup::{BustupBlockPromise, BustupBlockPromiseToken, BustupBuilder, BustupSkeleton},
    picture::PicBlock,
};

pub struct DefaultBustupBuilder;

impl BustupBuilder for DefaultBustupBuilder {
    type Args = ();
    type Skeleton<'a> = BustupSkeleton<'a>;
    type BlockType = PicBlock;
    type Output = Bustup;

    fn new<'a>(_args: &Self::Args, skeleton: BustupSkeleton<'a>) -> Self::Skeleton<'a> {
        skeleton
    }

    fn new_block(_args: &Self::Args, mut block: PicBlock) -> anyhow::Result<Self::BlockType> {
        cleanup_unused_areas(&mut block);
        Ok(block)
    }

    fn build(
        skeleton: Self::Skeleton<'_>,
        token: BustupBlockPromiseToken<PicBlock>,
    ) -> anyhow::Result<Self::Output> {
        let mut base_image = RgbaImage::new(
            skeleton.effective_width as u32,
            skeleton.effective_height as u32,
        );

        let mut lower_block = |block: BustupBlockPromise| block.get(&token).clone();

        for block in skeleton.base_blocks {
            let block = lower_block(block);

            image::imageops::overlay(
                &mut base_image,
                &block.data,
                block.offset_x as i64,
                block.offset_y as i64,
            );
        }

        let mut expressions = IndexMap::new();
        for (name, expression) in skeleton.expressions {
            let face1 = expression.face1.map(&mut lower_block);
            let face2 = expression.face2.map(&mut lower_block);
            let mouths = expression
                .mouth_blocks
                .into_iter()
                .map(&mut lower_block)
                .collect();
            let eyes = expression
                .eye_blocks
                .into_iter()
                .map(&mut lower_block)
                .collect();

            expressions.insert(
                name,
                BustupExpression {
                    face1,
                    face2,
                    mouths,
                    eyes,
                },
            );
        }

        Ok(Bustup {
            base_image,
            origin: (skeleton.origin_x, skeleton.origin_y),
            bustup_id: skeleton.bustup_id,
            expressions,
        })
    }
}

pub struct Bustup {
    pub base_image: RgbaImage,
    pub origin: (i16, i16),
    pub bustup_id: u32,
    pub expressions: IndexMap<String, BustupExpression>,
}

pub struct BustupExpression {
    pub face1: Option<PicBlock>,
    pub face2: Option<PicBlock>,
    pub mouths: Vec<PicBlock>,
    pub eyes: Vec<PicBlock>,
}

fn cleanup_unused_areas(block: &mut PicBlock) {
    let mut bitbox = bitbox![0u32; block.data.width() as usize * block.data.height() as usize];
    let coord_to_index = |x: u32, y: u32| (y * block.data.width() + x) as usize;
    for vertex in block
        .opaque_rects
        .iter()
        .chain(block.transparent_rects.iter())
    {
        let clamp_y = |y: u16| std::cmp::min(y, block.data.height() as u16 - 1);
        let clamp_x = |x: u16| std::cmp::min(x, block.data.width() as u16 - 1);
        for y in vertex.from_y.saturating_sub(0)..clamp_y(vertex.to_y) {
            for x in vertex.from_x.saturating_sub(0)..clamp_x(vertex.to_x) {
                bitbox.set(coord_to_index(x as u32, y as u32), true);
            }
        }
    }

    for (pixel, mask) in block.data.pixels_mut().zip(bitbox) {
        if !mask {
            *pixel = image::Rgba([0, 0, 0, 0]);
        }
    }
}
