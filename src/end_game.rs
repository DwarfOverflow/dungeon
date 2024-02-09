use bevy::{app::{App, Plugin}, asset::AssetServer, ecs::{schedule::OnEnter, system::{Commands, Res}}, log::info, math::vec3, sprite::{Anchor, Sprite, SpriteBundle}, transform::components::Transform};

use crate::*;

pub struct EndGamePlugin;
impl Plugin for EndGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::End), start_end_screen);
    }
}

pub fn start_end_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("END SCREEN");

    // Big Player
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("textures/entity/hero1.png"),
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            transform: Transform {
                translation: vec3(-100., -350., 5.),
                scale: vec3(40., 40., 1.),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    // Big Chest
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("textures/object/chest-4.png"),
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            transform: Transform {
                translation: vec3(-400., -300., 5.),
                scale: vec3(10., 10., 1.),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    // GG
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("textures/decor/G.png"),
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            transform: Transform {
                translation: vec3(-400., 50., 5.),
                scale: vec3(10., 10., 1.),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("textures/decor/G.png"),
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            transform: Transform {
                translation: vec3(-250., 50., 5.),
                scale: vec3(10., 10., 1.),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}