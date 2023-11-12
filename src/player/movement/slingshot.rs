use std::{collections::HashMap, time::Duration};

use crate::{level::GrapplePoint, player::input::InputAction};

use super::{jumper::Jumper, velocity::KinematicVelocity};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

#[derive(Component, Clone, Debug, PartialEq, Reflect)]
pub struct Slingshot {
    pub force: f32,
    pub range: f32,
    pub dir_multi: Vec2,
    pub above_multi: f32,
    closest_point: Option<Entity>,
    buffer_timer: Timer,
    delay_timer: Timer,
}

impl Slingshot {
    pub fn new(
        force: f32,
        range: f32,
        dir_multi: Vec2,
        above_multi: f32,
        buffer_time: f32,
        delay_time: f32,
    ) -> Self {
        Self {
            force,
            range,
            dir_multi,
            above_multi,
            closest_point: None,
            buffer_timer: Timer::from_seconds(buffer_time, TimerMode::Once),
            delay_timer: Timer::from_seconds(delay_time, TimerMode::Once),
        }
    }

    pub fn can_slingshot(&self) -> bool {
        self.closest_point.is_some()
    }

    pub fn get_closest(&self) -> Option<Entity> {
        self.closest_point
    }
}

pub fn slingshot(
    mut slingshot: Query<
        (
            &mut Slingshot,
            &GlobalTransform,
            &ActionState<InputAction>,
            &mut KinematicVelocity,
            Option<&mut Jumper>,
        ),
        Without<GrapplePoint>,
    >,
    points: Query<(Entity, &GlobalTransform), With<GrapplePoint>>,
    time: Res<Time>,
) {
    let points_map = points.iter().collect::<HashMap<Entity, &GlobalTransform>>();
    let points_vec = points.iter().collect::<Vec<(Entity, &GlobalTransform)>>();

    for (mut slingshot, transform, input, mut vel, jumper) in slingshot.iter_mut() {
        get_closest_points((transform, &mut slingshot), &points_vec);
        buffer_time(&mut slingshot, input, &time);

        if !slingshot.buffer_timer.finished() && slingshot.can_slingshot() {
            slingshot_impulse(&mut slingshot, transform, &mut vel, &points_map, jumper);
        }
    }
}

fn get_closest_points(
    slingshot: (&GlobalTransform, &mut Mut<Slingshot>),
    points: &Vec<(Entity, &GlobalTransform)>,
) {
    // Find closest point to grappler
    let mut closest = (f32::MAX, None);
    for (e, point_transform) in points.iter() {
        let dist = point_transform
            .translation()
            .truncate()
            .distance(slingshot.0.translation().truncate());

        if dist > slingshot.1.range {
            continue;
        }

        if dist < closest.0 {
            closest = (dist, Some(e));
        }
    }

    slingshot.1.closest_point = closest.1.copied();
}

fn buffer_time(slingshot: &mut Mut<Slingshot>, input: &ActionState<InputAction>, time: &Res<Time>) {
    slingshot
        .buffer_timer
        .tick(Duration::from_secs_f32(time.delta_seconds()));

    if !slingshot.delay_timer.finished() {
        slingshot
            .delay_timer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        return;
    }

    if input.just_pressed(InputAction::Slingshot) {
        slingshot.buffer_timer.reset();
    }
}

fn slingshot_impulse(
    slingshot: &mut Mut<Slingshot>,
    slingshot_pos: &GlobalTransform,
    vel: &mut Mut<KinematicVelocity>,
    points: &HashMap<Entity, &GlobalTransform>,
    jumper: Option<Mut<Jumper>>,
) {
    let point = match get_point(&*slingshot, points) {
        Ok(t) => t,
        Err(s) => {
            println!("{s}");
            return;
        }
    };

    // Set variables
    slingshot.buffer_timer.tick(Duration::from_secs(1000));
    slingshot.closest_point = None;
    slingshot.delay_timer.reset();

    if let Some(mut jumper) = jumper {
        jumper.can_release = false
    }

    // the actual impulse
    let dir = (point - slingshot_pos.translation().truncate()).normalize();

    // just branchless since i dont want to make an if statement, not a performance issue
    let force = slingshot.force
        * slingshot.dir_multi
        // basically: if dir < 0f32, then multiply it by above multi
        * (((dir.y < 0f32) as u8) as f32 * slingshot.above_multi
           // else, mutliply it by 1
            + ((dir.y >= 0f32) as u8) as f32 * 1f32);

    vel.x += dir.x * force.x;

    if vel.y < 0f32 {
        vel.y = dir.y.abs() * force.y;
    } else {
        vel.y += dir.y.abs() * force.y;
    }
}

fn get_point(
    slingshot: &Slingshot,
    points: &HashMap<Entity, &GlobalTransform>,
) -> Result<Vec2, String> {
    let point_entity = match slingshot.closest_point {
        Some(e) => e,
        None => return Err("Could not get closest point entity".to_string()),
    };

    Ok(match points.get(&point_entity) {
        Some(transform) => transform.translation().truncate(),
        None => return Err("Point was None, probably deleted during the frame".to_string()),
    })
}
