use std::collections::HashMap;

use bevy::prelude::*;
// TODO: Implement lookahead and smoothing

use super::{Player, PlayerSet};

pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init.in_set(PlayerSet::Camera))
            .add_systems(Update, follow_camera.in_set(PlayerSet::Camera));
    }
}

fn init(mut cmd: Commands, query: Query<Entity, With<Player>>) {
    cmd.spawn((
        Camera2dBundle {
            ..Default::default()
        },
        FollowCamera {
            entity: query.single(),
            lookahead: None,
            smoothing: None,
        },
        Name::from("Camera"),
    ));
}

#[derive(Component, Copy, Clone, Debug)]
pub struct FollowCamera {
    pub entity: Entity,
    pub lookahead: Option<CameraOptions>,
    pub smoothing: Option<CameraOptions>,
}

fn follow_camera(
    mut cam_query: Query<(&mut Transform, &FollowCamera)>,
    entity_query: Query<(Entity, &GlobalTransform), Without<FollowCamera>>,
) {
    let entity_map = entity_query
        .iter()
        .collect::<HashMap<Entity, &GlobalTransform>>();

    for (mut transform, cam) in cam_query.iter_mut() {
        let pos = match entity_map.get(&cam.entity) {
            Some(pos) => pos,
            None => continue,
        };

        transform.translation = pos.translation();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CameraOptions {
    pub amount: f32,
    pub multi: Vec2,
}
