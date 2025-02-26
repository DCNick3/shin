use glam::Vec4;
use shin_core::primitives::color::FloatColor4;
use shin_render::{
    ColorBlendType, DrawPrimitive, LayerFragmentShader, LayerShaderOutputKind,
    RenderProgramWithArguments, RenderRequestBuilder,
    quad_vertices::build_quad_vertices,
    render_texture::RenderTexture,
    shaders::types::{buffer::VertexSource, vertices::PosTexVertex},
};

use crate::{
    layer::LayerProperties,
    render::{PreRenderContext, VIRTUAL_CANVAS_SIZE_VEC, centered_projection_matrix},
};

pub fn apply_ghosting(
    context: &mut PreRenderContext,
    props: &LayerProperties,
    render_texture_src: &mut RenderTexture,
    render_texture_prev_frame: &RenderTexture,
    alpha: f32,
) {
    let mut pass = context.begin_pass(
        render_texture_src.as_texture_target(),
        None,
        "NewDrawableLayer/ghosting",
    );

    pass.run(
        RenderRequestBuilder::new()
            .color_blend_type(ColorBlendType::LayerPremultiplied1)
            .build(
                RenderProgramWithArguments::Layer {
                    output_kind: LayerShaderOutputKind::Layer,
                    fragment_shader: LayerFragmentShader::Default,
                    vertices: VertexSource::VertexData {
                        vertices: &build_quad_vertices(|t| PosTexVertex {
                            position: ((t * 2.0) - 1.0) * VIRTUAL_CANVAS_SIZE_VEC / 2.0,
                            texture_position: t,
                        }),
                    },
                    texture: render_texture_prev_frame.as_texture_source(),
                    transform: centered_projection_matrix() * props.get_ghosting_transform(),
                    color_multiplier: FloatColor4::from_rgba(1.0, 1.0, 1.0, alpha).premultiply(),
                    fragment_shader_param: Vec4::ZERO,
                },
                DrawPrimitive::TrianglesStrip,
            ),
    );
}
