use shin_render::{render_pass::RenderPass, PassKind};

use crate::{
    layer::{render_params::TransformParams, DrawableLayer, Layer, PreRenderContext},
    update::{AdvUpdatable, AdvUpdateContext},
};

#[derive(Clone)]
pub enum EitherLayer<L, R> {
    Left(L),
    Right(R),
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
