use bevy::{ecs::{event::{Event, EventReader}, system::{Commands, ResMut, Res, Query}, entity::Entity, query::With}, asset::AssetServer, transform::components::Transform};

use crate::*;

#[derive(Event)]
pub struct ChangeLevelEvent;

pub fn change_level_event_listener(
    mut events: EventReader<ChangeLevelEvent>,
    mut level_res: ResMut<CurrentLevel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&mut Transform, &mut Player)>,

    mut despawn_blue_door_query: Query<Entity, With<BlueDoor>>,
    mut despawn_red_door_query: Query<Entity, With<RedDoor>>,
    mut despawn_wall_query: Query<Entity, With<Wall>>,
    mut despawn_chest_query: Query<Entity, With<Chest>>,
) {
    if events.read().last().is_none() { return; }
    level_res.level += 1;
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
    }

    // build new level
    {
        let wall_tex = asset_server.load("textures/walls/dungeon-wall.png");
        let blue_door_tex = asset_server.load("textures/walls/door-blue.png");
        let red_door_tex = asset_server.load("textures/walls/door-red.png");
        let chest_tex = asset_server.load("textures/object/chest-1.png");

        let level_map = fs::read_to_string(format!("assets/map/level-{}", current_level))
            .expect("Erreur... Nous n'avons pas pu trouver le fichier de niveau.");
        // il faudra changer ça pour la version web
        
        let level_map: Vec<&str> = level_map.split_whitespace().collect();

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
