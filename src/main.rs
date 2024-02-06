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
mod end_game;
pub use crate::end_game::*;
mod math;
pub use crate::math::*;
mod setup;
pub use crate::setup::*;

// screen size
const RIGHT: i32 = 450;
const TOP: i32 = 300;

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
                    title: "Dungeon - Game of Thrones Adventure".to_owned(),
                    ..default()
                }),
                ..default()
            })
        )
        .add_plugins(PixelCameraPlugin)
        .add_plugins(SetupPlugin)
        .init_asset::<LevelAsset>()
        .init_asset_loader::<LevelAssetLoader>()
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.05)))
        .insert_resource(CurrentLevel { level: 0 })
        .insert_resource(BeginClick { position: None })
        .init_resource::<LevelMaps>()
        .init_resource::<TexturesRessource>()
        .add_state::<GameState>()
        .add_event::<ChangeLevelEvent>()
        .add_event::<TickEvent>()
        .add_event::<EndTickEvent>()
        .add_systems(Update, (
            change_level_event_listener,
            move_player, 
            animate_entity,
            tick_event_listener,
            end_tick_event_listener,
            send_maps_on_load
        ).run_if(in_state(GameState::Game)))
        .add_systems(OnEnter(GameState::Game), spawn_player)
        .add_systems(OnEnter(GameState::End), start_end_screen)
        .run();
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Game,
    End
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Right,
    No,
    Bottom
}

fn move_player(
    mut player: Query<&mut Player>,
    mut player_transform: Query<&mut Transform, With<Player>>,

    monster_query: Query<&Monster>,
    blue_door_query: Query<&BlueDoor>,
    red_door_query: Query<&RedDoor>,
    wall_query: Query<&Wall>,
    mut chest_query: Query<&mut Chest>,
    mut button_query: Query<Entity, With<StartButton>>,

    buttons: Res<Input<MouseButton>>,
    mut begin_click: ResMut<BeginClick>,
    q_windows: Query<&Window, With<PrimaryWindow>>,

    mut change_level_event: EventWriter<ChangeLevelEvent>,
    mut tick_event: EventWriter<TickEvent>,

    input: Res<Input<KeyCode>>,
    mut commands: Commands,
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

    // si click faire disparaitre le bouton click to start
    if mouse_tap && !button_query.is_empty() {
        for button in &mut button_query {
            commands.entity(button).despawn();
        }
    }


    // si le joueur est en train de tomber l'empecher de bouger
    let mut on_the_ground = false;
    for wall in wall_query.iter() {
        if wall.game_x == player.game_x.unwrap() && wall.game_y == player.game_y.unwrap()-1 {
            on_the_ground = true;
        }
    }
    for monster in monster_query.iter() {
        if player.game_x.unwrap() == monster.game_x() && player.game_y.unwrap()-1 == monster.game_y() {
            on_the_ground = true;
        }
    }
    if !on_the_ground { return; }

    // gerer les mouvements
    if (input.pressed(KeyCode::Left) || mouse_left) && !player.is_animating {
        // vérifier qu'il n'y a pas de murs
        let mut can_go = true;
        for wall in wall_query.iter() {
            if wall.game_x == player.game_x.unwrap()-1 && wall.game_y == player.game_y.unwrap() {
                can_go = false;
                break;
            }
        }
        if can_go {
            player.move_with_direction(Direction::Left);
            tick_event.send(TickEvent);
        }
    }
    else if (input.pressed(KeyCode::Right) || mouse_right) && !player.is_animating {
        // vérifier qu'il n'y a pas de murs
        let mut can_go = true;
        for wall in wall_query.iter() {
            if wall.game_x == player.game_x.unwrap()+1 && wall.game_y == player.game_y.unwrap() {
                can_go = false;
                break;
            }
        }
        if can_go {
            player.move_with_direction(Direction::Right);
            tick_event.send(TickEvent);
        }
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
                tick_event.send(TickEvent);
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
    pub bat_left: (Handle<Image>, Handle<Image>),   // same

    pub chest_open: (Handle<Image>, Handle<Image>, Handle<Image>), // same
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