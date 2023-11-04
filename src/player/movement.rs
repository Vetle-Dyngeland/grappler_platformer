use std::{collections::HashMap, time::Duration};

use crate::level::GrapplePoint;

use super::{input::InputAction, state_machine::states::*, Player, PlayerSet};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use leafwing_input_manager::prelude::ActionState;
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
                    get_grappler_points,
                    grappling,
                )
                    .in_set(PlayerSet::Movement),
            )
            .register_type::<KinematicVelocity>()
            .register_type::<KinematicGravity>()
            .register_type::<Jumper>()
            .register_type::<HorizontalMovement>()
            .register_type::<KinematicGravityUser>();
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
        Jumper::new(400f32, 0.35f32, 0.175f32, 0.2f32),
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
        Grappler::new(300f32, 20f32, 20f32, 20f32),
    ));
}

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct Grappler {
    pub max_grapple_distance: f32,
    pub min_grapple_distance: f32,
    pub far_impulse: f32,
    pub close_impulse: f32,
    current_point: Option<Entity>,
    closest_grappleable_point: Option<Entity>,
}

impl Grappler {
    pub fn new(
        max_grapple_distance: f32,
        min_grapple_distance: f32,
        far_impulse: f32,
        close_impulse: f32,
    ) -> Self {
        Self {
            max_grapple_distance,
            min_grapple_distance,
            far_impulse,
            close_impulse,
            current_point: None,
            closest_grappleable_point: None,
        }
    }

    pub fn can_grapple(&self) -> bool {
        self.closest_grappleable_point.is_some()
    }

    pub fn is_grappling(&self) -> bool {
        self.current_point.is_some()
    }
}

fn get_grappler_points(
    mut grappler: Query<(&GlobalTransform, &mut Grappler), Without<GrapplePoint>>,
    points: Query<(Entity, &GlobalTransform), (With<GrapplePoint>, Without<Grappler>)>,
) {
    for (transform, mut grappler) in grappler.iter_mut() {
        // Find closest point to grappler
        let mut closest = (f32::MAX, None);
        for (e, point_transform) in points.iter() {
            let dist = point_transform
                .translation()
                .truncate()
                .distance(transform.translation().truncate());

            if dist > grappler.max_grapple_distance {
                continue;
            }

            if dist < closest.0 {
                closest = (dist, Some(e));
            }
        }

        grappler.closest_grappleable_point = closest.1;
    }
}

fn grappling(
    mut grappler: Query<(
    Entity,
        &GlobalTransform,
        &mut Grappler,
        &mut KinematicVelocity,
        &ActionState<InputAction>,
    )>,
    points: Query<(Entity, &GlobalTransform), With<GrapplePoint>>,
    mut cmd: Commands,
    time: Res<Time>,
) {
    let points = points.iter().collect::<HashMap<Entity, &GlobalTransform>>();

    for (entity, transform, mut grappler, mut vel, input) in grappler.iter_mut() {
        if !grappler.can_grapple() && !grappler.is_grappling() {
            continue;
        }

        // If not grappling, dont grapple
        if !input.pressed(InputAction::Grapple) {
            grappler.current_point = None;
            continue;
        }

        // If just started to grapple, set the point
        if grappler.current_point == None {
            let closest = grappler.closest_grappleable_point;
            grappler.current_point = closest;
        }

        // Since the point was set previously, we can unwrap here
        let point = points.get(&grappler.current_point.expect("The current point was None"));

        // Just get the value of the point
        let point = if point.is_none() {
            grappler.current_point = None;
            println!("Grapple point doesn't exist");
            continue;
        } else {
            point
                .expect("Grapple point doesn't exist even though it does?")
                .translation()
                .truncate()
        };
        // TODO: Implement physics using rope joints instead (and of course figure out how tf rope
        // joints work)

        if transform.translation().truncate().distance_squared(point)
        > grappler.max_grapple_distance.powi(2)
        {
            let impulse = -(transform.translation().truncate() - point)
            * grappler.far_impulse
            * time.delta_seconds()
            * (transform.translation().truncate().distance(point)
                - grappler.max_grapple_distance)
                .abs()
            / 100f32;
            vel.x += impulse.x;
            vel.y += impulse.y;
        }
    }
}

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct HorizontalMovement {
    pub max_speed: f32,
    pub acceleration_time: f32,
    pub deccelration_time: f32,
    pub turn_around_multi: f32,
    pub air_control_multi: f32,
    pub air_friction_multi: f32,
}

fn horizontal_movement(
    mut query: Query<(
        &mut KinematicVelocity,
        &HorizontalMovement,
        &KinematicCharacterControllerOutput,
        &ActionState<InputAction>,
    )>,
    time: Res<Time>,
) {
    for (mut vel, movement, output, input) in query.iter_mut() {
        let input_val = input.clamped_value(InputAction::Run);
        if input_val == 0f32 || input_val.signum() != vel.x.signum() {
            deccelerate(&mut vel, movement, output, &time);
        }
        if input_val != 0f32
            && (vel.x.abs() <= movement.max_speed || vel.x.signum() != input_val.signum())
        {
            accelerate(&mut vel, movement, output, input_val, &time)
        }
    }
}

fn deccelerate(
    vel: &mut Mut<KinematicVelocity>,
    movement: &HorizontalMovement,
    output: &KinematicCharacterControllerOutput,
    time: &Res<Time>,
) {
    let sign = vel.x.signum();
    let air_multi = (!output.grounded as u8) as f32 * movement.air_friction_multi
        + (output.grounded as u8) as f32 * 1f32;
    vel.x -=
        movement.max_speed * sign * air_multi / movement.deccelration_time * time.delta_seconds();
    if vel.x.signum() != sign {
        vel.x = 0f32;
    }
}

