use dpi::PhysicalSize;

#[derive(Debug)]
pub struct Nv12Frame {
    pub y_plane: Vec<u8>,
    pub uv_plane: Vec<u8>,
    pub size: PhysicalSize<u32>,
}
