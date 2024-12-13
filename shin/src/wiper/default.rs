use glam::vec4;
use shin_core::time::Ticks;
use shin_render::{
    render_pass::RenderPass,
    shaders::types::{buffer::VertexSource, texture::TextureSource, vertices::LayerVertex},
    shin_orthographic_projection_matrix, ColorBlendType, DrawPrimitive, RenderProgramWithArguments,
    RenderRequestBuilder,
};

use crate::{
    update::{AdvUpdatable, AdvUpdateContext},
    wiper::timed::{TimedWiper, TimedWiperWrapper},
};

#[derive(Clone)]
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
        let transform = shin_orthographic_projection_matrix(0.0, 1.0, 1.0, 0.0, -1.0, 1.0);

        let vertices = &[
            LayerVertex {
                coords: vec4(0.0, 0.0, 0.0, 0.0),
            },
            LayerVertex {
                coords: vec4(1.0, 0.0, 1.0, 0.0),
            },
            LayerVertex {
                coords: vec4(0.0, 1.0, 0.0, 1.0),
            },
            LayerVertex {
                coords: vec4(1.0, 1.0, 1.0, 1.0),
            },
        ];

        pass.run(
            render_request_builder
                .color_blend_type(ColorBlendType::Opaque)
                .build(
                    RenderProgramWithArguments::WiperDefault {
                        vertices: VertexSource::VertexData { vertices },
                        texture_source,
                        texture_target,
                        transform,
                        alpha: vec4(progress, 0.0, 0.0, 0.0),
                    },
                    DrawPrimitive::TrianglesStrip,
                ),
        );
    }
}

pub type DefaultWiper = TimedWiperWrapper<DefaultWiperImpl>;

impl DefaultWiper {
    #[expect(unused)] // for future stuff
    pub fn new(duration: Ticks) -> Self {
        Self::from_inner(DefaultWiperImpl, duration)
    }
}
