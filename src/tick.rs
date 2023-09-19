use bevy::{ecs::{event::{Event, EventReader, EventWriter}, system::{ParamSet, Query}, query::With}, transform::components::Transform};

use crate::{Player, Wall, Direction, SCREEN_GAME_Y, Chest};

#[derive(Event)]
pub struct TickEvent;

pub fn tick_event_listener(
    mut events: ParamSet<(EventReader<TickEvent>, EventWriter<TickEvent>)>,
) {
    if events.p0().read().last().is_none() { return; }
}

#[derive(Event)]
pub struct EndTickEvent;

pub fn end_tick_event_listener(
    mut events: ParamSet<(EventReader<EndTickEvent>, EventWriter<TickEvent>)>,
    mut player: Query<&mut Player>,
    mut player_transform: Query<&mut Transform, With<Player>>,
    wall_query: Query<&Wall>,
    mut chest_query: Query<&mut Chest>,
) {
    if events.p0().read().last().is_none() { return; }

    // Gravity
    {
        let mut player = player.single_mut();
        if player.game_x.is_none() || player.game_y.is_none() || player.is_animating { return; }
        if wall_query.is_empty() { return; }

        let player_game_x = player.game_x.unwrap();
        let player_game_y = player.game_y.unwrap();

        
        let mut is_wall_under_player = false;
        for wall in wall_query.iter() {
            if player_game_x == wall.game_x && player_game_y-1 == wall.game_y {
                is_wall_under_player = true;
            }
        }

        if is_wall_under_player == false {
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
                    println!("spawn de monstre !");
                    chest.has_spawn = true; // Faire spawn le monstre :)
                }
            }
        }
    }
}