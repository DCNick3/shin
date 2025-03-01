use std::{fmt::Debug, sync::Arc};

use glam::{Mat4, Vec3, vec3};
use shin_render::{
    PassKind, RenderRequestBuilder,
    render_pass::RenderPass,
    shaders::types::{
        RenderClone,
        texture::{DepthStencilTarget, TextureTarget},
    },
};

use crate::{
    asset::bustup::Bustup,
    layer::{
        LayerProperties, NewDrawableLayer, NewDrawableLayerWrapper,
        new_drawable_layer::{NewDrawableLayerFastForward, NewDrawableLayerNeedsSeparatePass},
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
        user::picture_layer::{PictureBlockParams, PictureBlockPassKind},
    },
    render::PreRenderContext,
    update::{AdvUpdatable, AdvUpdateContext, Updatable, UpdateContext},
};

#[derive(Clone, RenderClone)]
pub struct BustupLayerImpl {
    bustup: Arc<Bustup>,
    label: String,
    mouth_state: u32,
    eyes_state: u32,
}

impl BustupLayerImpl {
    pub fn new(bustup: Arc<Bustup>, bustup_name: Option<String>) -> Self {
        Self {
            bustup,
            label: bustup_name.unwrap_or_else(|| "unnamed".to_string()),
            mouth_state: 0,
            eyes_state: 0,
        }
    }

    #[tracing::instrument(skip_all)]
    fn render_impl(
        &self,
        pass: &mut RenderPass,
        builder: RenderRequestBuilder,
        params: PictureBlockParams,
        transform: Mat4,
    ) {
        let translation = Mat4::from_translation(vec3(
            -self.bustup.origin_x as f32,
            -self.bustup.origin_y as f32,
            0.0,
        ));
        // NOTE: this scale is a ¿device-specific? scale (which is 1.0 on switch)
        let scale = Mat4::from_scale(Vec3::ONE);

        let transform = transform * scale * translation;

        pass.push_debug(&format!(
            "BustupLayer[{}]/{}",
            &self.label,
            match params.pass_kind {
                PictureBlockPassKind::OpaqueOnly => "opaque",
                PictureBlockPassKind::TransparentOnly => "transparent",
                PictureBlockPassKind::OpaqueAndTransparent => "opaque_and_transparent",
            }
        ));

        pass.push_debug("Base");
        for block in &self.bustup.base_blocks {
            super::picture_layer::render_block(block, pass, builder, params, transform);
        }
        pass.pop_debug();

        if let Some(block) = &self.bustup.face1 {
            pass.push_debug("Face1");
            super::picture_layer::render_block(block, pass, builder, params, transform);
            pass.pop_debug();
        }

        if let Some(block) = &self.bustup.face2 {
            pass.push_debug("Face2");
            super::picture_layer::render_block(block, pass, builder, params, transform);
            pass.pop_debug();
        }

        if !self.bustup.mouth_blocks.is_empty() {
            pass.push_debug("Mouth");
            super::picture_layer::render_block(
                &self.bustup.mouth_blocks[self.mouth_state as usize],
                pass,
                builder,
                params,
                transform,
            );
            pass.pop_debug();
        }

        if !self.bustup.eye_blocks.is_empty() {
            pass.push_debug("Eyes");
            super::picture_layer::render_block(
                &self.bustup.eye_blocks[self.eyes_state as usize],
                pass,
                builder,
                params,
                transform,
            );
            pass.pop_debug();
        }

        pass.pop_debug();
    }
}

pub type BustupLayer = NewDrawableLayerWrapper<BustupLayerImpl>;

impl BustupLayer {
    pub fn new(picture: Arc<Bustup>, picture_name: Option<String>) -> Self {
        Self::from_inner(BustupLayerImpl::new(picture, picture_name))
    }
}

impl NewDrawableLayerNeedsSeparatePass for BustupLayerImpl {}

impl NewDrawableLayer for BustupLayerImpl {
    #[tracing::instrument(skip_all)]
    fn render_drawable_indirect(
        &mut self,
        _context: &mut PreRenderContext,
        _props: &LayerProperties,
        _target: TextureTarget,
        _depth_stencil: DepthStencilTarget,
        _transform: &TransformParams,
    ) -> PassKind {
        todo!()
    }

    #[tracing::instrument(skip_all)]
    fn render_drawable_direct(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        drawable: &DrawableParams,
        clip: &DrawableClipParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        let Some(params) = PictureBlockParams::setup(pass_kind, drawable) else {
            return;
        };

        assert_eq!(
            clip.mode,
            DrawableClipMode::None,
            "Clipping effect is not implemented"
        );

        let builder =
            RenderRequestBuilder::new().depth_stencil_shorthand(stencil_ref, false, false);
        let transform = transform.compute_final_transform();

        self.render_impl(pass, builder, params, transform);
    }
}

impl NewDrawableLayerFastForward for BustupLayerImpl {
    fn fast_forward(&mut self) {}
}

impl AdvUpdatable for BustupLayerImpl {
    fn update(&mut self, _ctx: &AdvUpdateContext) {}
}

impl Debug for BustupLayerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BustupLayer").field(&self.label).finish()
    }
}
