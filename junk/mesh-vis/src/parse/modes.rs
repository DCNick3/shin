use crate::{
    descriptors::VertexDescriptor,
    parse::{
        matrix::{DecompositionOrder, MatrixParseState},
        vertex::VertexParseState,
        FloatParseState, HexParseState,
    },
};

pub struct MeshModeParseState {
    pub float_parse: FloatParseState,
    pub vertex_parse: Option<VertexParseState>,
}

impl MeshModeParseState {
    pub fn parse(hex_parse: &HexParseState, vertex_descriptor: Option<VertexDescriptor>) -> Self {
        let float_parse = FloatParseState::parse(&hex_parse.bytes);
        let vertex_parse =
            vertex_descriptor.map(|desc| VertexParseState::parse(&float_parse.raw_floats, desc));

        Self {
            float_parse,
            vertex_parse,
        }
    }
}

pub struct MatrixModeParseState {
    pub float_parse: FloatParseState,
    pub matrix_parse: MatrixParseState,
}

impl MatrixModeParseState {
    pub fn parse(hex_parse: &HexParseState, decomposition_order: DecompositionOrder) -> Self {
        let float_parse = FloatParseState::parse(&hex_parse.bytes);
        let matrix_parse = MatrixParseState::parse(&float_parse.floats, decomposition_order);

        Self {
            float_parse,
            matrix_parse,
        }
    }
}
