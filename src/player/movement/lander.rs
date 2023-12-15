use super::*;

#[derive(Component, Clone, Debug, PartialEq, Reflect)]
pub struct Lander {
    pub land_buffer_timer: Timer,
    pub vel_boost: Vec2,
    pub vel_multi: f32,
    pub vel_cap: f32,
    pub min_land_vel: f32,
    prev_state: Option<InAirState>,
    prev_vel: f32,
}

impl Lander {
    pub fn new(
        land_buffer_time: f32,
        vel_boost: Vec2,
        vel_multi: f32,
        vel_cap: f32,
        min_land_vel: f32,
    ) -> Self {
        Self {
            land_buffer_timer: Timer::from_seconds(land_buffer_time, TimerMode::Once),
            vel_boost,
            vel_multi,
            vel_cap,
            min_land_vel,
            prev_state: None,
            prev_vel: 0f32,
        }
    }
}

pub fn lander(
    mut query: Query<(
        &mut Lander,
        &mut KinematicVelocity,
        &ActionState<InputAction>,
        Option<&InAirState>,
        Option<&GroundedState>,
        &KinematicCharacterControllerOutput,
    )>,
    time: Res<Time>,
) {
    let end = |lander: &mut Mut<Lander>, s: Option<&InAirState>, vel: &Mut<KinematicVelocity>| {
        lander.prev_state = s.map(|s| *s); // Do after all other state handling
        lander.prev_vel = vel.y;
    };

    for (mut lander, mut vel, input, in_air_state, grounded_state, output) in query.iter_mut() {
        lander
            .land_buffer_timer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        if input.just_pressed(InputAction::Land) {
            lander.land_buffer_timer.reset();
        }
        if !should_land(&lander, &input, &grounded_state) {
            end(&mut lander, in_air_state, &vel);
            continue;
        }
        println!("Should land!");

        let dir = get_landing_dir(&vel, &output);
        let multi = get_landing_multi() * 1000f32;

        vel.x += (dir * multi).x;
        vel.y += (dir * multi).y;

        end(&mut lander, in_air_state, &vel)
    }
}

fn should_land(
    lander: &Mut<Lander>,
    input: &ActionState<InputAction>,
    state: &Option<&GroundedState>,
) -> bool {
    state.is_some()
        && input.pressed(InputAction::Land)
        && lander.prev_vel < lander.min_land_vel
        && lander.prev_state.is_some()
        && !lander.land_buffer_timer.finished()
}

fn get_landing_dir(
    vel: &Mut<KinematicVelocity>,
    output: &KinematicCharacterControllerOutput,
) -> Vec2 {
    if output.effective_translation.length_squared() - output.desired_translation.length_squared()
        > 0.01f32
    {
        let desired = output.desired_translation.normalize_or_zero();
        if !desired.is_nan() && desired != Vec2::ZERO {
            return desired;
        }
    }

    let vel = Vec2::new(vel.x, vel.y).normalize_or_zero();
    if vel == Vec2::ZERO {
        return Vec2::ONE.normalize_or_zero();
    }

    vel.normalize_or_zero()
}

fn get_landing_multi() -> Vec2 {
    Vec2::new(1f32, 1f32)
}
