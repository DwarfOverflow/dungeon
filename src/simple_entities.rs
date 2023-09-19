use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Wall {
    pub game_x: i32,
    pub game_y: i32
}

#[derive(Component, PartialEq)]
pub struct BlueDoor {
    pub game_x: i32,
    pub game_y: i32
}

#[derive(Component)]
pub struct RedDoor {
    pub game_x: i32,
    pub game_y: i32
}

#[derive(Component)]
pub struct Chest {
    pub game_x: i32,
    pub game_y: i32,
    pub is_open: bool,
    animation_index: i8,
    pub has_spawn: bool,
}

impl Chest {
    pub fn new(game_x: i32, game_y: i32) -> Chest {
        return Chest { game_x: game_x, game_y: game_y, is_open: false, has_spawn:false, animation_index:1}
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn animate(&mut self) -> Option<i8> {
        if self.is_open && self.animation_index < 4 && chrono::Local::now().timestamp_millis() % 10000 > 5000 {
            self.animation_index += 1; // Animation à améliorer !
            return Some(self.animation_index);
        }
        return None;
    }
}