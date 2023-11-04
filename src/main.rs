use bevy::{
    app::PluginGroupBuilder,
    prelude::*,
    window::{PresentMode, WindowMode},
};
use bevy_editor_pls::EditorPlugin;
use bevy_rapier2d::prelude::*;
use seldom_state::StateMachinePlugin;

pub mod level;
pub mod sprites;
pub mod player;

pub const DEBUG: bool = true;

pub const fn debug() -> bool {
    DEBUG
}

struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(sprites::SpritesPlugin)
            .add(level::LevelPlugin)
            .add(player::PlayerPlugin)
    }
}

struct OtherPlugins;

impl PluginGroup for OtherPlugins {
    fn build(self) -> PluginGroupBuilder {
        let builder = PluginGroupBuilder::start::<Self>()
            .add(StateMachinePlugin::default())
            .add(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100f32));
        if DEBUG {
            builder.add(EditorPlugin::default())
        } else {
            builder
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Platformer".to_string(),
                        mode: WindowMode::BorderlessFullscreen,
                        resolution: (1980f32, 1080f32).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            GamePlugins,
            OtherPlugins,
        ))
        .run();
}
