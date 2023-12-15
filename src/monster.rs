use bevy::{ecs::component::Component, math::{Vec3, Vec2}};

use crate::{Direction, TOP, RIGHT, ANIMATION_SPEED};

#[derive(Component)]
pub struct Monster {
    game_x: i32,
    game_y: i32,
    is_animating: bool,
    direction: Direction,
}

impl Monster {
    pub(crate) fn move_with_direction(&mut self, direction: Direction) {

        match direction {
            Direction::Left => self.move_with_animation(self.game_x-1, self.game_y),
            Direction::Right => self.move_with_animation(self.game_x+1, self.game_y),
            Direction::Bottom => self.move_with_animation(self.game_x, self.game_y-1),
            _ => ()
        }
    }

    fn move_with_animation(&mut self, game_x: i32, game_y: i32) {
        self.game_x = game_x;
        self.game_y = game_y;
    }

    pub fn animate(&mut self, current_position: &Vec3) -> Vec3 {

        let target = Monster::get_translation(self.game_x, self.game_y);

        // Voir si à la fin du trajet
        if target.distance(current_position.truncate()) < ANIMATION_SPEED*2. {
            self.is_animating = false;
            self.direction = Direction::No;
            return target.extend(0.);
        }
        self.is_animating = true;
        
        // Calculer étape intermédiaire
        let angle = ((current_position.x-target.x)/target.distance(current_position.truncate())).asin();
        let temporary_position =  Vec2::new(
            -angle.sin()*ANIMATION_SPEED + current_position.x,
            -angle.cos()*ANIMATION_SPEED + current_position.y
        );

        // Sens de l'animation
        if target.x > current_position.x {
            self.direction = Direction::Right;
        } else {
            self.direction = Direction::Left;
        }

        return  temporary_position.extend(0.);
    }

    pub(crate) fn new(game_x: i32, game_y: i32) -> Monster{
        return Monster { game_x: game_x, game_y:game_y, is_animating: false, direction: Direction::No };
    }

    pub(crate) fn get_translation(game_x: i32, game_y: i32) -> Vec2 {
        return Vec2::new(25. +(game_x*50-RIGHT) as f32, 25.+(game_y*50-TOP) as f32);
    }

    pub(crate) fn game_x(&self) -> i32 {
        return self.game_x;
    }

    pub(crate) fn game_y(&self) -> i32 {
        return self.game_y;
    }

    pub(crate) fn direction(&self) -> Direction {
        return self.direction;
    }
}