use std::{fmt::Debug, sync::Arc};

use glam::{Mat4, Vec3, Vec4};
use shin_render::{
    render_pass::RenderPass,
    shaders::types::buffer::{BytesAddress, VertexSource},
    ColorBlendType, DrawPrimitive, LayerBlendType, LayerFragmentShader, LayerShaderOutputKind,
    PassKind, RenderProgramWithArguments, RenderRequestBuilder,
};

use crate::{
    asset::picture::{GpuPictureBlock, Picture},
    layer::{
        new_drawable_layer::{DrawableLayer, NewDrawableLayer},
        properties::LayerProperties,
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
        Layer,
    },
    update::{Updatable, UpdateContext},
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
    pass_kind: PictureBlockPassKind,
    color_multiplier: Vec4,
    blend_type: LayerBlendType,
    fragment_shader: LayerFragmentShader,
    fragment_shader_param: Vec4,
}

impl PictureBlockParams {
    pub fn setup(gfx_pass_kind: PassKind, drawable_params: &DrawableParams) -> Option<Self> {
        let mut can_render_in_two_passes;

        if drawable_params.blend_type == LayerBlendType::Type1 {
            let everything_needs_blending = drawable_params.color_multiplier.w < 1.0;
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

        // downgrade no-op shader operations to default
        let fragment_shader = match drawable_params.fragment_shader {
            LayerFragmentShader::Default => LayerFragmentShader::Default,
            LayerFragmentShader::Mono => {
                if shader_param == Vec4::new(1.0, 1.0, 1.0, 0.0) {
                    LayerFragmentShader::Default
                } else {
                    LayerFragmentShader::Mono
                }
            }
            LayerFragmentShader::Fill => {
                if shader_param.w == 0.0 {
                    LayerFragmentShader::Default
                } else {
                    LayerFragmentShader::Fill
                }
            }
            LayerFragmentShader::Fill2 => {
                if shader_param.truncate() == Vec3::ZERO {
                    LayerFragmentShader::Default
                } else {
                    LayerFragmentShader::Fill2
                }
            }
            LayerFragmentShader::Negative => LayerFragmentShader::Negative,
            LayerFragmentShader::Gamma => {
                if shader_param.truncate() == Vec3::new(1.0, 1.0, 1.0) {
                    LayerFragmentShader::Default
                } else {
                    LayerFragmentShader::Gamma
                }
            }
        };

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

fn cull_block(block: &GpuPictureBlock, transform: Mat4) -> bool {
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

    let vertices = block.vertex_buffer.as_sliced_buffer_ref(
        BytesAddress::from_usize(offset * GpuPictureBlock::VERTICES_PER_RECT),
        BytesAddress::from_usize(count * GpuPictureBlock::VERTICES_PER_RECT),
    );
    let indices = block.index_buffer.as_sliced_buffer_ref(
        BytesAddress::from_usize(offset * GpuPictureBlock::INDICES_PER_RECT),
        BytesAddress::from_usize(count * GpuPictureBlock::INDICES_PER_RECT),
    );
    let vertices = VertexSource::VertexAndIndexBuffer {
        vertex_buffer: vertices,
        index_buffer: indices,
    };

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

pub struct PictureLayer {
    picture: Arc<Picture>,
    picture_name: Option<String>,

    props: LayerProperties,
}

impl PictureLayer {
    pub fn new(picture: Arc<Picture>, picture_name: Option<String>) -> Self {
        Self {
            picture,
            picture_name,
            props: LayerProperties::new(),
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

        pass.push_debug(&self.picture_name.as_ref().map_or("unnamed", |v| v.as_str()));
        for (&offset, &(ref positions, ref block)) in &self.picture.blocks {
            pass.push_debug(&format!("block {}", offset));
            for position in positions {
                let transform = transform * Mat4::from_translation(position.extend(0.0));
                render_block(block, pass, builder, params, transform);
            }
            pass.pop_debug();
        }
        pass.pop_debug();
    }
}

impl DrawableLayer for PictureLayer {
    fn get_properties(&self) -> &LayerProperties {
        &self.props
    }
}

impl NewDrawableLayer for PictureLayer {
    fn needs_separate_pass(&self) -> bool {
        false
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

impl Updatable for PictureLayer {
    fn update(&mut self, ctx: &UpdateContext) {
        self.props.update(ctx);
    }
}

impl Debug for PictureLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PictureLayer")
            .field(
                &self
                    .picture_name
                    .as_ref()
                    .map_or("<unnamed>", |v| v.as_str()),
            )
            .finish()
    }
}

impl Layer for PictureLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
