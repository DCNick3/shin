use shin_derive::RenderClone;
use shin_render::{PassKind, render_pass::RenderPass};

use crate::{
    layer::{DrawableLayer, Layer, render_params::TransformParams},
    render::PreRenderContext,
    update::{AdvUpdatable, AdvUpdateContext},
};

#[derive(RenderClone)]
pub enum EitherLayer<L, R> {
    Left(#[render_clone(needs_render)] L),
    Right(#[render_clone(needs_render)] R),
}

impl<L: AdvUpdatable, R: AdvUpdatable> AdvUpdatable for EitherLayer<L, R> {
    #[inline]
    fn update(&mut self, context: &AdvUpdateContext) {
        match self {
            EitherLayer::Left(left) => left.update(context),
            EitherLayer::Right(right) => right.update(context),
        }
    }
}

impl<L: Layer, R: Layer> Layer for EitherLayer<L, R> {
    fn fast_forward(&mut self) {
        match self {
            EitherLayer::Left(left) => left.fast_forward(),
            EitherLayer::Right(right) => right.fast_forward(),
        }
    }

    #[inline]
    fn get_stencil_bump(&self) -> u8 {
        match self {
            EitherLayer::Left(left) => left.get_stencil_bump(),
            EitherLayer::Right(right) => right.get_stencil_bump(),
        }
    }

    #[inline]
    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        match self {
            EitherLayer::Left(left) => left.pre_render(context, transform),
            EitherLayer::Right(right) => right.pre_render(context, transform),
        }
    }

    #[inline]
    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        match self {
            EitherLayer::Left(left) => left.render(pass, transform, stencil_ref, pass_kind),
            EitherLayer::Right(right) => right.render(pass, transform, stencil_ref, pass_kind),
        }
    }
}

impl<L: DrawableLayer, R: DrawableLayer> DrawableLayer for EitherLayer<L, R> {
    #[inline]
    fn properties(&self) -> &crate::layer::LayerProperties {
        match self {
            EitherLayer::Left(left) => left.properties(),
            EitherLayer::Right(right) => right.properties(),
        }
    }

    #[inline]
    fn properties_mut(&mut self) -> &mut crate::layer::LayerProperties {
        match self {
            EitherLayer::Left(left) => left.properties_mut(),
            EitherLayer::Right(right) => right.properties_mut(),
        }
    }
}
