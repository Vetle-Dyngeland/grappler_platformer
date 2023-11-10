use super::*;

#[derive(Default, Component, Clone, Debug, PartialEq, Reflect)]
pub struct Grappler {
    pub max_grapple_distance: f32,
    pub min_desired: f32,
    pub max_desired: f32,
    pub far_springyness: f32,
    pub close_springyness: f32,
    current_point: Option<Entity>,
    closest_grappleable_point: Option<Entity>,
}

impl Grappler {
    pub fn new(
        max_grapple_distance: f32,
        min_desired: f32,
        max_desired: f32,
        far_springyness: f32,
        close_springyness: f32,
    ) -> Self {
        Self {
            max_grapple_distance,
            min_desired,
            max_desired,
            far_springyness,
            close_springyness,
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

pub fn get_closest_points(
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

// Messiest code on the planet
pub fn grappler_movement(
    mut grappler: Query<(
        &GlobalTransform,
        &mut Grappler,
        &mut KinematicVelocity,
        &ActionState<InputAction>,
    )>,
    points: Query<(Entity, &GlobalTransform), With<GrapplePoint>>,
    time: Res<Time>,
) {
    let points = points.iter().collect::<HashMap<Entity, &GlobalTransform>>();

    for (transform, mut grappler, mut vel, input) in grappler.iter_mut() {
        // If not grappling and cant grapple, or not pressing grapple, dont grapple
        if (!grappler.can_grapple() && !grappler.is_grappling())
            || !input.pressed(InputAction::Grapple)
        {
            grappler.current_point = None;
            continue;
        }

        let point = match get_point(&points, &mut grappler) {
            Ok(v) => v,
            Err(s) => {
                println!("{s}");
                grappler.current_point = None;
                continue;
            }
        };

        let mut force = Vec2::splat(time.delta_seconds());
        let pos = transform.translation().truncate();

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
            continue;
        };

        let impulse = (point - transform.translation().truncate()) * force;
        vel.x += impulse.x;
        vel.y += impulse.y;
    }
}

fn get_point(
    points: &HashMap<Entity, &GlobalTransform>,
    grappler: &mut Grappler,
) -> Result<Vec2, String> {
    let current = match grappler.current_point {
        Some(e) => e,
        None => {
            let closest = grappler.closest_grappleable_point;
            grappler.current_point = closest;
            match closest {
                Some(e) => e,
                None => return Err("Closest point was none".to_string()),
            }
        }
    };

    match points.get(&current) {
        // If the current point is none, it was removed during the frame
        None => Err("Current poitn was none".to_string()),
        // We just need the position of the point, not the entire transform
        Some(v) => Ok(v.translation().truncate()),
    }
}
