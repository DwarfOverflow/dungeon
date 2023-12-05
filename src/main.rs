use bevy::window::PrimaryWindow;
use bevy::{math::*, prelude::*};
use bevy::sprite::Anchor;
use bevy_pixel_camera::{
    PixelCameraPlugin, PixelZoom, PixelViewport
};
use std::fs;

mod level;
pub use crate::level::*;
mod simple_entities;
pub use crate::simple_entities::*;
mod player;
pub use crate::player::*;
mod ressource;
pub use crate::ressource::*;
mod tick;
pub use crate::tick::*;

// screen size
const LEFT: i32 = -450;
const RIGHT: i32 = 450;
const BOTTOM: i32 = -300;
const TOP: i32 = 300;

//wall
const LEFT_WALL: f32 = LEFT as f32;
const RIGHT_WALL: f32 = RIGHT as f32;
const BOTTOM_WALL: f32 = BOTTOM as f32;
const TOP_WALL: f32 = TOP as f32;

const WALL_THICKNESS: f32 = 10.0;
const WALL_BLOCK_WIDTH: f32 = RIGHT_WALL - LEFT_WALL;
const WALL_BLOCK_HEIGHT: f32 = TOP_WALL - BOTTOM_WALL;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

// Game Size
const SCREEN_GAME_X: i32 = 18;
const SCREEN_GAME_Y: i32 = 12;

// Animation
const ANIMATION_SPEED: f32 = 1.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin)
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.05)))
        .insert_resource(CurrentLevel { level: 0 })
        .insert_resource(BeginClick { position: None })
        .add_event::<ChangeLevelEvent>()
        .add_event::<TickEvent>()
        .add_event::<EndTickEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            change_level_event_listener,
            move_player, animate_entity,
            tick_event_listener,
            end_tick_event_listener
        ))
        .run();
}

#[derive(PartialEq)]
enum Direction {
    Left,
    Right,
    No,
    Bottom
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut change_level_event: EventWriter<ChangeLevelEvent>
) {
    commands.spawn((
        Camera2dBundle::default(),
        PixelZoom::FitSize {
            width: 900,
            height: 600,
        },
        PixelViewport,
    ));

    // Player
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("textures/entity/hero1.png"),
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            transform: Transform {
                translation: vec3(0., 0., 1.),
                scale: vec3(2., 2., 1.),
                ..Default::default()
            },
            ..Default::default()
        },
        Player { game_x: None, game_y: None, is_animating: false, direction: Direction::No },
    ));

    //walls
    {
        let vertical_wall_size = vec2(WALL_THICKNESS, WALL_BLOCK_HEIGHT + WALL_THICKNESS);
        let horizontal_wall_size = vec2(WALL_BLOCK_WIDTH + WALL_THICKNESS, WALL_THICKNESS);
        
        //left wall
        commands.spawn(SpriteBundle {
            transform: Transform {
                translation: vec3(LEFT_WALL, 0.0, 0.0),
                ..default()
            },
            sprite: Sprite {
                color: WALL_COLOR,
                custom_size: Some(vertical_wall_size),
                ..default()
            },
            ..default()
        }
    );

        //right wall
        commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: vec3(RIGHT_WALL, 0.0, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(vertical_wall_size),
                    ..default()
                },
                ..default()
            }
        );

        //bottom wall
        commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: vec3(0.0, BOTTOM_WALL, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(horizontal_wall_size),
                    ..default()
                },
                ..default()
            },
        );

        //top wall
        commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: vec3(0.0, TOP_WALL, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(horizontal_wall_size),
                    ..default()
                },
                ..default()
            });
    }

    change_level_event.send(ChangeLevelEvent);
}

fn move_player(
    mut player: Query<&mut Player>,
    mut player_transform: Query<&mut Transform, With<Player>>,

    blue_door_query: Query<&BlueDoor>,
    red_door_query: Query<&RedDoor>,

    buttons: Res<Input<MouseButton>>,
    mut begin_click: ResMut<BeginClick>,
    q_windows: Query<&Window, With<PrimaryWindow>>,

    mut change_level_event: EventWriter<ChangeLevelEvent>,
    mut tick_event: EventWriter<TickEvent>,

    input: Res<Input<KeyCode>>,
) {
    let mut player = player.single_mut();
    if player.game_x.is_none() || player.game_y.is_none() { return; }

    let (mouse_left, mouse_right, mouse_tap) = {
        if q_windows.single().cursor_position().is_none() { return; }
        let current_position = q_windows.single().cursor_position().unwrap();

        if buttons.just_pressed(MouseButton::Left) {
            begin_click.position = Some(current_position);
        }

        let mut mouse_tap = false;
        let mut mouse_left = false;
        let mut mouse_right = false;
        if begin_click.position.is_some() {
            let begin_click_position = begin_click.position.unwrap().clone();
            if buttons.just_released(MouseButton::Left) {
                // Voir si il y a un mouvement
                if begin_click_position.distance(current_position) > 100. { // scroll
                    if begin_click_position.x < current_position.x {
                        mouse_right = true;
                    } else {
                        mouse_left = true;
                    }
                } else { // click
                    mouse_tap = true;
                }
            }
        }

        (mouse_left, mouse_right, mouse_tap)
    };

    if (input.pressed(KeyCode::Left) || mouse_left) && !player.is_animating {
        player.move_with_direction(Direction::Left);
        tick_event.send(TickEvent);
    }
    else if (input.pressed(KeyCode::Right) || mouse_right) && !player.is_animating {
        player.move_with_direction(Direction::Right);
        tick_event.send(TickEvent);
    }
    else if (input.pressed(KeyCode::Up) || mouse_tap) && !player.is_animating {
        // Blue Door
        for door in blue_door_query.iter() {
            if door.game_x == player.game_x.unwrap() && door.game_y == player.game_y.unwrap() {
                // teleport player to other blue door 
                for tp_door in blue_door_query.iter() {
                    if door.game_x != tp_door.game_x && door.game_y != tp_door.game_y && chrono::Local::now().timestamp_millis() > 500 {
                        let mut player_transform = player_transform.single_mut();
                        player_transform.translation = player.move_without_animation(tp_door.game_x, tp_door.game_y).extend(0.);
                        tick_event.send(TickEvent);
                        return;
                    }
                }
            }
        }

        // Red Door
        let red_door = red_door_query.single();
        if red_door.game_x == player.game_x.unwrap() && red_door.game_y == player.game_y.unwrap() {
            change_level_event.send(ChangeLevelEvent);
            return;
        }
    }
}

fn animate_entity(
    mut player_query: Query<(&mut Transform, &mut Player, &mut Handle<Image>)>,
    asset_server: Res<AssetServer>,
    mut end_tick_event: EventWriter<EndTickEvent>,
) {
    let player_query = player_query.single_mut();
    let mut player_transform = player_query.0;
    let mut player = player_query.1;
    let mut player_handle = player_query.2;
    if player.game_x.is_none() || player.game_y.is_none() { return; }

    let result = player.animate(&player_transform.translation);
    player_transform.translation = result.0;
    let end_tick = result.1;

    let image_index = if chrono::Local::now().timestamp_millis() % 300 > 150 {1} else {2};

    if player.direction == Direction::Left {
        *player_handle = asset_server.load(format!("textures/entity/hero-left-{}.png", image_index));
    } 
    else if player.direction == Direction::Right {
        *player_handle = asset_server.load(format!("textures/entity/hero-right-{}.png", image_index));
    }
    else  {
        *player_handle = asset_server.load("textures/entity/hero1.png");
    }

    if end_tick {
        end_tick_event.send(EndTickEvent);
    }
}