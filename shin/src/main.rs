mod asset;
// mod camera;
mod adv;
mod game_data;
mod interpolator;
mod layer;
mod render;
mod update;

fn main() {
    // old_main()
    pollster::block_on(render::run());
}
