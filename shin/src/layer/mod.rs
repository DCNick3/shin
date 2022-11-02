use num_derive::FromPrimitive;

#[derive(FromPrimitive)]
pub enum LayerType {
    Null = 0,
    Tile = 1,
    Picture = 2,
    Bustup = 3,
    Animation = 4,
    Effect = 5,
    Movie = 6,
    FocusLine = 7,
    Rain = 8,
    Quiz = 9,
}

pub struct Layer {}

pub struct LayerLoader {}

pub fn load_layer(_ty: LayerType) -> Layer {
    todo!()
}
