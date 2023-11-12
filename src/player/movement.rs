use std::{collections::HashMap, time::Duration};

use crate::level::GrapplePoint;

use super::{input::InputAction, state_machine::states::*, Player, PlayerSet};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use leafwing_input_manager::prelude::ActionState;

pub mod grappler;
pub mod gravity;
pub mod horizontal_movement;
pub mod jumper;
pub mod slingshot;
pub mod velocity;

use grappler::*;
use gravity::*;
use horizontal_movement::*;
use jumper::*;
use slingshot::*;
use velocity::*;

pub(super) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KinematicGravity(Vec2::NEG_Y * 1000f32))
            .add_systems(Startup, init.in_set(PlayerSet::Visuals))
            .add_systems(
                Update,
                (
                    kinematic_velocity,
                    kinematic_gravity,
                    jumper,
                    horizontal_movement,
                    grappler_movement,
                    slingshot,
                )
                    .in_set(PlayerSet::Movement),
            )
            .register_type::<KinematicVelocity>()
            .register_type::<KinematicGravity>()
            .register_type::<Jumper>()
            .register_type::<HorizontalMovement>()
            .register_type::<KinematicGravityUser>()
            .register_type::<Slingshot>()
            .register_type::<Grappler>();
    }
}

fn init(mut cmd: Commands, player_query: Query<Entity, With<Player>>) {
    cmd.entity(player_query.single()).insert((
        Collider::cuboid(12.5f32, 25f32),
        Friction {
            coefficient: 0f32,
            combine_rule: CoefficientCombineRule::Min,
        },
        Restitution {
            coefficient: 0f32,
            combine_rule: CoefficientCombineRule::Min,
        },
        RigidBody::KinematicPositionBased,
        KinematicVelocity::default(),
        KinematicGravityUser,
        Jumper::new(400f32, 0.35f32, 1.2f32, Vec2::new(1.2f32, 0.8f32), 0.175f32, 0.2f32),
        HorizontalMovement {
            max_speed: 250f32,
            acceleration_time: 0.2f32,
            turn_around_multi: 1.5f32,
            deccelration_time: 0.4f32,
            air_control_multi: 0.7f32,
            air_friction_multi: 0.2f32,
        },
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01f32),
            slide: true,
            max_slope_climb_angle: 50f32.to_radians(),
            min_slope_slide_angle: 20f32.to_radians(),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Relative(0.3f32),
                min_width: CharacterLength::Relative(0.5f32),
                include_dynamic_bodies: false,
            }),
            snap_to_ground: Some(CharacterLength::Relative(0.1f32)),
            apply_impulse_to_dynamic_bodies: true,
            ..Default::default()
        },
        Slingshot::new(
            800f32,
            250f32,
            Vec2::new(0.8f32, 1f32),
            0.5f32,
            0.5f32,
            0.7f32,
        ),
        // Dont really like current grappler, will probably not include it/rework it
        //Grappler::new(300f32, 75f32, 200f32, 0.008f32, 0.2f32, 0.4f32),
    ));
}
