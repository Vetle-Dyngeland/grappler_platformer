use super::velocity::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Resource, Copy, Clone, Debug, PartialEq, Reflect)]
pub struct KinematicGravity(pub Vec2);

#[derive(Default, Component, Copy, Clone, Debug, PartialEq, Reflect)]
pub struct KinematicGravityUser;

pub fn kinematic_gravity(
    mut query: Query<(&mut KinematicVelocity, Option<&GravityScale>), With<KinematicGravityUser>>,
    gravity: Res<KinematicGravity>,
    time: Res<Time>,
) {
    for (mut vel, scale) in query.iter_mut() {
        let scale = match scale {
            Some(s) => s.0,
            None => 1f32,
        };

        let add = gravity.0 * scale * time.delta_seconds();
        vel.x += add.x;
        vel.y += add.y;
    }
}
