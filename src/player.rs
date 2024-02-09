use bevy::{app::{App, Plugin}, asset::AssetServer, ecs::system::{Commands, Res}, math::{vec2, vec3, Vec2, Vec3}, sprite::{Anchor, Sprite, SpriteBundle}, transform::components::Transform};

use crate::*;
use crate::math::get_distance;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Game), spawn_player)
            .add_systems(Update, move_player.run_if(in_state(GameState::Game)));
    }
}

const FALLING_SPEED: f32 = 2.;

#[derive(Component)]
pub struct Player {
    pub game_x: Option<i32>,
    pub game_y: Option<i32>,
    pub is_animating: bool,
    pub(crate) direction: Direction,
    pub(crate) has_change_pos: bool,
}

impl Player {
    fn check_if_outdoor(&mut self) {
        if self.game_x.is_none() || self.game_y.is_none() { return; }

        while self.game_x.unwrap() < 0 {
            self.game_x = Some(self.game_x.unwrap() + 1);
        }
        while self.game_x.unwrap() >= SCREEN_GAME_X {
            self.game_x = Some(self.game_x.unwrap() - 1);
        }
    }

    pub(crate) fn move_with_direction(&mut self, direction: Direction) {
        if self.game_x.is_none() || self.game_y.is_none() { return; }

        match direction {
            Direction::Left => self.move_with_animation(self.game_x.unwrap()-1, self.game_y.unwrap()),
            Direction::Right => self.move_with_animation(self.game_x.unwrap()+1, self.game_y.unwrap()),
            Direction::Bottom => self.move_with_animation(self.game_x.unwrap(), self.game_y.unwrap()-1),
            _ => ()
        }
    }

    pub fn move_without_animation(&mut self, game_x: i32, game_y: i32) -> Vec2 {
        self.has_change_pos = true;
        self.game_x = Some(game_x);
        self.game_y = Some(game_y);

        self.check_if_outdoor();

        let res = vec2(9. + (game_x*50-RIGHT) as f32, (game_y*50-TOP) as f32);
        return res;
    }

    fn move_with_animation(&mut self, game_x: i32, game_y: i32) {
        self.has_change_pos = true;

        self.game_x = Some(game_x);
        self.game_y = Some(game_y);

        self.check_if_outdoor();
    }

    pub fn animate(&mut self, current_position: &Vec3) -> (Vec3, bool) {
        if self.game_x.is_none() || self.game_y.is_none() { return (vec3(0., 0., 0.), false); }

        let target = Vec2::new(9. + (self.game_x.unwrap()*50-RIGHT) as f32, (self.game_y.unwrap()*50-TOP) as f32);

        if current_position.x != target.x { // On le bouge sur l'axe des X
            self.is_animating = true;
            // Si la co X est proche de destination
            if get_distance(target.x, current_position.x) < ANIMATION_SPEED {
                return (vec3(target.x, current_position.y, 1.), false);
            }

            // sinon on bouge progressivement
            if current_position.x > target.x { // voir de quel coté aller
                self.direction = Direction::Left;
                return (vec3(current_position.x-ANIMATION_SPEED, current_position.y, 1.), false);
            } else {
                self.direction = Direction::Right;
                return (vec3(current_position.x+ANIMATION_SPEED, current_position.y, 1.), false);
            }
        } else {
            // si Y proche destination
            if get_distance(current_position.y, target.y) < ANIMATION_SPEED {
                self.direction = Direction::No;
                self.is_animating = false;
                return (target.extend(1.), true);
            }
            
            // sinon on bouge progressivement
            self.is_animating = true;
            if current_position.y > target.y { // voir de quel coté aller
                self.direction = Direction::Bottom;
                return (vec3(target.x, current_position.y-FALLING_SPEED, 1.), false);
            } else {
                self.direction = Direction::No;
                return (vec3(target.x, current_position.y+ANIMATION_SPEED, 1.), false);
            }
        }
    }
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
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