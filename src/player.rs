use bevy::{ecs::component::Component, math::{Vec2, vec2, vec3, Vec3}};

use crate::{Direction, SCREEN_GAME_X, TOP, RIGHT, ANIMATION_SPEED};

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

        // Voir si à la fin du trajet
        if target.distance(current_position.truncate()) < ANIMATION_SPEED*2. {
            let was_animating = self.is_animating;
            self.is_animating = false;
            self.direction = Direction::No;
            if was_animating || self.has_change_pos {
                return (target.extend(1.), true);
            } else {
                return (target.extend(1.), false);
            }
        }
        self.has_change_pos = false;
        self.is_animating = true;
        
        // Calculer étape intermédiaire
        let angle = ((current_position.x-target.x)/target.distance(current_position.truncate())).asin();
        let temporary_position =  Vec2::new(
            -angle.sin()*ANIMATION_SPEED + current_position.x,
            -angle.cos()*ANIMATION_SPEED + current_position.y
        );

        // si il bouge sur l'axe des Y
        if vec2(target.x, 0.).distance(vec2(current_position.x, 0.)) < vec2(0., target.y).distance(vec2(0., current_position.y)) {
            self.direction = Direction::Bottom;
        }
        // Axe des X
        else {
            if target.x > current_position.x {
                self.direction = Direction::Right;
            } else {
                self.direction = Direction::Left;
            }
        }

        return  (temporary_position.extend(1.), false);
    }
}