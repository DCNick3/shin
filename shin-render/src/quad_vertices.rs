use glam::{Mat4, Vec2, vec2, vec3};
use shin_core::primitives::color::UnormColor;
use shin_render_shader_types::{
    buffer::VertexSource, texture::TextureSource, vertices::PosColTexVertex,
};

use crate::{
    DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder, render_pass::RenderPass,
};

/// A struct that stores a quad and allows to more easily dispatch various rendering commands, generating the correct vertex data under the hood.
#[derive(Copy, Clone)]
pub struct QuadVertices {
    vertices: [PosColTexVertex; 4],
}

const TOP_LEFT: usize = 0;
const TOP_RIGHT: usize = 1;
const BOTTOM_LEFT: usize = 2;
const BOTTOM_RIGHT: usize = 3;

impl QuadVertices {
    pub fn new() -> Self {
        Self {
            vertices: [
                // top-left
                PosColTexVertex {
                    position: vec3(0.0, 0.0, 0.0),
                    color: UnormColor::WHITE,
                    texture_position: vec2(0.0, 0.0),
                },
                // top-right
                PosColTexVertex {
                    position: vec3(100.0, 0.0, 0.0),
                    color: UnormColor::WHITE,
                    texture_position: vec2(1.0, 0.0),
                },
                // bottom-left
                PosColTexVertex {
                    position: vec3(0.0, 100.0, 0.0),
                    color: UnormColor::WHITE,
                    texture_position: vec2(0.0, 1.0),
                },
                // bottom-right
                PosColTexVertex {
                    position: vec3(100.0, 100.0, 0.0),
                    color: UnormColor::WHITE,
                    texture_position: vec2(1.0, 1.0),
                },
            ],
        }
    }

    pub fn with_box(mut self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        self.vertices[TOP_LEFT].position.x = left;
        self.vertices[TOP_LEFT].position.y = top;
        self.vertices[TOP_RIGHT].position.x = right;
        self.vertices[TOP_RIGHT].position.y = top;
        self.vertices[BOTTOM_LEFT].position.x = left;
        self.vertices[BOTTOM_LEFT].position.y = bottom;
        self.vertices[BOTTOM_RIGHT].position.x = right;
        self.vertices[BOTTOM_RIGHT].position.y = bottom;

        self
    }

    pub fn with_tex_box(mut self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        self.vertices[TOP_LEFT].texture_position.x = left;
        self.vertices[TOP_LEFT].texture_position.y = top;
        self.vertices[TOP_RIGHT].texture_position.x = right;
        self.vertices[TOP_RIGHT].texture_position.y = top;
        self.vertices[BOTTOM_LEFT].texture_position.x = left;
        self.vertices[BOTTOM_LEFT].texture_position.y = bottom;
        self.vertices[BOTTOM_RIGHT].texture_position.x = right;
        self.vertices[BOTTOM_RIGHT].texture_position.y = bottom;

        self
    }

    pub fn with_color(mut self, color: UnormColor) -> Self {
        self.vertices[TOP_LEFT].color = color;
        self.vertices[TOP_RIGHT].color = color;
        self.vertices[BOTTOM_LEFT].color = color;
        self.vertices[BOTTOM_RIGHT].color = color;

        self
    }

    pub fn render_sprite(
        &self,
        pass: &mut RenderPass,
        builder: RenderRequestBuilder,
        sprite: TextureSource,
        transform: Mat4,
    ) {
        pass.run(builder.build(
            RenderProgramWithArguments::Sprite {
                vertices: VertexSource::VertexData {
                    vertices: &self.vertices,
                },
                sprite,
                transform,
            },
            DrawPrimitive::TrianglesStrip,
        ));
    }

    pub fn fill(&self, pass: &mut RenderPass, builder: RenderRequestBuilder, transform: Mat4) {
        pass.run(builder.build(
            RenderProgramWithArguments::Fill {
                vertices: VertexSource::VertexData {
                    vertices: &self.vertices.map(PosColTexVertex::pos_col),
                },
                transform,
            },
            DrawPrimitive::TrianglesStrip,
        ))
    }
}

#[inline]
pub fn build_quad_vertices<T, F>(f: F) -> [T; 4]
where
    F: Fn(Vec2) -> T,
{
    [
        f(vec2(0.0, 0.0)),
        f(vec2(1.0, 0.0)),
        f(vec2(0.0, 1.0)),
        f(vec2(1.0, 1.0)),
    ]
}
