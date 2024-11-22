mod my_combo_box;

use std::fmt::Display;

use iced::{
    color, executor, font, keyboard, widget,
    widget::{column, text, text_input, vertical_space, Column, Space, Text},
    Application, Element, Settings, Subscription, Task, Theme,
};
use iced_core::{alignment::Horizontal, Length};
use iced_widget::{
    // combo_box as my_combo_box,
    container,
    row,
    scrollable,
};

#[derive(Debug, Clone, Copy)]
enum FieldType {
    Float,
    Vector2,
    Vector3,
    Vector4,
    UintColor,
}

impl FieldType {
    pub fn float_count(&self) -> u32 {
        match self {
            FieldType::Float => 1,
            FieldType::Vector2 => 2,
            FieldType::Vector3 => 3,
            FieldType::Vector4 => 4,
            FieldType::UintColor => 1,
        }
    }
}

struct FieldValueIntoIter {
    array: [PrimitiveValue; 4],
    pos: usize,
}

impl Iterator for FieldValueIntoIter {
    type Item = PrimitiveValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.array.len() {
            None
        } else {
            let result = self.array[self.pos];
            self.pos += 1;

            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.array.len() - self.pos;
        (len, Some(len))
    }
}

#[derive(Debug, Clone, Copy)]
enum PrimitiveValue {
    Float(f32),
    UintColor(u32),
}

impl Display for PrimitiveValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveValue::Float(v) => v.fmt(f),
            PrimitiveValue::UintColor(v) => write!(f, "{:08x}", v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FieldValue {
    Float(f32),
    Vector2([f32; 2]),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    UintColor(u32),
}

impl IntoIterator for FieldValue {
    type Item = PrimitiveValue;
    type IntoIter = FieldValueIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let (array, pos) = match self {
            FieldValue::Float(s) => (
                [
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::Float(s),
                ],
                3,
            ),
            FieldValue::Vector2(a) => (
                [
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::Float(a[0]),
                    PrimitiveValue::Float(a[1]),
                ],
                2,
            ),
            FieldValue::Vector3(a) => (
                [
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::Float(a[0]),
                    PrimitiveValue::Float(a[1]),
                    PrimitiveValue::Float(a[2]),
                ],
                1,
            ),
            FieldValue::Vector4(a) => (
                [
                    PrimitiveValue::Float(a[0]),
                    PrimitiveValue::Float(a[1]),
                    PrimitiveValue::Float(a[2]),
                    PrimitiveValue::Float(a[3]),
                ],
                0,
            ),
            FieldValue::UintColor(v) => (
                [
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::Float(0.0),
                    PrimitiveValue::UintColor(v),
                ],
                3,
            ),
        };
        FieldValueIntoIter { array, pos }
    }
}

#[derive(Debug, Clone, Copy)]
struct FieldDescriptor {
    name: &'static str,
    ty: FieldType,
}

#[derive(Debug, Clone, Copy)]
struct VertexDescriptor {
    name: &'static str,
    fields: &'static [FieldDescriptor],
    position_visualizations: &'static [&'static str],
}

impl VertexDescriptor {
    pub fn float_count_per_vertex(&self) -> u32 {
        self.fields.iter().map(|fd| fd.ty.float_count()).sum()
    }

    pub fn column_names(&self) -> Vec<String> {
        let mut result = Vec::new();

        for field in self.fields {
            let subfields: &[&str] = match field.ty {
                FieldType::Float | FieldType::UintColor => {
                    result.push(field.name.to_string());
                    continue;
                }
                FieldType::Vector2 => &["x", "y"],
                FieldType::Vector3 => &["x", "y", "z"],
                FieldType::Vector4 => &["x", "y", "z", "w"],
            };

            for subfield in subfields {
                result.push(format!("{}.{}", field.name, subfield));
            }
        }

        result
    }
}

impl Display for VertexDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

