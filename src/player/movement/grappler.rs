use super::*;

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct Grappler {
    pub range: f32,
    pub min_desired: f32,
    pub max_desired: f32,
    pub far_springyness: f32,
    pub close_springyness: f32,
    pub grapple_buffer: f32,
    current_point: Option<Entity>,
    closest_point: Option<Entity>,
    grapple_buffer_timer: Option<f32>,
}

impl Grappler {
    pub fn new(
        range: f32,
        min_desired: f32,
        max_desired: f32,
        far_springyness: f32,
        close_springyness: f32,
        grapple_buffer: f32,
    ) -> Self {
        Self {
            range,
            min_desired,
            max_desired,
            far_springyness,
            close_springyness,
            grapple_buffer,
            current_point: None,
            closest_point: None,
            grapple_buffer_timer: None,
        }
    }

    pub fn can_grapple(&self) -> bool {
        self.closest_point.is_some()
    }

    pub fn is_grappling(&self) -> bool {
        self.current_point.is_some()
    }
}

pub fn grappler_movement(
    mut grappler: Query<(
        &GlobalTransform,
        &mut Grappler,
        &mut KinematicVelocity,
        &ActionState<InputAction>,
        &mut Jumper,
    )>,
    points: Query<(Entity, &GlobalTransform), With<GrapplePoint>>,
    time: Res<Time>,
) {
    let points_map = points.iter().collect::<HashMap<Entity, &GlobalTransform>>();
    let points_vec = points.iter().collect::<Vec<(Entity, &GlobalTransform)>>();

    for (transform, mut grappler, mut vel, input, mut jumper) in grappler.iter_mut() {
        get_closest_points((transform, &mut grappler), &points_vec);

        if grapple_buffer(&mut grappler, &input, &time) {
            continue;
        }

        // If not grappling and cant grapple, or not pressing grapple, dont grapple
        if !grappler.can_grapple() && !grappler.is_grappling() {
            grappler.current_point = None;
            continue;
        }

        let point = match get_point(&points_map, &mut grappler) {
            Ok(v) => v,
            Err(s) => {
                if let Some(s) = s {
                    println!("{s}");
                }
                grappler.current_point = None;
                continue;
            }
        };

        let pos = transform.translation().truncate();
        let v = grappler_forces(&time, pos, &mut grappler, point);
        vel.x += v.x;
        vel.y += v.y;

        if jumper.jump_buffer_remaining() > 0f32 {
            let v = jumper.jump(Vec2::new(vel.x, vel.y));
            vel.x = v.x;
            vel.y = v.y;
            grappler.current_point = None;
        }
    }
}

fn get_closest_points(
    grappler: (&GlobalTransform, &mut Mut<Grappler>),
    points: &Vec<(Entity, &GlobalTransform)>,
) {
    // Find closest point to grappler
    let mut closest = (f32::MAX, None);
    for (e, point_transform) in points.iter() {
        let dist = point_transform
            .translation()
            .truncate()
            .distance(grappler.0.translation().truncate());

        if dist > grappler.1.range {
            continue;
        }

        if dist < closest.0 {
            closest = (dist, Some(e));
        }
    }

    grappler.1.closest_point = closest.1.copied();

}
/// returns true if should continue
fn grapple_buffer(
    grappler: &mut Mut<Grappler>,
    input: &ActionState<InputAction>,
    time: &Res<Time>,
) -> bool {
    if let Some(t) = grappler.grapple_buffer_timer {
        grappler.grapple_buffer_timer = Some(t + time.delta_seconds());
    }

    if input.just_pressed(InputAction::Grapple) {
        grappler.grapple_buffer_timer = Some(0f32);
    }

    if !input.pressed(InputAction::Grapple) {
        grappler.grapple_buffer_timer = None;
        grappler.current_point = None;
        return true;
    }
    false
}

fn get_point(
    points: &HashMap<Entity, &GlobalTransform>,
    grappler: &mut Grappler,
) -> Result<Vec2, Option<String>> {
    let current = match grappler.current_point {
        Some(e) => e,
        None => {
            match grappler.grapple_buffer_timer {
                None => return Err(Some("Hasn't started grappling yet?".to_string())),
                Some(t) => {
                    if t >= grappler.grapple_buffer {
                        return Err(None);
                    }
                }
            }

            let closest = grappler.closest_point;
            grappler.current_point = closest;
            match closest {
                Some(e) => e,
                None => return Err(Some("Closest point was none".to_string())),
            }
        }
    };

    match points.get(&current) {
        // If the current point is none, it was removed during the frame
        None => Err(Some("Current point was none".to_string())),
        // We just need the position of the point, not the entire transform
        Some(v) => Ok(v.translation().truncate()),
    }
}

fn grappler_forces(time: &Res<Time>, pos: Vec2, grappler: &mut Mut<Grappler>, point: Vec2) -> Vec2 {
    let mut force = Vec2::splat(time.delta_seconds());

    // Force if distance is less than the min desired distance
    let min_force = -grappler.close_springyness
        * (pos.distance(point) - grappler.min_desired)
            .powf(2f32)
            .abs()
        * Vec2::ONE;

    // Force if distance is more than the max desired distance
    let max_force = grappler.far_springyness
        * (pos.distance(point) - grappler.max_desired)
            .powf(1.65f32)
            .abs()
        * Vec2::new(1.2f32, 1f32);

    // Just normal ternary operations
    force *= if pos.distance_squared(point) < grappler.min_desired.powi(2) {
        min_force
    } else if pos.distance_squared(point) > grappler.max_desired.powi(2) {
        max_force
    } else {
        Vec2::ZERO
    };

    return (point - pos) * force;
}
