use bevy::reflect::GetTupleField;
use bevy::window::PrimaryWindow;
use bevy::{math::*, prelude::*};
use bevy_pixel_camera::PixelCameraPlugin;

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
        .add_plugins(PlayerPlugin)
        .add_plugins(LevelPlugin)
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.05)))
        .insert_resource(BeginClick { position: None })
        .init_resource::<TexturesRessource>()
        .add_state::<GameState>()
        .add_event::<TickEvent>()
        .add_event::<EndTickEvent>()
        .add_systems(Update, (
            animate_entity,
            tick_event_listener,
            end_tick_event_listener,
        ).run_if(in_state(GameState::Game)))
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