const VERTEX_DESCRTIPTORS: &[VertexDescriptor] = &[
    VertexDescriptor {
        name: "Pos3",
        fields: &[FieldDescriptor {
            name: "pos",
            ty: FieldType::Vector3,
        }],
        position_visualizations: &["pos"],
    },
    VertexDescriptor {
        name: "Pos4",
        fields: &[FieldDescriptor {
            name: "pos",
            ty: FieldType::Vector4,
        }],
        position_visualizations: &["pos"],
    },
    VertexDescriptor {
        name: "Sprite (Pos3ColUTex2)",
        fields: &[
            FieldDescriptor {
                name: "pos",
                ty: FieldType::Vector3,
            },
            FieldDescriptor {
                name: "col",
                ty: FieldType::UintColor,
            },
            FieldDescriptor {
                name: "tex",
                ty: FieldType::Vector2,
            },
        ],
        position_visualizations: &["pos", "tex"],
    },
];

struct MeshVisualizerState {
    vertex_text: String,
    index_text: String,
    selected_vertex_type: Option<VertexDescriptor>,
    vertex_selector: my_combo_box::State<VertexDescriptor>,
}

impl Default for MeshVisualizerState {
    fn default() -> Self {
        Self {
            vertex_text: "".to_string(),
            index_text: "".to_string(),
            selected_vertex_type: None,
            vertex_selector: my_combo_box::State::new(VERTEX_DESCRTIPTORS.to_vec()),
        }
    }
}

enum HexParseResult {
    Valid,
    PartiallyValid,
}

struct HexParseState {
    bytes: Vec<u8>,
    result: HexParseResult,
}

impl HexParseState {
    pub fn parse(text: &str) -> Self {
        // find first non-hex character
        let mut valid_hex_prefix_len = text
            .find(|c: char| !c.is_ascii_hexdigit())
            .unwrap_or(text.len());
        if valid_hex_prefix_len % 2 != 0 {
            valid_hex_prefix_len -= 1;
        }
        let valid_hex_prefix = &text[..valid_hex_prefix_len];
        let parsed_bytes = hex::decode(valid_hex_prefix).unwrap();
        let parse_state = if valid_hex_prefix_len == text.len() {
            HexParseResult::Valid
        } else {
            HexParseResult::PartiallyValid
        };

        Self {
            bytes: parsed_bytes,
            result: parse_state,
        }
    }

    pub fn view(&self) -> Column<'static, Message> {
        Column::with_children([
            match self.result {
                HexParseResult::Valid => text("Valid hex").color(color!(0x00c000)),
                HexParseResult::PartiallyValid => text("Invalid hex!").color(color!(0xc00000)),
            }
            .into(),
            text(format!("{0} (0x{0:x}) bytes", self.bytes.len())).into(),
        ])
    }
}

enum FloatParseResult {
    Valid,
    PartiallyValid,
}

struct FloatParseState {
    raw_floats: Vec<[u8; 4]>,
    floats: Vec<f32>,
    result: FloatParseResult,
}

impl FloatParseState {
    pub fn parse(bytes: &[u8]) -> Self {
        const FLOAT_LEN: usize = 4;

        let mut valid_prefix_len = bytes.len();
        valid_prefix_len -= valid_prefix_len % FLOAT_LEN;

        let valid_prefix = &bytes[..valid_prefix_len];

        let raw_floats: Vec<[u8; 4]> = valid_prefix
            .chunks_exact(4)
            .map(|c| c.try_into().unwrap())
            .collect();
        let floats = raw_floats.iter().map(|&c| f32::from_le_bytes(c)).collect();

        let result = if valid_prefix_len == bytes.len() {
            FloatParseResult::Valid
        } else {
            FloatParseResult::PartiallyValid
        };

        Self {
            raw_floats,
            floats,
            result,
        }
    }

    pub fn view(&self) -> Column<'static, Message> {
        Column::with_children([
            match self.result {
                FloatParseResult::Valid => text("Valid floats").color(color!(0x00c000)),
                FloatParseResult::PartiallyValid => text("Invalid floats!").color(color!(0xc00000)),
            }
            .into(),
            text(format!("{0} (0x{0:x}) floats", self.floats.len())).into(),
            text(format!("{:?}", self.floats)).into(),
        ])
    }
}

