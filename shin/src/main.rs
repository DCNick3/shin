mod asset;
// mod camera;
// pub mod layer;
// mod vm;
mod render;

fn main() {
    // old_main()
    pollster::block_on(render::run());
}
