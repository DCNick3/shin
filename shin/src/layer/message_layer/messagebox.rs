use glam::{vec2, vec3, Mat4, Vec2};
use shin_core::{primitives::color::FloatColor4, vm::command::types::MessageboxType};
use shin_render::{
    render_pass::RenderPass,
    shaders::types::{buffer::VertexSource, vertices::PosColTexVertex},
    shin_orthographic_projection_matrix, ColorBlendType, DrawPrimitive, RenderProgramWithArguments,
    RenderRequestBuilder,
};

use crate::layer::message_layer::{MessageLayer, SlidingOutMessagebox};

#[derive(Debug, Copy, Clone)]
pub struct Messagebox {
    pub ty: MessageboxType,
    pub natural_slide: f32,
    pub height: f32,
}

impl From<SlidingOutMessagebox> for Messagebox {
    fn from(value: SlidingOutMessagebox) -> Self {
        Self {
            ty: value.ty,
            natural_slide: value.slide_out.value(),
            height: value.height,
        }
    }
}

const MESSAGEBOX_TEXTURE_SIZE: Vec2 = vec2(1648.0, 288.0);
macro_rules! make_vertices {
    ($color:expr; $([$x:expr, $y:expr, $x_tex:expr, $y_tex:expr]),*) => {
        &[
            $(
                PosColTexVertex {
                    position: vec3($x, $y, 1.0),
                    color: $color,
                    texture_position: vec2($x_tex / MESSAGEBOX_TEXTURE_SIZE.x, $y_tex / MESSAGEBOX_TEXTURE_SIZE.y),
                }
            ),*
        ] as &[_]
    };
}

impl MessageLayer {
    pub(super) fn render_messagebox(
        &self,
        pass: &mut RenderPass,
        builder: RenderRequestBuilder,
        transform: Mat4,
        messagebox: Messagebox,
    ) {
        if messagebox.natural_slide == 0.0 {
            return;
        }
        let final_slide_progress = messagebox.natural_slide * self.modal_slide.value();

        match messagebox.ty {
            MessageboxType::Neutral | MessageboxType::WitchSpace | MessageboxType::Ushiromiya => {
                // draw a "normal" messagebox at the bottom of the screen
                let texture = match messagebox.ty {
                    MessageboxType::Neutral => &self.messagebox_textures.message_window_1,
                    MessageboxType::WitchSpace => &self.messagebox_textures.message_window_2,
                    MessageboxType::Ushiromiya => &self.messagebox_textures.message_window_3,
                    _ => unreachable!(),
                }
                .as_source();

                // TODO: need settings handle to get v13_msgwinalpha
                let msgwinalpha = 0.85;

                let transform = transform
                    * Mat4::from_translation(vec3(
                        0.0,
                        (1.0 - final_slide_progress) * 96.0 + (1080.0 - messagebox.height) - 32.0,
                        0.0,
                    ));

                // slide progress also affects the messagebox opacity!
                let color =
                    FloatColor4::from_rgba(1.0, 1.0, 1.0, final_slide_progress * msgwinalpha)
                        .into_unorm();

                let char_name_width = self.char_name_width;

                let char_name_vertices = if char_name_width == 0.0 {
                    make_vertices!(color;
                        [130.0, -32.0, 0.0, 144.0],
                        [130.0, 80.0, 0.0, 256.0],
                        [178.0, -32.0, 48.0, 144.0],
                        [178.0, 80.0, 48.0, 256.0],
                        [1742.0, -32.0, 64.0, 144.0],
                        [1742.0, 80.0, 64.0, 256.0],
                        [1790.0, -32.0, 112.0, 144.0],
                        [1790.0, 80.0, 112.0, 256.0]
                    )
                } else {
                    make_vertices!(color;
                        [130.0, -32.0, 0.0, 0.0],
                        [130.0, 80.0, 0.0, 112.0],
                        [178.0, -32.0, 48.0, 0.0],
                        [178.0, 80.0, 48.0, 112.0],
                        [178.0 + char_name_width, -32.0, 64.0, 0.0],
                        [178.0 + char_name_width, 80.0, 64.0, 112.0],
                        [290.0 + char_name_width, -32.0, 160.0, 0.0],
                        [290.0 + char_name_width, 80.0, 160.0, 112.0],
                        [1742.0, -32.0, 176.0, 0.0],
                        [1742.0, 80.0, 176.0, 112.0],
                        [1790.0, -32.0, 224.0, 0.0],
                        [1790.0, 80.0, 224.0, 112.0]
                    )
                };

                let high = self.height.value() + 32.0;
                let mid = high - 256.0;

                let message_vertices = make_vertices!(color;
                    /* 0  */ [130.0, 80.0, 240.0, 16.0],
                    /* 1  */ [178.0, 80.0, 288.0, 16.0],
                    /* 2  */ [446.0, 80.0, 304.0, 16.0],
                    /* 3  */ [1790.0, 80.0, 1648.0, 16.0],
                    /* 4  */ [130.0, mid, 240.0, 32.0],
                    /* 5  */ [178.0, mid, 288.0, 32.0],
                    /* 6  */ [446.0, mid, 304.0, 32.0],
                    /* 7  */ [1790.0, mid, 1648.0, 32.0],
                    /* 8  */ [130.0, high, 240.0, 288.0],
                    /* 9  */ [178.0, high, 288.0, 288.0],
                    /* 10 */ [446.0, high, 304.0, 288.0],
                    /* 11 */ [1790.0, high, 1648.0, 288.0]
                );
                let message_indices = &[0, 4, 1, 5, 2, 6, 3, 7, 11, 6, 10, 5, 9, 4, 8];

                pass.run(builder.color_blend_type(ColorBlendType::Layer1).build(
                    RenderProgramWithArguments::Sprite {
                        vertices: VertexSource::VertexData {
                            vertices: char_name_vertices,
                        },
                        sprite: texture,
                        transform,
                    },
                    DrawPrimitive::TrianglesStrip,
                ));

                pass.run(builder.color_blend_type(ColorBlendType::Layer1).build(
                    RenderProgramWithArguments::Sprite {
                        vertices: VertexSource::VertexAndIndexData {
                            vertices: message_vertices,
                            indices: message_indices,
                        },
                        sprite: texture,
                        transform,
                    },
                    DrawPrimitive::TrianglesStrip,
                ));
            }
            MessageboxType::Novel => {
                // draw a full-screen translucent overlay
                let transform = shin_orthographic_projection_matrix(0.0, 1.0, 1.0, 0.0, -1.0, 1.0);
                todo!()
            }
            MessageboxType::Transparent | MessageboxType::NoText => {
                // nothing!
            }
        }
    }
}