fn accelerate(
    vel: &mut Mut<KinematicVelocity>,
    movement: &HorizontalMovement,
    output: &KinematicCharacterControllerOutput,
    input_val: f32,
    time: &Res<Time>,
) {
    let mut aim_speed = movement.max_speed * input_val;
    if input_val.signum() != vel.x.signum() {
        aim_speed *= movement.turn_around_multi;
    }

    let mut vel_add = aim_speed / movement.acceleration_time * time.delta_seconds();
    if !output.grounded {
        vel_add *= movement.air_control_multi
    }

    if (vel.x + vel_add).abs() > movement.max_speed && vel.x.signum() == vel_add.signum() {
        return;
    }
    vel.x += vel_add;
}

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct Jumper {
    pub jump_force: f32,
    pub release_multi: f32,
    coyote_time: Timer,
    jump_buffer: Timer,
    can_release: bool,
}

fn jumper(
    mut query: Query<(
        &mut KinematicVelocity,
        &mut Jumper,
        Option<&GroundedState>,
        &ActionState<InputAction>,
        &KinematicCharacterControllerOutput,
    )>,
    time: Res<Time>,
) {
    for (mut vel, mut jumper, state, input, output) in query.iter_mut() {
        if vel.y > 0f32 && input.released(InputAction::Jump) && jumper.can_release {
            jumper.can_release = false;
            vel.y *= jumper.release_multi;
        }
        jumper
            .coyote_time
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        if output.grounded && state.is_some_and(|s| *s != GroundedState::Jumping) {
            jumper.coyote_time.reset();
        }

        jumper
            .jump_buffer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        if input.just_pressed(InputAction::Jump) {
            jumper.jump_buffer.reset();
        }

        if !state.is_some_and(|s| *s == GroundedState::Jumping) {
            return;
        }

        jumper.coyote_time.tick(Duration::from_secs(1000));
        jumper.jump_buffer.tick(Duration::from_secs(1000));
        jumper.can_release = true;
        vel.y += jumper.jump_force;
    }
}

impl Jumper {
    pub fn new(
        jump_force: f32,
        release_multi: f32,
        coyote_time: f32,
        jump_buffer_time: f32,
    ) -> Self {
        Self {
            jump_force,
            release_multi,
            coyote_time: Timer::from_seconds(coyote_time, TimerMode::Once),
            jump_buffer: Timer::from_seconds(jump_buffer_time, TimerMode::Once),
            can_release: false,
        }
    }

    pub fn can_jump(&self) -> bool {
        !self.coyote_time.finished() && !self.jump_buffer.finished()
    }
}

#[derive(Resource, Copy, Clone, Debug, PartialEq, Reflect)]
pub struct KinematicGravity(pub Vec2);

#[derive(Default, Component, Copy, Clone, Debug, PartialEq, Reflect)]
pub struct KinematicGravityUser;

fn kinematic_gravity(
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

pub mod velocity {
    use super::*;

    #[derive(Clone, Copy, Component, Debug, PartialEq, Reflect, Default)]
    pub struct KinematicVelocity {
        pub x: f32,
        pub y: f32,
    }

    impl KinematicVelocity {
        fn to_vec2(self) -> Vec2 {
            Vec2::from(self)
        }
    }

    impl From<Vec2> for KinematicVelocity {
        fn from(value: Vec2) -> Self {
            Self {
                x: value.x,
                y: value.y,
            }
        }
    }

    impl From<KinematicVelocity> for Vec2 {
        fn from(value: KinematicVelocity) -> Self {
            Self {
                x: value.x,
                y: value.y,
            }
        }
    }

    impl std::ops::Add for KinematicVelocity {
        type Output = Self;
        fn add(self, rhs: Self) -> Self::Output {
            Self::from(Vec2::from(self) + Vec2::from(rhs))
        }
    }

    impl std::ops::Add<Vec2> for KinematicVelocity {
        type Output = Vec2;
        fn add(self, rhs: Vec2) -> Self::Output {
            rhs + Vec2::from(self)
        }
    }
    impl std::ops::Add<KinematicVelocity> for Vec2 {
        type Output = KinematicVelocity;
        fn add(self, rhs: KinematicVelocity) -> Self::Output {
            rhs + KinematicVelocity::from(self)
        }
    }

    impl std::ops::AddAssign<Vec2> for KinematicVelocity {
        fn add_assign(&mut self, rhs: Vec2) {
            self.x += rhs.x;
            self.y += rhs.y;
        }
    }

    pub fn kinematic_velocity(
        mut query: Query<(
            &mut KinematicVelocity,
            &mut Transform,
            Option<&mut KinematicCharacterController>,
            Option<&KinematicCharacterControllerOutput>,
        )>,
        time: Res<Time>,
    ) {
        for (mut vel, mut transform, controller, output) in query.iter_mut() {
            kinematic_velocity_collision_check(&mut vel, &output);
            let translation = vel.to_vec2() * time.delta_seconds();
            match controller {
                Some(mut controller) => controller.translation = Some(translation),
                None => transform.translation += translation.extend(0f32),
            }
        }
    }

    fn kinematic_velocity_collision_check(
        vel: &mut KinematicVelocity,
        output: &Option<&KinematicCharacterControllerOutput>,
    ) {
        const LENIANCY: f32 = 0.01f32;

        let output = match output {
            Some(o) => *o,
            None => return,
        };
        let diff = (output.desired_translation - output.effective_translation).abs();
        if diff.x > LENIANCY {
            vel.x = 0f32;
        }
        if diff.y > LENIANCY {
            vel.y = 0f32
        }
    }
}
