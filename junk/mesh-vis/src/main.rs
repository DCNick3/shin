mod descriptors;
mod modes;
mod my_combo_box;
mod parse;

use iced::{
    font, keyboard, widget,
    widget::{text, text_input, Column, Space},
    Element, Task,
};
use iced_core::{alignment::Horizontal, Length, Size};
use iced_widget::{
    // combo_box as my_combo_box,
    row,
};
use parse::{modes::MeshModeParseState, HexParseState};

use crate::{
    descriptors::{
        VertexDescriptor, VertexFieldDescriptor, VertexFieldType, VertexFieldValue,
        VertexPrimitiveValue, VERTEX_DESCRTIPTORS,
    },
    modes::Mode,
    parse::{matrix::DecompositionOrder, modes::MatrixModeParseState},
};

struct MeshVisualizerState {
    main_hex_text: String,
    selected_mode: Option<Mode>,
    mode_selector: my_combo_box::State<Mode>,
    // mesh mode
    index_text: String,
    selected_vertex_type: Option<VertexDescriptor>,
    vertex_selector: my_combo_box::State<VertexDescriptor>,
    // matrix mode
    selected_decomposition_order: DecompositionOrder,
    decomposition_order_selector: my_combo_box::State<DecompositionOrder>,
}

impl Default for MeshVisualizerState {
    fn default() -> Self {
        Self {
            main_hex_text: "".to_string(),
            selected_mode: None,
            mode_selector: my_combo_box::State::new(vec![Mode::Mesh, Mode::Matrix]),
            index_text: "".to_string(),
            selected_vertex_type: None,
            vertex_selector: my_combo_box::State::new(VERTEX_DESCRTIPTORS.to_vec()),
            selected_decomposition_order: DecompositionOrder::default(),
            decomposition_order_selector: my_combo_box::State::new(vec![
                DecompositionOrder::RST,
                DecompositionOrder::TRS,
            ]),
        }
    }
}

struct VertexView {
    field_values: Vec<VertexFieldValue>,
}

impl VertexView {
    pub fn new(raw_floats: &[[u8; 4]], desc: &[VertexFieldDescriptor]) -> Self {
        let mut field_values = Vec::new();

        let mut iter = raw_floats.iter().copied();

        macro_rules! f {
            () => {
                f32::from_le_bytes(iter.next().unwrap())
            };
        }

        for field in desc {
            let val = match field.ty {
                VertexFieldType::Float => VertexFieldValue::Float(f!()),
                VertexFieldType::Vector2 => VertexFieldValue::Vector2([f!(), f!()]),
                VertexFieldType::Vector3 => VertexFieldValue::Vector3([f!(), f!(), f!()]),
                VertexFieldType::Vector4 => VertexFieldValue::Vector4([f!(), f!(), f!(), f!()]),
                VertexFieldType::UintColor => {
                    VertexFieldValue::UintColor(u32::from_le_bytes(iter.next().unwrap()))
                }
            };

            field_values.push(val);
        }

        VertexView { field_values }
    }

    pub fn primitive_values(&self) -> impl Iterator<Item = VertexPrimitiveValue> + '_ {
        self.field_values.iter().copied().flatten()
    }

    pub fn view<Message: 'static>(
        &self,
        _desc: &[VertexFieldDescriptor],
    ) -> iced_aw::GridRow<'static, Message> {
        let mut columns = vec![];

        // TODO: visually show which floats belong to the same field
        // TODO: visualize color fields
        for &field_value in &self.field_values {
            for (i, v) in (0..).zip(field_value) {
                if i == 0 {
                    // todo!()
                }

                let r = row([])
                    // .push_maybe((i == 0).then(|| {
                    //     todo!();
                    //     todo!()
                    // }))
                    .push(
                        text(format!("{}", v))
                            .font(font::Font::MONOSPACE)
                            .align_x(Horizontal::Right)
                            .width(Length::Shrink),
                    );

                columns.push(r)
            }
        }

        iced_aw::GridRow::with_elements(columns)
    }
}

#[derive(Debug, Clone)]
enum Message {
    SwitchFocus { backwards: bool },
    MainHexTextChanged(String),
    ModeChanged(Mode),
    IndexTextChanged(String),
    VertexTypeChanged(VertexDescriptor),
    DecompositionOrderChanged(DecompositionOrder),
}

