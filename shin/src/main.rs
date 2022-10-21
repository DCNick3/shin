use anyhow::Result;
use bevy::prelude::*;

fn hello_world() {
    trace!("!!! hello world !!!");
    println!("hello world!");
}

fn main() {
    App::new()
        .insert_resource(bevy::render::settings::WgpuSettings {
            // we don't actually need a beefy GPU for this
            power_preference: bevy::render::settings::PowerPreference::LowPower,
            ..Default::default()
        })
        .add_plugins_with(DefaultPlugins, |group| {
            // here we can modify the default plugins
            group
        })
        .add_startup_system(hello_world)
        .run();
}
