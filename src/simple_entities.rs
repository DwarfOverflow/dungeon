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
    game_x: i32,
    game_y: i32,
    is_open: bool
}

impl Chest {
    pub fn new(game_x: i32, game_y: i32) -> Chest {
        return Chest { game_x: game_x, game_y: game_y, is_open: false }
    }
}