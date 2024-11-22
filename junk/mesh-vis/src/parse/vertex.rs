use iced::font;
use iced_core::{alignment::Horizontal, color, Length};
use iced_widget::{scrollable, text, Column, Space, Text};

use crate::{descriptors::VertexDescriptor, VertexView};

pub enum VertexParseResult {
    Valid,
    PartiallyValid,
}

impl VertexParseResult {
    pub fn view(&self) -> Text<'static> {
        match self {
            VertexParseResult::Valid => text("Valid vertices").color(color!(0x00c000)),
            VertexParseResult::PartiallyValid => text("Invalid vertices!").color(color!(0xc00000)),
        }
    }
}

pub struct VertexParseState {
    pub descriptor: VertexDescriptor,
    pub vertices: Vec<VertexView>,
    pub result: VertexParseResult,
}

impl VertexParseState {
    pub fn parse(raw_floats: &[[u8; 4]], descriptor: VertexDescriptor) -> Self {
        let vertex_len = descriptor.float_count_per_vertex() as usize;

        let mut valid_prefix_len = raw_floats.len();
        valid_prefix_len -= valid_prefix_len % vertex_len;
        let valid_prefix = &raw_floats[..valid_prefix_len];

        let vertices = valid_prefix
            .chunks_exact(vertex_len)
            .map(|v| VertexView::new(v, &descriptor.fields))
            .collect();

        let result = if valid_prefix_len == raw_floats.len() {
            VertexParseResult::Valid
        } else {
            VertexParseResult::PartiallyValid
        };

        Self {
            descriptor,
            vertices,
            result,
        }
    }

    pub fn view<Message: 'static>(&self) -> Column<'static, Message> {
        Column::with_children([
            // ---
            Space::with_height(10).into(),
            // ---
            self.result.view().into(),
            text(format!("{0} (0x{0:x}) vertices", self.vertices.len())).into(),
            scrollable(
                iced_aw::Grid::with_rows(
                    [iced_aw::GridRow::with_elements(
                        self.descriptor
                            .column_names()
                            .into_iter()
                            .map(|v| {
                                text(v).width(Length::Fill).align_x(Horizontal::Left).font(
                                    iced::Font {
                                        weight: font::Weight::Bold,
                                        ..iced::Font::MONOSPACE
                                    },
                                )
                            })
                            .collect(),
                    )]
                    .into_iter()
                    .chain(self.vertices.iter().map(|v| v.view(self.descriptor.fields)))
                    .collect(),
                )
                // .column_widths(&[
                //     Length::FillPortion(1),
                //     Length::FillPortion(1),
                //     Length::FillPortion(1),
                // ])
                .horizontal_alignment(Horizontal::Right)
                .column_spacing(20)
                .width(Length::Fill), // .padding(20),
            )
            .width(Length::Fill)
            .into(),
        ])
    }
}
