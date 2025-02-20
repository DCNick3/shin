use std::{fmt::Debug, sync::Arc};

use glam::vec2;
use shin_core::{
    time::Ticks,
    vm::command::types::{MaskFlags, MaskParam},
};
use shin_render::{
    render_pass::RenderPass,
    shaders::types::{buffer::VertexSource, texture::TextureSource, vertices::MaskVertex},
    shin_orthographic_projection_matrix, ColorBlendType, DrawPrimitive, RenderProgramWithArguments,
    RenderRequestBuilder,
};

use crate::{
    asset::mask::MaskTexture,
    update::{AdvUpdatable, AdvUpdateContext},
    wiper::timed::{TimedWiper, TimedWiperWrapper},
};

#[derive(Clone)]
pub struct MaskWiperImpl {
    mask: Arc<MaskTexture>,
    param2: MaskParam,
    flags: MaskFlags,
}

impl Debug for MaskWiperImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaskWiperImpl")
            .field("mask", &self.mask.label)
            .field("param2", &self.param2)
            .field("flags", &self.flags)
            .finish()
    }
}

impl AdvUpdatable for MaskWiperImpl {
    fn update(&mut self, _context: &AdvUpdateContext) {}
}

impl TimedWiper for MaskWiperImpl {
    fn render(
        &self,
        pass: &mut RenderPass,
        render_request_builder: RenderRequestBuilder,
        texture_target: TextureSource,
        texture_source: TextureSource,
        progress: f32,
    ) {
        let inv_param2 = 1.0 / self.param2.0;
        let mut min = 1.0 - progress * (inv_param2 + 1.0);
        let mut max = min + inv_param2;

        if self.flags.contains(MaskFlags::FLIP_MIN_MAX) {
            std::mem::swap(&mut min, &mut max);
        }

        let screen_size = vec2(1920.0, 1080.0);
        let mask_size = self.mask.texture.size_vec();

        let transform =
            shin_orthographic_projection_matrix(0.0, screen_size.x, screen_size.y, 0.0, -1.0, 1.0);

        let [mut x1, mut y1] = [0.0, 0.0];

        let [mut x2, mut y2] = if self.flags.contains(MaskFlags::SCALE) {
            (screen_size / mask_size).to_array()
        } else {
            [1.0, 1.0]
        };

        if self.flags.contains(MaskFlags::FLIP_X) {
            std::mem::swap(&mut x1, &mut x2);
        }
        if self.flags.contains(MaskFlags::FLIP_Y) {
            std::mem::swap(&mut y1, &mut y2);
        }

        let vertices = &[
            MaskVertex {
                position: vec2(0.0, 0.0),
                texture_position: vec2(0.0, 0.0),
                mask_position: vec2(x1, y1),
            },
            MaskVertex {
                position: vec2(screen_size.x, 0.0),
                texture_position: vec2(1.0, 0.0),
                mask_position: vec2(x2, y1),
            },
            MaskVertex {
                position: vec2(0.0, screen_size.y),
                texture_position: vec2(0.0, 1.0),
                mask_position: vec2(x1, y2),
            },
            MaskVertex {
                position: vec2(screen_size.x, screen_size.y),
                texture_position: vec2(1.0, 1.0),
                mask_position: vec2(x2, y2),
            },
        ];

        pass.run(
            render_request_builder
                .color_blend_type(ColorBlendType::Opaque)
                .build(
                    RenderProgramWithArguments::WiperMask {
                        vertices: VertexSource::VertexData { vertices },
                        texture_source,
                        texture_target,
                        texture_mask: self.mask.texture.as_source(),
                        transform,
                        minmax: vec2(min, max),
                    },
                    DrawPrimitive::TrianglesStrip,
                ),
        );
    }
}

pub type MaskWiper = TimedWiperWrapper<MaskWiperImpl>;

impl MaskWiper {
    pub fn new(
        duration: Ticks,
        mask: Arc<MaskTexture>,
        param2: MaskParam,
        flags: MaskFlags,
    ) -> Self {
        Self::from_inner(
            MaskWiperImpl {
                mask,
                param2,
                flags,
            },
            duration,
        )
    }
}
