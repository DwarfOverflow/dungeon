use bevy::{app::{App, Plugin, Update}, asset::AssetServer, ecs::{event::{Event, EventReader, EventWriter}, query::With, schedule::{common_conditions::in_state, IntoSystemConfigs}, system::{Commands, ParamSet, Query, Res}}, math::Vec2, prelude::default, render::color::Color, sprite::{Sprite, SpriteBundle}, transform::components::Transform};

use crate::*;

pub struct TickPlugin;
impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                tick_event_listener,
                end_tick_event_listener,
            ).run_if(in_state(GameState::Game)))
            .add_event::<TickEvent>()
            .add_event::<EndTickEvent>();
    }
}

#[derive(Event)]
pub struct TickEvent;

pub fn tick_event_listener(
    mut events: ParamSet<(EventReader<TickEvent>, EventWriter<TickEvent>)>,
    mut monsters: Query<&mut Monster>,
    wall_query: Query<&Wall>,
    player: Query<&Player>,
) {
    if events.p0().read().last().is_none() { return; }
    let player = player.single();
    if player.game_x.is_none() || player.game_y.is_none() { return; }
    let player_game_x = player.game_x.unwrap();

    { // move monster
        for mut monster in monsters.iter_mut() {
            monster.has_moved = false; // tous les monstres n'ont pas encore bougé
        }

        let mut ended = false;
        while ended == false && monsters.iter().len() != 0{
            println!("while ended == false:");
            let monsters_pos = { // liste statique de toutes les positions des monstres
                let mut monsters_pos: Vec<(i32, i32)> = Vec::new();
                monsters.iter().for_each(|e| {monsters_pos.push((e.game_x(), e.game_y()));});
                monsters_pos
            };

            for mut monster in monsters.iter_mut() {
                if !monster.has_moved {
                    let has_moved = move_monster(monster.as_mut(), player_game_x, &wall_query, &monsters_pos);
                    if has_moved {
                        monster.has_moved = true;
                        break;
                    }
                }
                ended = true;
            }
        }
    }
}

fn move_monster(
    monster: &mut Monster,
    player_game_x: i32,
    wall_query: &Query<'_, '_, &Wall>, 
    monsters_pos: &Vec<(i32, i32)>
) -> bool {
    if monster.game_x() > player_game_x {
        // vérifier qu'il n'y a pas de murs
        let mut can_go = true;
        for wall in wall_query.iter() {
            if wall.game_x == monster.game_x()-1 && wall.game_y == monster.game_y() {
                can_go = false;
                break;
            }
        }
        for other_monster in monsters_pos.iter() {
            if other_monster.0 == monster.game_x()-1 && other_monster.1 == monster.game_y() {
                can_go = false; // empecher un monstre de rentrer dans un autre
                break;
            }
        }
        if can_go {
            monster.move_with_direction(Direction::Left);
            return true;
        }
    }
    if monster.game_x() < player_game_x {
        // vérifier qu'il n'y a pas de murs
        let mut can_go = true;
        for wall in wall_query.iter() {
            if wall.game_x == monster.game_x()+1 && wall.game_y == monster.game_y() {
                can_go = false;
                break;
            }
        }
        for other_monster in monsters_pos.iter() {
            if other_monster.0 == monster.game_x()+1 && other_monster.1 == monster.game_y() {
                can_go = false;
                break;
            }
        }
        if can_go {
            monster.move_with_direction(Direction::Right);
            return true;
        }
    }
    return false;
}

#[derive(Event)]
pub struct EndTickEvent;

pub fn end_tick_event_listener(
    mut events: ParamSet<(EventReader<EndTickEvent>, EventWriter<TickEvent>, EventWriter<ChangeLevelEvent>)>,
    mut player: Query<&mut Player>,
    mut player_transform: Query<&mut Transform, With<Player>>,
    monster_query: Query<&Monster>,
    wall_query: Query<&Wall>,
    mut chest_query: Query<&mut Chest>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if events.p0().read().last().is_none() { return; }

    // Gravity
    {
        let mut player = player.single_mut();
        if player.game_x.is_none() || player.game_y.is_none() || player.is_animating { return; }
        if wall_query.is_empty() { return; }

        let player_game_x = player.game_x.unwrap();
        let player_game_y = player.game_y.unwrap();

        
        let mut is_something_under_player = false;
        for wall in wall_query.iter() {
            if player_game_x == wall.game_x && player_game_y-1 == wall.game_y {
                is_something_under_player = true;
            }
        }
        for monster in monster_query.iter() {
            if player_game_x == monster.game_x() && player_game_y-1 == monster.game_y() {
                is_something_under_player = true;
            }
        }

        if is_something_under_player == false {
            player.move_with_direction(Direction::Bottom);
            events.p1().send(TickEvent);
        }
    }

    // Quand le joueur tombe tout en bas le mettre en haut
    {
        let mut player = player.single_mut();
        let mut player_transform = player_transform.single_mut();
        if player.game_x.is_none() || player.game_y.is_none() || player.is_animating { return; }
        if wall_query.is_empty() { return; }

        let player_game_x = player.game_x.unwrap();
        let player_game_y = player.game_y.unwrap();

        if player_game_y < -1 {
            player_transform.translation = player.move_without_animation(player_game_x, SCREEN_GAME_Y-1).extend(0.);
            events.p1().send(TickEvent);
        }
    }

    { //spawn monster if needed
        let player = player.single();
        if player.game_x.is_none() || player.game_y.is_none() || player.is_animating { return; }

        if player.game_x.is_some() && player.game_y.is_some() {
            for mut chest in chest_query.iter_mut() {
                if chest.is_open && chest.has_spawn == false && !(player.game_x.unwrap() == chest.game_x && player.game_y.unwrap() == chest.game_y) {
                    chest.has_spawn = true;

                    let monster_tex = asset_server.load("textures/entity/left-bat-1.png");
                    commands.spawn((
                        SpriteBundle {
                            texture: monster_tex,
                            transform: Transform {
                                translation: Monster::get_translation(chest.game_x, chest.game_y).extend(0.),
                                ..default()
                            },
                            sprite: Sprite {
                                color: Color::rgb(1., 1., 1.),
                                custom_size: Some(Vec2::new(45., 45.,)),
                                ..default()
                            },
                            ..default()
                        },
                        Monster::new(chest.game_x, chest.game_y),
                    ));

                }
            }
        }
    }

    { // Regarder si le monstre percute le joueur
        let player = player.single();
        if player.game_x.is_none() || player.game_y.is_none() { return; }
        for monster in monster_query.iter() {
            if monster.game_x() == player.game_x.unwrap() && monster.game_y() == player.game_y.unwrap() {
                events.p2().send(ChangeLevelEvent { new_level: false });
            }
        }
    }
}