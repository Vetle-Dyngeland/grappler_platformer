use super::*;

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct Jumper {
    pub jump_force: f32,
    pub x_multi: f32,
    pub release_multi: f32,
    pub can_release: bool,
    pub coyote_time: Timer,
    pub jump_buffer: Timer,
}

pub fn jumper(
    mut query: Query<(
        &mut KinematicVelocity,
        &mut Jumper,
        &mut WallMovement,
        Option<&GroundedState>,
        Option<&WallState>,
        &ActionState<InputAction>,
        &KinematicCharacterControllerOutput,
    )>,
    time: Res<Time>,
) {
    for (mut vel, mut jumper, mut wall_mover, grounded_state, wall_state, input, output) in
        query.iter_mut()
    {
        jumper_timers(&mut vel, &mut jumper, grounded_state, input, output, &time);
        jumping(
            &mut jumper,
            &mut wall_mover,
            &mut vel,
            grounded_state,
            wall_state,
        )
    }
}

fn jumping(
    jumper: &mut Mut<Jumper>,
    wall_mover: &mut Mut<WallMovement>,
    vel: &mut Mut<KinematicVelocity>,
    grounded_state: Option<&GroundedState>,
    wall_state: Option<&WallState>,
) {
    if grounded_state.is_some_and(|s| *s == GroundedState::Jumping) {
        let v = jumper.jump(Vec2::new(vel.x, vel.y));
        vel.x = v.x;
        vel.y = v.y;
        return;
    }
    if wall_state.is_some_and(|s| *s == WallState::Jumping) {
        let v = wall_mover.walljump(jumper, Vec2::new(vel.x, vel.y));
        vel.x = v.x;
        vel.y = v.y;
        return;
    }
}

fn jumper_timers(
    vel: &mut Mut<KinematicVelocity>,
    jumper: &mut Mut<Jumper>,
    state: Option<&GroundedState>,
    input: &ActionState<InputAction>,
    output: &KinematicCharacterControllerOutput,
    time: &Res<Time>,
) {
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
}

impl Jumper {
    pub fn new(
        jump_force: f32,
        release_multi: f32,
        x_multi: f32,
        coyote_time: f32,
        jump_buffer_time: f32,
    ) -> Self {
        Self {
            jump_force,
            release_multi,
            x_multi,
            coyote_time: Timer::from_seconds(coyote_time, TimerMode::Once),
            jump_buffer: Timer::from_seconds(jump_buffer_time, TimerMode::Once),
            can_release: false,
        }
    }

    pub fn should_jump(&self) -> bool {
        !self.coyote_time.finished() && !self.jump_buffer.finished()
    }

    pub fn should_walljump(&self, wall_mover: &WallMovement) -> bool {
        !wall_mover.coyote_time.0.finished() && !self.jump_buffer.finished()
    }

    pub fn coyote_timer_remaining(&self) -> f32 {
        self.coyote_time.remaining_secs()
    }

    pub fn jump_buffer_remaining(&self) -> f32 {
        self.jump_buffer.remaining_secs()
    }

    /// Takes the current velocity and returns what the velocity will be after the jump
    /// Also resets the coyote timer and jump buffer timer and sets can_release to true
    pub fn jump(&mut self, current_vel: Vec2) -> Vec2 {
        self.coyote_time.tick(Duration::from_secs(1000));
        self.jump_buffer.tick(Duration::from_secs(1000));
        self.can_release = true;
        current_vel + Vec2::new(0f32, self.jump_force) * Vec2::new(self.x_multi, 1f32)
    }
}
