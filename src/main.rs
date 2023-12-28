use bevy::reflect::GetTupleField;
use bevy::window::PrimaryWindow;
use bevy::{math::*, prelude::*};
use bevy::sprite::Anchor;
use bevy_pixel_camera::{
    PixelCameraPlugin, PixelZoom, PixelViewport
};
use rand::prelude::*;

mod level;
pub use crate::level::*;
mod simple_entities;
pub use crate::simple_entities::*;
mod monster;
pub use crate::monster::Monster;
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
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
        )
        .add_plugins(PixelCameraPlugin)
        .init_asset::<LevelAsset>()
        .init_asset_loader::<LevelAssetLoader>()
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.05)))
        .insert_resource(CurrentLevel { level: 0 })
        .insert_resource(BeginClick { position: None })
        .init_resource::<LevelMaps>()
        .init_resource::<TexturesRessource>()
        .add_event::<ChangeLevelEvent>()
        .add_event::<TickEvent>()
        .add_event::<EndTickEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            change_level_event_listener,
            move_player, 
            animate_entity,
            tick_event_listener,
            end_tick_event_listener,
            send_maps_on_load
        ))
        .run();
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Right,
    No,
    Bottom
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut level_maps: ResMut<LevelMaps>,
    mut textures_ressource: ResMut<TexturesRessource>,
) {
    commands.spawn((
        Camera2dBundle::default(),
        PixelZoom::FitSize {
            width: 1200,
            height: 800,
        },
        PixelViewport,
    ));

    // load level assets
    for index in 0..NB_LEVEL {
        level_maps.maps = Vec::new();
        level_maps.maps_handle.push(asset_server.load(format!("map/level-{}.lev", index+1)));
    }

    { // load complex entities assets
        textures_ressource.player_center = asset_server.load("textures/entity/hero1.png");
        textures_ressource.player_left = (asset_server.load("textures/entity/hero-left-1.png"), asset_server.load("textures/entity/hero-left-2.png"));
        textures_ressource.player_right = (asset_server.load("textures/entity/hero-right-1.png"), asset_server.load("textures/entity/hero-right-2.png"));

        textures_ressource.bat_left = (asset_server.load("textures/entity/left-bat-1.png"), asset_server.load("textures/entity/left-bat-2.png"));
        textures_ressource.bat_right = (asset_server.load("textures/entity/right-bat-1.png"), asset_server.load("textures/entity/right-bat-2.png"));
    }

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
        Player { game_x: None, game_y: None, is_animating: false, direction: Direction::No, has_change_pos:false },
    ));

    { // side wall
        let wall_tex = asset_server.load("textures/walls/dungeon-wall.png");

        for game_y in 0..18 {
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

    { // Clouds
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
}

fn move_player(
    mut player: Query<&mut Player>,
    mut player_transform: Query<&mut Transform, With<Player>>,

    blue_door_query: Query<&BlueDoor>,
    red_door_query: Query<&RedDoor>,
    wall_query: Query<&Wall>,
    mut chest_query: Query<&mut Chest>,

    buttons: Res<Input<MouseButton>>,
    mut begin_click: ResMut<BeginClick>,
    q_windows: Query<&Window, With<PrimaryWindow>>,

    mut change_level_event: EventWriter<ChangeLevelEvent>,
    mut tick_event: EventWriter<TickEvent>,

    input: Res<Input<KeyCode>>,
) {
    let mut player = player.single_mut();
    if player.game_x.is_none() || player.game_y.is_none() { return; }

    // Obtenir les mouvements de souris
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

    // si le joueur est en train de tomber l'empecher de bouger
    let mut on_the_ground = false;
    for wall in wall_query.iter() {
        if wall.game_x == player.game_x.unwrap() && wall.game_y == player.game_y.unwrap()-1 {
            on_the_ground = true;
        }
    }
    if !on_the_ground { return; }

    // gerer les mouvements
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
        let mut all_chest_open = true;
        for chest in chest_query.iter() {
            if !chest.is_open {
                all_chest_open = false;
            }
        }
        if red_door.game_x == player.game_x.unwrap() && red_door.game_y == player.game_y.unwrap() && all_chest_open {
            change_level_event.send(ChangeLevelEvent {new_level:true});
            return;
        }

        // Chest
        for mut chest in chest_query.iter_mut() {
            if chest.game_x == player.game_x.unwrap() && chest.game_y == player.game_y.unwrap() {
                chest.open();
                return;
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct TexturesRessource {
    pub player_center: Handle<Image>,
    pub player_right: (Handle<Image>, Handle<Image>),
    pub player_left: (Handle<Image>, Handle<Image>),

    pub bat_right: (Handle<Image>, Handle<Image>), // never used but it save handle in memory
    pub bat_left: (Handle<Image>, Handle<Image>)   // same
}

fn animate_entity(
    mut queries: ParamSet<(
        Query<(&mut Transform, &mut Player, &mut Handle<Image>)>,
        Query<(&mut Chest, &mut Handle<Image>)>,
        Query<(&mut Transform, &mut Monster, &mut Handle<Image>)>
    )>,
    textures_ressource: Res<TexturesRessource>,
    asset_server: Res<AssetServer>,
    mut end_tick_event: EventWriter<EndTickEvent>,
) {
    // Player
    {
        let mut player_query = queries.p0();

        let player_query = player_query.single_mut();
        let mut player_transform = player_query.0;
        let mut player = player_query.1;
        let mut player_handle = player_query.2;
        if player.game_x.is_none() || player.game_y.is_none() { return; }

        let result = player.animate(&player_transform.translation);
        player_transform.translation = result.0;
        let end_tick = result.1;

        let image_index = if chrono::Local::now().timestamp_millis() % 300 > 150 {0} else {1};

        if player.direction == Direction::Left {
            //*player_handle = asset_server.load(format!("textures/entity/hero-left-{}.png", image_index));
            *player_handle = textures_ressource.player_left.get_field::<Handle<Image>>(image_index).unwrap().clone();
        } 
        else if player.direction == Direction::Right {
            *player_handle = textures_ressource.player_right.get_field::<Handle<Image>>(image_index).unwrap().clone();
        }
        else  {
            *player_handle = textures_ressource.player_center.clone();
        }

        if end_tick {
            end_tick_event.send(EndTickEvent);
        }
    }

    // Chest
    {
        let mut chest_query = queries.p1();

        if chest_query.is_empty() { return; }
        for mut chest in chest_query.iter_mut() {
            match chest.0.animate() {
                Some(t) => *chest.1 = asset_server.load(format!("textures/object/chest-{}.png", t)),
                _ => ()
            }
        }
    }

    { // Monster
        let mut monster_query = queries.p2();
        for monster in monster_query.iter_mut() {
            let mut monster_transform = monster.0;
            let mut monster_entity = monster.1;
            let mut monster_image = monster.2;

            monster_transform.translation = monster_entity.animate(&monster_transform.translation);

            let image_index = if chrono::Local::now().timestamp_millis() % 600 > 300 {1} else {2};
            if monster_entity.direction() == Direction::Left {
                *monster_image = asset_server.load(format!("textures/entity/left-bat-{}.png", image_index));
            } 
            else { // Right is the default direction
                *monster_image = asset_server.load(format!("textures/entity/right-bat-{}.png", image_index));
            }
        }
    }
}