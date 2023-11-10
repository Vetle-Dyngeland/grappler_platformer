use super::*;

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct Jumper {
    pub jump_force: f32,
    pub release_multi: f32,
    coyote_time: Timer,
    jump_buffer: Timer,
    can_release: bool,
}

pub fn jumper(
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
