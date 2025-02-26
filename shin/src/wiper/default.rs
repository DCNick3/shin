use shin_core::time::Ticks;
use shin_render::{
    ColorBlendType, DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder,
    quad_vertices::build_quad_vertices,
    render_pass::RenderPass,
    shaders::types::{buffer::VertexSource, texture::TextureSource, vertices::PosTexVertex},
};

use crate::{
    render::normalized_projection_matrix,
    update::{AdvUpdatable, AdvUpdateContext},
    wiper::timed::{TimedWiper, TimedWiperWrapper},
};

#[derive(Debug, Clone)]
pub struct DefaultWiperImpl;

impl AdvUpdatable for DefaultWiperImpl {
    fn update(&mut self, _context: &AdvUpdateContext) {}
}

impl TimedWiper for DefaultWiperImpl {
    fn render(
        &self,
        pass: &mut RenderPass,
        render_request_builder: RenderRequestBuilder,
        texture_target: TextureSource,
        texture_source: TextureSource,
        progress: f32,
    ) {
        let transform = normalized_projection_matrix();

        let vertices = &build_quad_vertices(|t| PosTexVertex {
            position: t,
            texture_position: t,
        });

        pass.run(
            render_request_builder
                .color_blend_type(ColorBlendType::Opaque)
                .build(
                    RenderProgramWithArguments::WiperDefault {
                        vertices: VertexSource::VertexData { vertices },
                        texture_source,
                        texture_target,
                        transform,
                        alpha: progress,
                    },
                    DrawPrimitive::TrianglesStrip,
                ),
        );
    }
}

pub type DefaultWiper = TimedWiperWrapper<DefaultWiperImpl>;

impl DefaultWiper {
    pub fn new(duration: Ticks) -> Self {
        Self::from_inner(DefaultWiperImpl, duration)
    }
}
