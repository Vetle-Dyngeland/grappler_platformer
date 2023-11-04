use std::{collections::HashMap, error::Error};

use bevy::prelude::*;

pub(super) struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpriteSheets::default())
            .add_systems(PreStartup, load);
    }
}

fn load(
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut sheets: ResMut<SpriteSheets>,
) {
    let png = "sprites/player_sheet.png";
    let csv = "assets/sprites/player_sheet.csv";
    let handle = asset_server.load(png);
    let mut atlas = TextureAtlas::new_empty(handle, Vec2::new(561f32, 604f32));
    let data = get_csv_data(csv).unwrap_or_else(|_| panic!(""));
    let mut map = HashMap::new();

    for (i, (name, transform)) in data.iter().enumerate() {
        map.insert(name.clone(), SpriteInfo(i));
        let transform = transform.iter().map(|u| *u as f32).collect::<Vec<f32>>();
        atlas.add_texture(Rect::new(
            transform[0],
            transform[1],
            transform[0] + transform[2],
            transform[1] + transform[3],
        ));
    }
    let atlas_handle = atlases.add(atlas);
    sheets.0.insert(
        "player_sheet".to_string(),
        SpriteSheet {
            info: map,
            atlas: atlas_handle,
        },
    );
}

#[derive(Resource, Default)]
pub struct SpriteSheets(pub HashMap<String, SpriteSheet>);

/// To be used in a hashmap (HashMap<String (*the name*), SpriteSheet>)
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteSheet {
    pub info: HashMap<String, SpriteInfo>,
    pub atlas: Handle<TextureAtlas>,
}

/// To be used in a hashmap (HashMap<String (*the name*), SpriteInfo>)
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteInfo(pub usize);

fn get_csv_data(path: &str) -> Result<Vec<(String, [u32; 4])>, Box<dyn Error>> {
    let mut data = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .flexible(false)
        .from_path(path)?;
    for result in rdr.records().map(|r| {
        let mut result = r?.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let name = result.remove(0);
        let transform = result
            .iter()
            .map(|d| d.parse::<u32>().unwrap())
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap_or_else(|v: Vec<u32>| {
                panic!("Expected an array of length 4, but got {}", v.len())
            });

        Ok::<(String, [u32; 4]), Box<dyn Error>>((name, transform))
    }) {
        data.push(result?)
    }

    Ok(data)
}
