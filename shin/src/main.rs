mod asset;
// mod camera;
mod layer;
// mod vm;
mod interpolator;
mod render;

fn main() {
    // old_main()
    pollster::block_on(render::run());
}
