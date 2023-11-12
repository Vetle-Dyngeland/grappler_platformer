use super::{Player, PlayerSet};
use bevy::prelude::*;

use seldom_state::prelude::*;
use states::*;
use triggers::*;

pub(super) struct StateMachinePlugin;

impl Plugin for StateMachinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init.in_set(PlayerSet::StateMachine))
            .register_type::<GroundedState>()
            .register_type::<InAirState>()
            .register_type::<WallState>();
    }
}

fn init(mut cmd: Commands, player_query: Query<Entity, With<Player>>) {
    cmd.entity(player_query.single()).insert((
        GroundedState::Idle, StateMachine::default()
            .trans::<InAirState>(GroundedTrigger, GroundedState::Idle)
            .trans::<GroundedState>(
                StateIsTrigger(GroundedState::Walking)
                    .not()
                    .and(WalkingTrigger),
                GroundedState::Walking,
            )
            .trans::<GroundedState>(StateIsTrigger(GroundedState::Jumping), InAirState::Rising)
            .trans::<GroundedState>(JumpTrigger, GroundedState::Jumping)
            .trans::<AnyState>(
                FallingTrigger
                    .and(GroundedTrigger.not())
                    .and(StateIsTrigger(InAirState::Falling).not())
                    .and(GrapplingTrigger.not()),
                InAirState::Falling,
            )
            .trans::<AnyState>(
                FallingTrigger
                    .not()
                    .and(GroundedTrigger.not())
                    .and(StateIsTrigger(InAirState::Rising).not())
                    .and(GrapplingTrigger.not()),
                InAirState::Rising,
            )
            .trans::<InAirState>(
                GrapplingTrigger.and(StateIsTrigger(InAirState::Grapple).not()),
                InAirState::Grapple,
            ),
    ));
}

pub mod states {
    use bevy::prelude::*;

    #[derive(Clone, Copy, Component, Reflect, PartialEq, PartialOrd, Debug)]
    #[component(storage = "SparseSet")]
    pub enum GroundedState {
        Idle,
        Walking,
        Jumping,
    }

    #[derive(Clone, Copy, Component, Reflect, PartialEq, PartialOrd, Debug)]
    #[component(storage = "SparseSet")]
    pub enum InAirState {
        Rising,
        Falling,
        Grapple,
    }

    #[derive(Clone, Copy, Component, Reflect, PartialEq, PartialOrd, Debug)]
    #[component(storage = "SparseSet")]
    pub enum WallState {
        Rising,
        Sliding,
        Jumping,
    }
}

pub mod triggers {
    use super::*;
    use crate::player::movement::{velocity::*, grappler::*, jumper::*};
    use bevy_rapier2d::prelude::*;

    #[derive(Copy, Clone, Debug, Reflect, PartialEq)]
    pub struct CanGrappleTrigger;

    impl BoolTrigger for CanGrappleTrigger {
        type Param<'w, 's> = Query<'w, 's, Option<&'static Grappler>>;

        fn trigger(&self, entity: Entity, param: Self::Param<'_, '_>) -> bool {
            param
                .get(entity)
                .is_ok_and(|g| g.is_some_and(|g| g.can_grapple()))
        }
    }

    #[derive(Copy, Clone, Debug, Reflect, PartialEq)]
    pub struct GrapplingTrigger;

    impl BoolTrigger for GrapplingTrigger {
        type Param<'w, 's> = Query<'w, 's, Option<&'static Grappler>>;

        fn trigger(&self, entity: Entity, param: Self::Param<'_, '_>) -> bool {
            param
                .get(entity)
                .is_ok_and(|g| g.is_some_and(|g| g.is_grappling()))
        }
    }

    #[derive(Copy, Clone, Debug, Reflect, PartialEq)]
    pub struct StateIsTrigger<T: Component + PartialEq>(pub T);

    impl<T> BoolTrigger for StateIsTrigger<T>
    where
        T: Component + PartialEq,
    {
        type Param<'w, 's> = Query<'w, 's, Option<&'static T>>;

        fn trigger(&self, entity: Entity, param: Self::Param<'_, '_>) -> bool {
            param
                .get(entity)
                .is_ok_and(|s| s.is_some_and(|s| *s == self.0))
        }
    }

    #[derive(Copy, Clone, Debug, Reflect, PartialEq)]
    pub struct WalkingTrigger;

    impl BoolTrigger for WalkingTrigger {
        type Param<'w, 's> = Query<'w, 's, &'static KinematicVelocity>;

        fn trigger(&self, entity: Entity, param: Self::Param<'_, '_>) -> bool {
            param.get(entity).is_ok_and(|v| v.x.abs() > 5f32)
        }
    }

    #[derive(Copy, Clone, Debug, Reflect, PartialEq)]
    pub struct FallingTrigger;

    impl BoolTrigger for FallingTrigger {
        type Param<'w, 's> = Query<'w, 's, &'static KinematicVelocity>;

        fn trigger(&self, entity: Entity, param: Self::Param<'_, '_>) -> bool {
            param.get(entity).is_ok_and(|v| v.y < -5f32)
        }
    }

    #[derive(Copy, Clone, Debug, Reflect, PartialEq)]
    pub struct GroundedTrigger;

    impl BoolTrigger for GroundedTrigger {
        type Param<'w, 's> = Query<'w, 's, Option<&'static KinematicCharacterControllerOutput>>;

        fn trigger(&self, entity: Entity, param: Self::Param<'_, '_>) -> bool {
            param
                .get(entity)
                .is_ok_and(|o| o.is_some_and(|output| output.grounded))
        }
    }

    #[derive(Copy, Clone, Debug, Reflect, PartialEq)]
    pub struct JumpTrigger;

    impl BoolTrigger for JumpTrigger {
        type Param<'w, 's> = Query<'w, 's, &'static Jumper>;

        fn trigger(&self, entity: Entity, param: Self::Param<'_, '_>) -> bool {
            param.get(entity).is_ok_and(|j| j.should_jump())
        }
    }
}
