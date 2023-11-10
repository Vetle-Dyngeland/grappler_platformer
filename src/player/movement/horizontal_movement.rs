use super::*;

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct HorizontalMovement {
    pub max_speed: f32,
    pub acceleration_time: f32,
    pub deccelration_time: f32,
    pub turn_around_multi: f32,
    pub air_control_multi: f32,
    pub air_friction_multi: f32,
}

pub fn horizontal_movement(
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
