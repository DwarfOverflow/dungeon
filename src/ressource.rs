use bevy::{ecs::system::Resource, math::Vec2};

#[derive(Resource, Clone, Copy)]
pub struct BeginClick {
    pub position: Option<Vec2>
}