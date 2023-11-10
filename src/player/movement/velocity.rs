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
