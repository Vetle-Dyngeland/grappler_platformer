use crate::sprites::SpriteSheets;

use super::{Player, PlayerSet};
use bevy::prelude::*;

pub(super) struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init.in_set(PlayerSet::Visuals));
    }
}

fn init(mut cmd: Commands, player_query: Query<Entity, With<Player>>, sheets: Res<SpriteSheets>) {
    let atlas = &sheets.0.get("player_sheet").unwrap().atlas;
    let idx = sheets
        .0
        .get("player_sheet")
        .unwrap()
        .info
        .get("p1_stand")
        .unwrap()
        .0;

    cmd.entity(player_query.single()).insert((
        TextureAtlasSprite {
            custom_size: Some((25f32, 50f32).into()),
            index: idx,
            ..Default::default()
        },
        atlas.clone(),
    ));
}
