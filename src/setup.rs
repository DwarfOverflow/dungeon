use bevy::{math::{vec2, vec3}, prelude::*, sprite::Anchor};
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use rand::Rng;

use crate::{GameState, LevelMaps, StartButton, TexturesRessource, NB_LEVEL, RIGHT, TOP};

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Game), (
                build_side_wall,
                spawn_cloud,
                spawn_start_buttons,
            ))
            .add_systems(Startup, (
                load_level_maps,
                load_entity_assets,
                spawn_camera,
            ));
    }
}

fn spawn_camera(mut commands: Commands) {
    println!("spawn camera");
    commands.spawn((
        Camera2dBundle::default(),
        PixelZoom::FitSize {
            width: 1200,
            height: 800,
        },
        PixelViewport,
    ));
}

fn load_level_maps(
    asset_server: Res<AssetServer>,
    mut level_maps: ResMut<LevelMaps>,
) {
    println!("load level map");
    for index in 0..NB_LEVEL {
        level_maps.maps = Vec::new();
        level_maps.maps_handle.push(asset_server.load(format!("map/level-{}.lev", index+1)));
    }
}

fn load_entity_assets(
    asset_server: Res<AssetServer>,
    mut textures_ressource: ResMut<TexturesRessource>,
){
    println!("load entity assets");
    textures_ressource.player_center = asset_server.load("textures/entity/hero1.png");
    textures_ressource.player_left = (asset_server.load("textures/entity/hero-left-1.png"), asset_server.load("textures/entity/hero-left-2.png"));
    textures_ressource.player_right = (asset_server.load("textures/entity/hero-right-1.png"), asset_server.load("textures/entity/hero-right-2.png"));

    textures_ressource.bat_left = (asset_server.load("textures/entity/left-bat-1.png"), asset_server.load("textures/entity/left-bat-2.png"));
    textures_ressource.bat_right = (asset_server.load("textures/entity/right-bat-1.png"), asset_server.load("textures/entity/right-bat-2.png"));

    textures_ressource.chest_open = (asset_server.load("textures/object/chest-2.png"), asset_server.load("textures/object/chest-3.png"), asset_server.load("textures/object/chest-4.png"))
}

fn build_side_wall(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    println!("build side wall");
    let wall_tex = asset_server.load("textures/walls/dungeon-wall.png");

    for game_y in -2..20 {
        commands.spawn(SpriteBundle { // left
                texture: wall_tex.clone(),
                transform: Transform {
                    translation: vec3(25. +(-1*50-RIGHT) as f32, 25. + (game_y*50-TOP) as f32, 0.),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(1., 1., 1.),
                    custom_size: Some(Vec2::new(50., 50.,)),
                    ..default()
                },
                ..default()
        });

        commands.spawn(SpriteBundle { // right
            texture: wall_tex.clone(),
            transform: Transform {
                translation: vec3(25. +(18*50-RIGHT) as f32, 25. + (game_y*50-TOP) as f32, 0.),
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(1., 1., 1.),
                custom_size: Some(Vec2::new(50., 50.,)),
                ..default()
            },
            ..default()
        });
    } 
}

fn spawn_cloud(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    println!("spawn cloud");
    let mut rng = rand::thread_rng();

    for for_x in 0..1000 {
        let x = for_x-500;

        // Bottom clouds
        for for_y in 0..100 {
            let y = for_y-430;
            
            let spawning_chance = ({if for_y < 30 {0.003} else {0.0007}} + {if for_x < 95 || for_x > 905 {0.0005} else {0.003}}) / 4.;

            if rng.gen::<f32>() < spawning_chance {
                commands.spawn(SpriteBundle {
                    texture: asset_server.load("textures/decor/cloud.png"),
                    transform: Transform {
                        translation: vec3(x as f32, y as f32, 2.0),
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(vec2(150., 150.)),
                        ..default()
                    },
                    ..default()
                });
            }
        }

        // Top clouds
        for for_y in 0..100 {
            let y = for_y+350;
            
            let spawning_chance = ({if for_y < 30 {0.003} else {0.0007}} + {if for_x < 95 || for_x > 905 {0.0005} else {0.003}}) / 4.;

            if rng.gen::<f32>() < spawning_chance {
                commands.spawn(SpriteBundle {
                    texture: asset_server.load("textures/decor/cloud.png"),
                    transform: Transform {
                        translation: vec3(x as f32, y as f32, 2.0),
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(vec2(150., 150.)),
                        ..default()
                    },
                    ..default()
                });
            }
        }
    }
}

fn spawn_start_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    println!("spawn start button");
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("textures/decor/start-button.png"),
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            transform: Transform {
                translation: vec3(-260., -100., 5.),
                scale: vec3(0.7, 0.7, 1.),
                ..Default::default()
            },
            ..Default::default()
        },
        StartButton {},
    ));
}