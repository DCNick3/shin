use std::fmt::Debug;

use glam::{Vec4, vec3};
use shin_core::primitives::color::FloatColor4;
use shin_render::{
    ColorBlendType, DrawPrimitive, LayerBlendType, PassKind, RenderProgramWithArguments,
    RenderRequestBuilder,
    render_pass::RenderPass,
    shaders::types::{RenderClone, buffer::VertexSource, vertices::PosColVertex},
};

use crate::{
    layer::{
        NewDrawableLayer, NewDrawableLayerWrapper,
        new_drawable_layer::{NewDrawableLayerFastForward, NewDrawableLayerNeedsSeparatePass},
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
    },
    update::{AdvUpdatable, AdvUpdateContext},
};

#[derive(Clone, RenderClone)]
pub struct TileLayerImpl {
    color: FloatColor4,
    rect: Vec4,
}

impl TileLayerImpl {
    pub fn new(color: FloatColor4, rect: Vec4) -> Self {
        Self { color, rect }
    }
}

pub type TileLayer = NewDrawableLayerWrapper<TileLayerImpl>;

impl TileLayer {
    pub fn new(color: FloatColor4, rect: Vec4) -> Self {
        Self::from_inner(TileLayerImpl::new(color, rect))
    }
}

impl NewDrawableLayerNeedsSeparatePass for TileLayerImpl {}

impl NewDrawableLayer for TileLayerImpl {
    fn render_drawable_direct(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        &DrawableParams {
            color_multiplier,
            blend_type,
            fragment_shader,
            shader_param,
        }: &DrawableParams,
        clip: &DrawableClipParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        let tinted_color = color_multiplier * self.color;
        let fragment_shader = fragment_shader.simplify(shader_param);

        if tinted_color.a <= 0.0 {
            return;
        }

        let target_pass = if blend_type == LayerBlendType::Type1 && tinted_color.a >= 1.0 {
            PassKind::Opaque
        } else {
            PassKind::Transparent
        };

        if pass_kind != target_pass {
            return;
        }

        assert_eq!(
            clip.mode,
            DrawableClipMode::None,
            "Clipping effect is not implemented"
        );

        let blend_type = match pass_kind {
            PassKind::Opaque => ColorBlendType::Opaque,
            PassKind::Transparent => ColorBlendType::from_regular_layer(blend_type),
        };

        let color = fragment_shader
            .evaluate(tinted_color, shader_param)
            .into_unorm();

        let transform = transform.compute_final_transform();

        let left = self.rect.x;
        let right = self.rect.x + self.rect.z;
        let top = self.rect.y;
        let bottom = self.rect.y + self.rect.w;

        let vertices = &[
            PosColVertex {
                position: vec3(left, top, 0.0),
                color,
            },
            PosColVertex {
                position: vec3(right, top, 0.0),
                color,
            },
            PosColVertex {
                position: vec3(left, bottom, 0.0),
                color,
            },
            PosColVertex {
                position: vec3(right, bottom, 0.0),
                color,
            },
        ];

        pass.push_debug("TileLayer");

        pass.run(
            RenderRequestBuilder::new()
                .depth_stencil_shorthand(stencil_ref, false, false)
                .color_blend_type(blend_type)
                .build(
                    RenderProgramWithArguments::Fill {
                        vertices: VertexSource::VertexData { vertices },
                        transform,
                    },
                    DrawPrimitive::TrianglesStrip,
                ),
        );

        pass.pop_debug();
    }
}

impl NewDrawableLayerFastForward for TileLayerImpl {
    fn fast_forward(&mut self) {}
}

impl AdvUpdatable for TileLayerImpl {
    fn update(&mut self, _ctx: &AdvUpdateContext) {}
}

impl Debug for TileLayerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.color.into_array().map(|v| (v * 255.0) as u8);
        let color = format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color[0], color[1], color[2], color[3]
        );

        f.debug_tuple("TileLayer").field(&color).finish()
    }
}
