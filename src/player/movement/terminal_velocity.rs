use super::*;

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct TerminalVelocity {
    pub base_val: f32,
    pub hold_down_multi: f32,
    pub wall_slide_multi: f32,
}

pub fn terminal_velocity(
    mut query: Query<(
        &mut KinematicVelocity,
        &TerminalVelocity,
        &WallMovement,
        &ActionState<InputAction>,
    )>,
) {
    let bool_multi = |b: bool, m: f32| (b as i8) as f32 * m + (!b as i8) as f32;

    for (mut vel, terminal_vel, wall_mover, input) in query.iter_mut() {
        let val = terminal_vel.base_val
            * bool_multi(
                wall_mover.current_wall.is_some(),
                terminal_vel.wall_slide_multi,
            )
            * bool_multi(
                input.pressed(InputAction::Down),
                terminal_vel.hold_down_multi,
            );
        if vel.y < val {
            vel.y = val
        }
    }
}

impl TerminalVelocity {
    pub fn new(base_val: f32, hold_down_multi: f32, wall_slide_multi: f32) -> Self {
        Self {
            base_val,
            hold_down_multi,
            wall_slide_multi,
        }
    }
}
