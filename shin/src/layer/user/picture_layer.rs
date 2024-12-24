use std::{fmt::Debug, sync::Arc};

use glam::{Mat4, Vec3, Vec4};
use shin_core::primitives::color::FloatColor4;
use shin_render::{
    render_pass::RenderPass,
    shaders::types::{
        buffer::VertexSource,
        texture::{DepthStencilTarget, TextureTarget},
    },
    ColorBlendType, DrawPrimitive, LayerBlendType, LayerFragmentShader, LayerShaderOutputKind,
    PassKind, RenderProgramWithArguments, RenderRequestBuilder,
};

use crate::{
    asset::picture::{GpuPictureBlock, Picture},
    layer::{
        new_drawable_layer::{NewDrawableLayer, NewDrawableLayerNeedsSeparatePass},
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
        LayerProperties, NewDrawableLayerWrapper, PreRenderContext,
    },
    update::{AdvUpdatable, AdvUpdateContext},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PictureBlockPassKind {
    // 0
    OpaqueOnly,
    // 2
    TransparentOnly,
    // 1 or 3
    OpaqueAndTransparent,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PictureBlockParams {
    pub pass_kind: PictureBlockPassKind,
    pub color_multiplier: FloatColor4,
    pub blend_type: LayerBlendType,
    pub fragment_shader: LayerFragmentShader,
    pub fragment_shader_param: Vec4,
}

impl PictureBlockParams {
    pub fn setup(gfx_pass_kind: PassKind, drawable_params: &DrawableParams) -> Option<Self> {
        let can_render_in_two_passes;

        if drawable_params.blend_type == LayerBlendType::Type1 {
            let everything_needs_blending = drawable_params.color_multiplier.a < 1.0;
            can_render_in_two_passes = !everything_needs_blending;

            if gfx_pass_kind == PassKind::Opaque && everything_needs_blending {
                return None;
            }
        } else {
            can_render_in_two_passes = false;
            if gfx_pass_kind == PassKind::Opaque {
                return None;
            }
        }

        let mut pass_kind = match gfx_pass_kind {
            PassKind::Opaque => PictureBlockPassKind::OpaqueOnly,
            PassKind::Transparent => PictureBlockPassKind::TransparentOnly,
        };

        let color_multiplier = drawable_params.color_multiplier;
        let blend_type = drawable_params.blend_type;
        let shader_param = drawable_params.shader_param;

        let fragment_shader = drawable_params.fragment_shader.simplify(shader_param);

        if !can_render_in_two_passes && gfx_pass_kind == PassKind::Transparent {
            pass_kind = PictureBlockPassKind::OpaqueAndTransparent;
        }

        Some(Self {
            pass_kind,
            color_multiplier,
            blend_type,
            fragment_shader,
            fragment_shader_param: shader_param,
        })
    }
}

fn cull_block(_block: &GpuPictureBlock, _transform: Mat4) -> bool {
    // TODO: implement picture block culling
    true
}

// TODO: make this a method of GpuPictureBlock?
pub fn render_block(
    block: &GpuPictureBlock,
    pass: &mut RenderPass,
    builder: RenderRequestBuilder,
    PictureBlockParams {
        pass_kind,
        color_multiplier,
        blend_type,
        fragment_shader,
        fragment_shader_param,
    }: PictureBlockParams,
    transform: Mat4,
) {
    let (offset, count) = match pass_kind {
        PictureBlockPassKind::OpaqueOnly => (0, block.opaque_rect_count),
        PictureBlockPassKind::TransparentOnly => {
            (block.opaque_rect_count, block.transparent_rect_count)
        }
        PictureBlockPassKind::OpaqueAndTransparent => {
            (0, block.opaque_rect_count + block.transparent_rect_count)
        }
    };

    if count == 0 {
        return;
    }

    if !cull_block(block, transform) {
        return;
    }

    // NOTE: we don't need to slice the vertex buffer, only the index buffer
    let vertices = block.vertex_buffer.as_buffer_ref();
    let indices = block.index_buffer.as_sliced_buffer_ref(
        offset * GpuPictureBlock::INDICES_PER_RECT,
        count * GpuPictureBlock::INDICES_PER_RECT,
    );
    let vertices = VertexSource::VertexAndIndexBuffer { vertices, indices };

    let color_blend_type = match pass_kind {
        PictureBlockPassKind::OpaqueOnly => ColorBlendType::Opaque,
        PictureBlockPassKind::TransparentOnly | PictureBlockPassKind::OpaqueAndTransparent => {
            ColorBlendType::from_premultiplied_layer(blend_type)
        }
    };

    pass.run(builder.color_blend_type(color_blend_type).build(
        RenderProgramWithArguments::Layer {
            output_kind: LayerShaderOutputKind::LayerPremultiply,
            fragment_shader,
            vertices,
            texture: block.texture.as_source(),
            transform,
            color_multiplier,
            fragment_shader_param,
        },
        DrawPrimitive::Triangles,
    ));
}

#[derive(Clone)]
pub struct PictureLayerImpl {
    picture: Arc<Picture>,
    label: String,
}

impl PictureLayerImpl {
    pub fn new(picture: Arc<Picture>, picture_name: Option<String>) -> Self {
        Self {
            picture,
            label: picture_name.unwrap_or_else(|| "unnamed".to_string()),
        }
    }

    fn render_blocks(
        &self,
        pass: &mut RenderPass,
        builder: RenderRequestBuilder,
        params: PictureBlockParams,
        transform: Mat4,
    ) {
        let translation = Mat4::from_translation(Vec3::new(
            -self.picture.origin_x as f32,
            -self.picture.origin_y as f32,
            0.0,
        ));
        // NOTE: this scale is combination of a scale from the header (anything besides 1.0 is rejected by shin-core rn) and ¿device-specific? scale (which is 1.0 on switch)
        let scale = Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0));

        let transform = transform * scale * translation;

        pass.push_debug(&format!(
            "PictureLayer[{}]/{}",
            &self.label,
            match params.pass_kind {
                PictureBlockPassKind::OpaqueOnly => "opaque",
                PictureBlockPassKind::TransparentOnly => "transparent",
                PictureBlockPassKind::OpaqueAndTransparent => "opaque_and_transparent",
            }
        ));
        for (&offset, (positions, block)) in &self.picture.blocks {
            pass.push_debug(&format!("Block[{}]", offset));
            for position in positions {
                let transform = transform * Mat4::from_translation(position.extend(0.0));
                render_block(block, pass, builder, params, transform);
            }
            pass.pop_debug();
        }
        pass.pop_debug();
    }
}

pub type PictureLayer = NewDrawableLayerWrapper<PictureLayerImpl>;

impl PictureLayer {
    pub fn new(picture: Arc<Picture>, picture_name: Option<String>) -> Self {
        Self::from_inner(PictureLayerImpl::new(picture, picture_name))
    }
}

impl NewDrawableLayerNeedsSeparatePass for PictureLayerImpl {}

impl NewDrawableLayer for PictureLayerImpl {
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

        self.render_blocks(pass, builder, params, transform);
    }
}

impl AdvUpdatable for PictureLayerImpl {
    fn update(&mut self, _ctx: &AdvUpdateContext) {}
}

impl Debug for PictureLayerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PictureLayer").field(&self.label).finish()
    }
}
