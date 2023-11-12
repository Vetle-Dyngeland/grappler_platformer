use super::{Player, PlayerSet};
use bevy::{prelude::*, reflect::TypePath};
use leafwing_input_manager::{axislike::VirtualAxis, prelude::*};

pub(super) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<InputAction>::default())
            .add_systems(Startup, init.in_set(PlayerSet::Input));
    }
}

fn init(mut cmd: Commands, player_query: Query<Entity, With<Player>>) {
    cmd.entity(player_query.single())
        .insert(InputManagerBundle {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(VirtualAxis::horizontal_arrow_keys(), InputAction::Run)
                .insert(KeyCode::C, InputAction::Jump)
                //.insert(KeyCode::X, InputAction::Grapple)
                .insert(KeyCode::X, InputAction::Slingshot)
                .build(),
        });
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, TypePath)]
pub enum InputAction {
    Run,
    Jump,
    Grapple,
    Slingshot,
}