enum VertexParseResult {
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

struct Vertex {
    field_values: Vec<FieldValue>,
}

impl Vertex {
    pub fn new(raw_floats: &[[u8; 4]], desc: &[FieldDescriptor]) -> Self {
        let mut field_values = Vec::new();

        let mut iter = raw_floats.iter().copied();

        macro_rules! f {
            () => {
                f32::from_le_bytes(iter.next().unwrap())
            };
        }

        for field in desc {
            let val = match field.ty {
                FieldType::Float => FieldValue::Float(f!()),
                FieldType::Vector2 => FieldValue::Vector2([f!(), f!()]),
                FieldType::Vector3 => FieldValue::Vector3([f!(), f!(), f!()]),
                FieldType::Vector4 => FieldValue::Vector4([f!(), f!(), f!(), f!()]),
                FieldType::UintColor => {
                    FieldValue::UintColor(u32::from_le_bytes(iter.next().unwrap()))
                }
            };

            field_values.push(val);
        }

        Vertex { field_values }
    }

    pub fn primitive_values(&self) -> impl Iterator<Item = PrimitiveValue> + '_ {
        self.field_values.iter().copied().flatten()
    }

    pub fn view(&self, _desc: &[FieldDescriptor]) -> iced_aw::GridRow<'static, Message> {
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

struct VertexParseState {
    descriptor: VertexDescriptor,
    vertices: Vec<Vertex>,
    result: VertexParseResult,
}

impl VertexParseState {
    pub fn parse(raw_floats: &[[u8; 4]], descriptor: VertexDescriptor) -> Self {
        let vertex_len = descriptor.float_count_per_vertex() as usize;

        let mut valid_prefix_len = raw_floats.len();
        valid_prefix_len -= valid_prefix_len % vertex_len;
        let valid_prefix = &raw_floats[..valid_prefix_len];

        let vertices = valid_prefix
            .chunks_exact(vertex_len)
            .map(|v| Vertex::new(v, &descriptor.fields))
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
}

struct MeshState {
    hex_parse: HexParseState,
    float_parse: FloatParseState,
    vertex_parse: Option<VertexParseState>,
}

impl MeshState {
    pub fn parse(text: &str, vertex_descriptor: Option<VertexDescriptor>) -> Self {
        let hex_parse = HexParseState::parse(text);
        let float_parse = FloatParseState::parse(&hex_parse.bytes);
        let vertex_parse =
            vertex_descriptor.map(|desc| VertexParseState::parse(&float_parse.raw_floats, desc));

        Self {
            hex_parse,
            float_parse,
            vertex_parse,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    SwitchFocus { backwards: bool },
    VertexTextChanged(String),
    IndexTextChanged(String),
    VertexTypeChanged(VertexDescriptor),
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
        Message::VertexTextChanged(text) => {
            state.vertex_text = text;
            Task::none()
        }
        Message::IndexTextChanged(text) => {
            state.index_text = text;
            Task::none()
        }
        Message::VertexTypeChanged(desc) => {
            state.selected_vertex_type = Some(desc);
            // well....
            widget::focus_next()
        }
    }
}

fn view(app_state: &MeshVisualizerState) -> Element<Message> {
    let state = MeshState::parse(&app_state.vertex_text, app_state.selected_vertex_type);

    // using macro makes RustRover freak out

    let col1 = Column::with_children([
        text_input("Enter vertex buffer in hex...", &app_state.vertex_text)
            .on_input(Message::VertexTextChanged)
            .on_submit(Message::SwitchFocus { backwards: false })
            .padding(10)
            .into(),
        Space::with_height(10).into(),
        state.hex_parse.view().into(),
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
        .padding(10)
        .into(),
        state.vertex_parse.map_or_else(
            || Column::with_children([]).into(),
            |state| {
                Column::with_children([
                    // ---
                    Space::with_height(10).into(),
                    // ---
                    state.result.view().into(),
                    text(format!("{0} (0x{0:x}) vertices", state.vertices.len())).into(),
                    scrollable(
                        iced_aw::Grid::with_rows(
                            [iced_aw::GridRow::with_elements(
                                state
                                    .descriptor
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
                            .chain(
                                state
                                    .vertices
                                    .iter()
                                    .map(|v| v.view(state.descriptor.fields)),
                            )
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
                .into()
            },
        ),
    ])
    .padding(25);

    let col2 = Column::with_children([text_input(
        "Enter index buffer as array...",
        &app_state.index_text,
    )
    .on_input(Message::IndexTextChanged)
    .padding(10)
    .into()])
    .padding(25);

    let res: Element<_> = row!(col1, col2).padding(25).into();

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
        .run()

    // MeshVisualizerState::run(Settings::default())
}
