use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub(super) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);
    }
}

fn init(mut cmd: Commands) {
    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb_u8(205, 255, 150),
                custom_size: Some(Vec2::new(500f32, 25f32)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0f32, -50f32, 0f32),
            ..Default::default()
        },
        Collider::cuboid(250f32, 12.75f32),
        Name::from("Platform"),
    ));

    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb_u8(205, 255, 150),
                custom_size: Some(Vec2::new(25f32, 1000f32)),
                ..Default::default()
            },
            transform: Transform::from_xyz(-300f32, 0f32, 0f32),
            ..Default::default()
        },
        Collider::cuboid(12.75f32, 500f32),
        Name::from("Wall"),
    ));

    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb_u8(205, 255, 150),
                custom_size: Some(Vec2::new(25f32, 700f32)),
                ..Default::default()
            },
            transform: Transform::from_xyz(250f32, 150f32, 0f32),
            ..Default::default()
        },
        Collider::cuboid(12.75f32, 350f32),
        Name::from("Wall #2"),
    ));

    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb_u8(255, 75, 125),
                custom_size: Some(Vec2::new(50f32, 50f32)),
                ..Default::default()
            },
            transform: Transform::from_xyz(550f32, 100f32, 0f32),
            ..Default::default()
        },
        GrapplePoint,
        Name::from("Grapple Point"),
    ));

    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb_u8(255, 75, 125),
                custom_size: Some(Vec2::new(50f32, 50f32)),
                ..Default::default()
            },
            transform: Transform::from_xyz(200f32, 300f32, 0f32),
            ..Default::default()
        },
        GrapplePoint,
        Name::from("Grapple Point #2"),
    ));
}

#[derive(Component)]
pub struct GrapplePoint;
