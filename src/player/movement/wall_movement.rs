use super::*;

/// The i8s indicate where the wall is in relation to the player (-1 is to the left, and 1 is to
/// the right)
#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct WallMovement {
    pub walljump_force: Vec2,
    pub walljump_y_range: (f32, f32),
    pub max_wall_toi: f32,
    pub coyote_time: (Timer, i8),
    pub current_wall: Option<(Entity, i8)>,
}

pub fn wall_movement(
    mut query: Query<(
        Entity,
        &mut WallMovement,
        Option<&GroundedState>,
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

    for (entity, mut wall_mover, grounded_state, transform, sprite) in query.iter_mut() {
        get_walls(
            entity,
            &mut wall_mover,
            sprite,
            transform,
            grounded_state,
            &ctx,
            &bodies,
        );
        coyote_timer(&mut wall_mover, &time);
    }
}

fn coyote_timer(wall_mover: &mut Mut<WallMovement>, time: &Res<Time>) {
    wall_mover
        .coyote_time
        .0
        .tick(Duration::from_secs_f32(time.delta_seconds()));
    if let Some((_, i)) = wall_mover.current_wall {
        wall_mover.coyote_time.0.reset();
        wall_mover.coyote_time.1 = i;
    }
}

fn get_walls(
    wall_mover_entity: Entity,
    wall_mover: &mut Mut<WallMovement>,
    sprite: &TextureAtlasSprite,
    transform: &GlobalTransform,
    state: Option<&GroundedState>,
    ctx: &Res<RapierContext>,
    walls_map: &HashMap<Entity, Option<&RigidBody>>,
) {
    if state.is_some() {
        wall_mover.current_wall = None;
        return;
    }

    let pos = transform.translation().truncate()
        + (wall_mover.walljump_y_range.0 + wall_mover.walljump_y_range.1) / 2f32;
    let size = match sprite.custom_size {
        Some(v) => Vec2::new(
            v.x,
            wall_mover.walljump_y_range.0.abs() + wall_mover.walljump_y_range.1.abs(),
        ),
        None => Vec2::new(25f32, 50f32),
    };

    let new_shape = Collider::cuboid(size.x / 2f32, size.y / 2f32);

    let predi = |e: Entity| {
        e != wall_mover_entity
            && !walls_map
                .get(&e)
                .unwrap()
                .is_some_and(|r| *r == RigidBody::Dynamic)
    };
    let filter = QueryFilter::default().exclude_sensors().predicate(&predi);

    let mut cast_shape = |vel: Vec2| -> Option<Toi> {
        if let Some((entity, hit)) =
            ctx.cast_shape(pos, 0f32, vel, &new_shape, wall_mover.max_wall_toi, filter)
        {
            let i = if hit.witness1.x < pos.x { -1 } else { 1 };
            wall_mover.current_wall = Some((entity, i));
            Some(hit)
        } else {
            None
        }
    };

    if cast_shape(Vec2::X).is_none() && cast_shape(Vec2::NEG_X).is_none() {
        wall_mover.current_wall = None;
    }
}

impl WallMovement {
    pub fn new(
        walljump_force: Vec2,
        walljump_y_range: (f32, f32),
        max_wall_toi: f32,
        coyote_time: f32,
    ) -> Self {
        Self {
            walljump_force,
            walljump_y_range,
            max_wall_toi,
            current_wall: None,
            coyote_time: (Timer::from_seconds(coyote_time, TimerMode::Once), 0),
        }
    }

    pub fn coyote_timer_remaining(&self) -> f32 {
        self.coyote_time.0.remaining_secs()
    }

    /// Same as jump, but with walls
    pub fn walljump(&mut self, jumper: &mut Mut<Jumper>, current_vel: Vec2) -> Vec2 {
        self.coyote_time.0.tick(Duration::from_secs(1000));
        jumper.jump_buffer.tick(Duration::from_secs(1000));
        jumper.can_release = true;
        current_vel + self.walljump_force * Vec2::new(-self.coyote_time.1 as f32, 1f32)
    }

    pub fn get_current_wall(&self) -> Option<(Entity, i8)> {
        self.current_wall
    }
}
