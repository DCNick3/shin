mod asset;
mod camera;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::asset::picture::PicturePlugin;
use bevy::render::camera::CameraProjectionPlugin;

fn add_pillarbox_rects(commands: &mut Commands) {
    let bottom_rect = shapes::Rectangle {
        extents: Vec2::new(1920.0, 9999.0),
        origin: RectangleOrigin::CustomCenter(Vec2::new(0.0, -540.0 - 9999.0 / 2.0)),
    };
    let top_rect = shapes::Rectangle {
        extents: Vec2::new(1920.0, 9999.0),
        origin: RectangleOrigin::CustomCenter(Vec2::new(0.0, 540.0 + 9999.0 / 2.0)),
    };
    let left_rect = shapes::Rectangle {
        extents: Vec2::new(9999.0, 1080.0),
        origin: RectangleOrigin::CustomCenter(Vec2::new(-960.0 - 9999.0 / 2.0, 0.0)),
    };
    let right_rect = shapes::Rectangle {
        extents: Vec2::new(9999.0, 1080.0),
        origin: RectangleOrigin::CustomCenter(Vec2::new(960.0 + 9999.0 / 2.0, 0.0)),
    };

    commands.spawn_bundle(
        GeometryBuilder::new()
            .add(&bottom_rect)
            .add(&top_rect)
            .add(&left_rect)
            .add(&right_rect)
            .build(
                DrawMode::Fill(FillMode::color(Color::rgb(0.0, 0.0, 0.0))),
                Transform::from_xyz(0.0, 0.0, 999.0),
            ),
    );
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    trace!("!!! hello world !!!");

    commands.spawn_bundle(camera::Camera2dBundle {
        projection: camera::OrthographicProjection::default(),
        ..Default::default()
    });
    add_pillarbox_rects(&mut commands);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("ship_p1a.pic"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("bea.png"),
        transform: Transform::from_scale(Vec3::splat(1.0))
            .mul_transform(Transform::from_xyz(0.0, 0.0, 1.0)),
        ..default()
    });

    // commands.spawn_bundle(PictureLayerBundle {
    //     picture_layer: PictureLayer {
    //         picture: asset_server.load("ship_p1a.pic"),
    //     },
    //     transform: Default::default(),
    //     global_transform: Default::default(),
    //     visibility: Default::default(),
    //     computed_visibility: Default::default(),
    // });
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
        .add_plugin(ShapePlugin)
        .add_plugin(PicturePlugin)
        .add_startup_system(setup)
        .run();
}