fn update(state: &mut MeshVisualizerState, message: Message) -> Task<Message> {
    match message {
        Message::SwitchFocus { backwards: shift } => {
            if shift {
                widget::focus_previous()
            } else {
                widget::focus_next()
            }
        }
        Message::MainHexTextChanged(text) => {
            state.main_hex_text = text;
            Task::none()
        }
        Message::ModeChanged(mode) => {
            state.selected_mode = Some(mode);
            // it's annoying that we have to do this ourselves, but, AFAIU, widgets can't initiate tasks my themselves

            widget::focus_next()
        }
        Message::IndexTextChanged(text) => {
            state.index_text = text;
            Task::none()
        }
        Message::VertexTypeChanged(desc) => {
            state.selected_vertex_type = Some(desc);
            // it's annoying that we have to do this ourselves, but, AFAIU, widgets can't initiate tasks my themselves
            widget::focus_next()
        }
        Message::DecompositionOrderChanged(order) => {
            state.selected_decomposition_order = order;
            // it's annoying that we have to do this ourselves, but, AFAIU, widgets can't initiate tasks my themselves
            widget::focus_next()
        }
    }
}

fn view(app_state: &MeshVisualizerState) -> Element<Message> {
    let hex_parse_state = HexParseState::parse(&app_state.main_hex_text);

    let col1 = Column::new();

    let mut col1 = col1.extend([
        text_input("Enter data in hex...", &app_state.main_hex_text)
            .on_input(Message::MainHexTextChanged)
            .on_submit(Message::SwitchFocus { backwards: false })
            .padding(10)
            .into(),
        Space::with_height(10).into(),
        hex_parse_state.view().into(),
        Space::with_height(10).into(),
        my_combo_box::ComboBox::new(
            &app_state.mode_selector,
            "Select mode...",
            app_state.selected_mode.as_ref(),
            Message::ModeChanged,
        )
        .padding(10)
        .into(),
    ]);

    let mut col2 = Column::new();

    match app_state.selected_mode {
        None => {
            col2 = col2.push(
                text("Select a mode..."), // .padding(25)
            );
            // cols = cols.push(Space::with_width(10));
        }
        Some(Mode::Mesh) => {
            let state = MeshModeParseState::parse(&hex_parse_state, app_state.selected_vertex_type);

            // TODO: this might be a little too much code for a single function (and it's deeply nested too...)
            col1 = col1.extend([
                Space::with_height(10).into(),
                state.float_parse.view().into(),
                // ---
                Space::with_height(20).into(),
                // ---
                my_combo_box::ComboBox::new(
                    &app_state.vertex_selector,
                    "Select vertex type...",
                    app_state.selected_vertex_type.as_ref(),
                    Message::VertexTypeChanged,
                )
                .into(),
                state.vertex_parse.map_or_else(
                    || Column::with_children([]).into(),
                    |state| state.view().into(),
                ),
            ]);

            col2 =
                col2.extend([
                    text_input("Enter index buffer as array...", &app_state.index_text)
                        .on_input(Message::IndexTextChanged)
                        .padding(10)
                        .into(),
                ]);
        }
        Some(Mode::Matrix) => {
            let state = MatrixModeParseState::parse(
                &hex_parse_state,
                app_state.selected_decomposition_order,
            );

            col1 = col1.extend([
                Space::with_height(10).into(),
                state.float_parse.view().into(),
                // ---
                Space::with_height(20).into(),
                state.matrix_parse.result.view().into(),
                state.matrix_parse.matrix_view().into(),
            ]);

            col2 = col2.extend([
                text("Decomposition:")
                    .font(iced::Font {
                        weight: font::Weight::Bold,
                        ..iced::Font::DEFAULT
                    })
                    .into(),
                Space::with_height(10).into(),
                my_combo_box::ComboBox::new(
                    &app_state.decomposition_order_selector,
                    "Select decomposition order...",
                    Some(&app_state.selected_decomposition_order),
                    Message::DecompositionOrderChanged,
                )
                .into(),
                Space::with_height(20).into(),
                state.matrix_parse.decomposition_view().into(),
            ]);
        }
    }

    let cols = iced_widget::Row::with_children([
        col1.width(Length::FillPortion(1)).padding(25).into(),
        col2.width(Length::FillPortion(1)).padding(25).into(),
    ]);

    let res: Element<_> = cols.padding(25).into();

    res
}

fn main() -> iced::Result {
    iced::application::application("Mesh visualizer", update, view)
        .subscription(|_state| {
            keyboard::on_key_press(|key, modifiers| {
                let keyboard::Key::Named(key) = key else {
                    return None;
                };

                match (key, modifiers) {
                    (keyboard::key::Named::Tab, _) => Some(Message::SwitchFocus {
                        backwards: modifiers.shift(),
                    }),
                    _ => None,
                }
            })
        })
        .window_size(Size::new(1920.0, 1080.0))
        .run()

    // MeshVisualizerState::run(Settings::default())
}
