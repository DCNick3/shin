mod asset;
// mod camera;
// pub mod layer;
// mod vm;
mod interpolator;
mod render;

fn main() {
    // old_main()
    pollster::block_on(render::run());
}
