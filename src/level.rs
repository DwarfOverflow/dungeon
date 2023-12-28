use bevy::{ecs::{event::{Event, EventReader}, system::{Commands, ResMut, Res, Query}, entity::Entity, query::With}, asset::{AssetServer, LoadContext, AssetLoader, io::Reader, AsyncReadExt}, transform::components::Transform, utils::BoxedFuture};
use serde::Deserialize;
use bevy::utils::thiserror;

use thiserror::Error;
use crate::*;

pub const NB_LEVEL: i32 = 3;

#[derive(Event)]
pub struct ChangeLevelEvent {
    pub new_level: bool,
}

pub fn change_level_event_listener(
    mut change_level_event: EventReader<ChangeLevelEvent>,
    mut level_res: ResMut<CurrentLevel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    custom_assets: ResMut<Assets<LevelAsset>>,
    mut player_query: Query<(&mut Transform, &mut Player)>,

    mut despawn_blue_door_query: Query<Entity, With<BlueDoor>>,
    mut despawn_red_door_query: Query<Entity, With<RedDoor>>,
    mut despawn_wall_query: Query<Entity, With<Wall>>,
    mut despawn_chest_query: Query<Entity, With<Chest>>,
    mut despawn_monster_query: Query<Entity, With<Monster>>,
) {
    match change_level_event.read().last() {
        None => return,
        Some(event) => {
            if event.new_level {
                level_res.level += 1;
            }
        }
    }
    let current_level = level_res.level;

    // destroy ancient level
    {
        if !despawn_blue_door_query.is_empty() {
            for entity in &mut despawn_blue_door_query { commands.entity(entity).despawn(); }
        }
        if !despawn_red_door_query.is_empty() {
            for entity in &mut despawn_red_door_query { commands.entity(entity).despawn(); }
        }
        if !despawn_chest_query.is_empty() {
            for entity in &mut despawn_chest_query { commands.entity(entity).despawn(); }
        }
        if !despawn_wall_query.is_empty() {
            for entity in &mut despawn_wall_query { commands.entity(entity).despawn(); }
        }
        if !despawn_monster_query.is_empty() {
            for entity in &mut despawn_monster_query { commands.entity(entity).despawn(); }
        }
    }

    // build new level
    {
        let wall_tex = asset_server.load("textures/walls/dungeon-wall.png");
        let blue_door_tex = asset_server.load("textures/walls/door-blue.png");
        let red_door_tex = asset_server.load("textures/walls/door-red.png");
        let chest_tex = asset_server.load("textures/object/chest-1.png");

        let level_map = {
            let handle: Handle<LevelAsset> = asset_server.load(format!("map/level-{}.lev", current_level));
            let custom_asset = custom_assets.get(&handle);
            let level_map = &custom_asset.unwrap().map;
            let level_map: Vec<&str> = level_map.split_whitespace().collect();
            level_map
        };

        let mut game_x;
        let mut game_y = level_map.len() as i32 -1;

        for line in level_map {
            game_x = 0;
            for block in line.chars() {
                let block_pos = vec2(25. +(game_x*50-RIGHT) as f32, 25. + (game_y*50-TOP) as f32);
                match block {
                    '1' => {
                        commands.spawn((
                            SpriteBundle {
                                texture: wall_tex.clone(),
                                transform: Transform {
                                    translation: block_pos.extend(0.),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1., 1., 1.),
                                    custom_size: Some(Vec2::new(50., 50.,)),
                                    ..default()
                                },
                                ..default()
                            },
                            Wall { game_x, game_y },
                        ));
                    }
                    '&' => {
                        let player_query = player_query.single_mut();
                        let mut player_transform = player_query.0;
                        let mut player = player_query.1;
                        player_transform.translation = player.move_without_animation(game_x, game_y).extend(0.);
                    }
                    'B' => {
                        commands.spawn((
                            SpriteBundle {
                                texture: blue_door_tex.clone(),
                                transform: Transform {
                                    translation: block_pos.extend(0.),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1., 1., 1.),
                                    custom_size: Some(Vec2::new(50., 50.,)),
                                    ..default()
                                },
                                ..default()
                            },
                            BlueDoor { game_x, game_y },
                        ));
                    }
                    'R' => {
                        commands.spawn((
                            SpriteBundle {
                                texture: red_door_tex.clone(),
                                transform: Transform {
                                    translation: block_pos.extend(0.),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1., 1., 1.),
                                    custom_size: Some(Vec2::new(50., 50.,)),
                                    ..default()
                                },
                                ..default()
                            },
                            RedDoor { game_x, game_y },
                        ));
                    }
                    'C' => {
                        let block_pos = Vec2::new(block_pos.x, block_pos.y-5.);
                        commands.spawn((
                            SpriteBundle {
                                texture: chest_tex.clone(),
                                transform: Transform {
                                    translation: block_pos.extend(0.),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1., 1., 1.),
                                    custom_size: Some(Vec2::new(50., 50.,)),
                                    ..default()
                                },
                                ..default()
                            },
                            Chest::new(game_x, game_y),
                        ));
                    }
                    _ => (),
                }
                game_x += 1;
            }
            game_y -= 1;
        }
    }
}

pub fn send_maps_on_load(mut level_maps: ResMut<LevelMaps>, custom_assets: ResMut<Assets<LevelAsset>>, mut change_level_event: EventWriter<ChangeLevelEvent>) {
    if level_maps.sended { return; }
    let mut maps = Vec::new();
    for map_handle in &level_maps.maps_handle {
        match custom_assets.get(map_handle) {
            Some(v) => maps.push(v.clone()),
            None => return
        }
    }

    change_level_event.send(ChangeLevelEvent {new_level:true});
    level_maps.sended = true;
}

#[derive(Resource, Default)]
pub struct LevelMaps {
    pub maps_handle: Vec<Handle<LevelAsset>>,
    pub maps: Vec<LevelAsset>,
    pub sended: bool,
}

#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct LevelAsset {
    pub map: String,
}

#[derive(Default)]
pub struct LevelAssetLoader;

/// Possible errors that can be produced by [`LevelAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LevelAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for LevelAssetLoader {
    type Asset = LevelAsset;
    type Settings = ();
    type Error = LevelAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let map = String::from_utf8(bytes).unwrap();
            let custom_asset = LevelAsset { map };
            Ok(custom_asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["lev"]
    }
}