use super::*;

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct Jumper {
    pub jump_force: f32,
    pub x_multi: f32,
    pub release_multi: f32,
    pub walljump_multi: Vec2,
    pub can_release: bool,
    pub max_wall_toi: f32,
    pub walljump_y_range: (f32, f32),
    pub terminal_vel: f32,
    pub wallslide_terminal_vel: f32,
    pub hold_down_vel_multi: f32,
    coyote_time: Timer,
    walljump_coyote_time: (Timer, i8),
    jump_buffer: Timer,
    current_wall: Option<(Entity, i8)>,
}

pub fn jumper(
    mut query: Query<(
        Entity,
        &mut KinematicVelocity,
        &mut Jumper,
        Option<&GroundedState>,
        Option<&WallState>,
        &ActionState<InputAction>,
        &KinematicCharacterControllerOutput,
        &GlobalTransform,
        &TextureAtlasSprite,
    )>,
    rb_query: Query<(Entity, Option<&RigidBody>)>,
    ctx: Res<RapierContext>,
    time: Res<Time>,
) {
    let bodies = rb_query
        .iter()
        .collect::<HashMap<Entity, Option<&RigidBody>>>();

    for (
        entity,
        mut vel,
        mut jumper,
        grounded_state,
        wall_state,
        input,
        output,
        transform,
        sprite,
    ) in query.iter_mut()
    {
        get_walls(
            entity,
            &mut jumper,
            sprite,
            transform,
            grounded_state,
            &ctx,
            &bodies,
        );
        jumper_timers(&mut vel, &mut jumper, grounded_state, input, output, &time);
        jumping(&mut jumper, &mut vel, grounded_state, wall_state)
    }
}

fn jumping(
    jumper: &mut Mut<Jumper>,
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
        let v = jumper.walljump(Vec2::new(vel.x, vel.y));
        vel.x = v.x;
        vel.y = v.y;
        return;
    }
}

fn get_walls(
    jumper_entity: Entity,
    jumper: &mut Mut<Jumper>,
    sprite: &TextureAtlasSprite,
    transform: &GlobalTransform,
    state: Option<&GroundedState>,
    ctx: &Res<RapierContext>,
    walls_map: &HashMap<Entity, Option<&RigidBody>>,
) {
    if state.is_some() {
        jumper.current_wall = None;
        return;
    }

    let pos = transform.translation().truncate()
        + (jumper.walljump_y_range.0 + jumper.walljump_y_range.1) / 2f32;
    let size = match sprite.custom_size {
        Some(v) => Vec2::new(
            v.x,
            jumper.walljump_y_range.0.abs() + jumper.walljump_y_range.1.abs(),
        ),
        None => Vec2::new(25f32, 50f32),
    };

    let new_shape = Collider::cuboid(size.x / 2f32, size.y / 2f32);

    let predi = |e: Entity| {
        e != jumper_entity
            && !walls_map
                .get(&e)
                .unwrap()
                .is_some_and(|r| *r == RigidBody::Dynamic)
    };
    let filter = QueryFilter::default().exclude_sensors().predicate(&predi);

    let mut cast_shape = |vel: Vec2| -> Option<Toi> {
        if let Some((entity, hit)) =
            ctx.cast_shape(pos, 0f32, vel, &new_shape, jumper.max_wall_toi, filter)
        {
            let i = if hit.witness1.x < pos.x { -1 } else { 1 };
            jumper.current_wall = Some((entity, i));
            Some(hit)
        } else {
            None
        }
    };

    cast_shape(Vec2::X);
    cast_shape(Vec2::NEG_X);
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

    jumper
        .walljump_coyote_time
        .0
        .tick(Duration::from_secs_f32(time.delta_seconds()));
    if let Some((_, i)) = jumper.current_wall {
        jumper.walljump_coyote_time.0.reset();
        jumper.walljump_coyote_time.1 = i;
    }
}

impl Jumper {
    pub fn new(
        jump_force: f32,
        release_multi: f32,
        x_multi: f32,
        walljump_multi: Vec2,
        coyote_time: f32,
        jump_buffer_time: f32,
        max_wall_toi: f32,
        walljump_y_range: (f32, f32),
        terminal_vel: f32,
        wallslide_terminal_vel: f32,
        hold_down_vel_multi: f32,
    ) -> Self {
        Self {
            jump_force,
            release_multi,
            x_multi,
            walljump_multi,
            walljump_y_range,
            coyote_time: Timer::from_seconds(coyote_time, TimerMode::Once),
            walljump_coyote_time: (Timer::from_seconds(coyote_time, TimerMode::Once), 0),
            jump_buffer: Timer::from_seconds(jump_buffer_time, TimerMode::Once),
            max_wall_toi,
            terminal_vel,
            wallslide_terminal_vel,
            hold_down_vel_multi,
            can_release: false,
            current_wall: None,
        }
    }

    pub fn should_jump(&self) -> bool {
        !self.coyote_time.finished() && !self.jump_buffer.finished()
    }

    pub fn should_walljump(&self) -> bool {
        !self.walljump_coyote_time.0.finished() && !self.jump_buffer.finished()
    }

    pub fn coyote_timer_remaining(&self) -> f32 {
        self.coyote_time.remaining_secs()
    }

    pub fn walljump_coyote_timer_remaining(&self) -> f32 {
        self.walljump_coyote_time.0.remaining_secs()
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

    /// Same as jump, but with walls
    pub fn walljump(&mut self, current_vel: Vec2) -> Vec2 {
        self.walljump_coyote_time.0.tick(Duration::from_secs(1000));
        self.jump_buffer.tick(Duration::from_secs(1000));
        self.can_release = true;
        current_vel
            + Vec2::new(0f32, self.jump_force)
                * Vec2::new(self.x_multi, 1f32)
                * self.walljump_multi
                * Vec2::new(self.walljump_coyote_time.1 as f32, 1f32)
    }

    pub fn get_current_wall(&self) -> Option<(Entity, i8)> {
        self.current_wall
    }
}
