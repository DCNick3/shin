extern crate self as shin;

mod asset;
// mod camera;
mod adv;
mod audio;
mod fps_counter;
mod input;
mod interpolator;
mod layer;
mod render;
mod update;
mod window;

fn main() {
    // old_main()
    pollster::block_on(window::run());
}
