mod camera;

use anyhow::Result;
use bevy::prelude::*;
use bevy::render::camera::CameraProjectionPlugin;

// struct

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    trace!("!!! hello world !!!");

    commands.spawn_bundle(camera::Camera2dBundle {
        projection: camera::OrthographicProjection::default(),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("bea.png"),
        transform: Transform::from_scale(Vec3::splat(1.0)),
        ..default()
    });
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
        .add_plugin(CameraProjectionPlugin::<camera::OrthographicProjection>::default())
        .add_startup_system(setup)
        .run();
}
