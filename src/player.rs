use bevy::{ecs::component::Component, math::{Vec2, vec2, vec3, Vec3}};

use crate::{Direction, SCREEN_GAME_X, TOP, RIGHT, ANIMATION_SPEED};
use crate::math::get_distance;

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