use bevy::ecs::system::WithEntity;
use bevy::window::PrimaryWindow;
use bevy::{math::*, prelude::*};
use bevy::sprite::Anchor;
use bevy_pixel_camera::{
    PixelCameraPlugin, PixelZoom, PixelViewport
};
use std::fs;

// screen size
const LEFT: i32 = -450;
const RIGHT: i32 = 450;
const BOTTOM: i32 = -300;
const TOP: i32 = 300;

//wall
const LEFT_WALL: f32 = LEFT as f32;
const RIGHT_WALL: f32 = RIGHT as f32;
const BOTTOM_WALL: f32 = BOTTOM as f32;
const TOP_WALL: f32 = TOP as f32;

const WALL_THICKNESS: f32 = 10.0;
const WALL_BLOCK_WIDTH: f32 = RIGHT_WALL - LEFT_WALL;
const WALL_BLOCK_HEIGHT: f32 = TOP_WALL - BOTTOM_WALL;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

// Game Size
const SCREEN_GAME_X: i32 = 18;
const SCREEN_GAME_Y: i32 = 12;

// Animation
const ANIMATION_SPEED: f32 = 3.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin)
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.05)))
        .insert_resource(CurrentLevel { level: 0 })
        .insert_resource(BeginClick { position: None })
        .add_event::<ChangeLevelEvent>()
        .add_event::<TickEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            change_level_event_listener,
            move_player, animate_entity,
            tick_event_listener,
        ))
        .run();
}

#[derive(Event)]
struct ChangeLevelEvent;

#[derive(Event)]
struct TickEvent;

#[derive(Resource, Clone, Copy)]
struct BeginClick {
    position: Option<Vec2>
}

#[derive(Resource, Clone, Copy)]
struct CurrentLevel {
    level: usize,
}

#[derive(Component)]
struct Wall {
    game_x: i32,
    game_y: i32
}

#[derive(Component, PartialEq)]
struct BlueDoor {
    game_x: i32,
    game_y: i32
}

#[derive(Component)]
struct RedDoor {
    game_x: i32,
    game_y: i32
}

#[derive(Component)]
struct Chest {
    game_x: i32,
    game_y: i32,
    is_open: bool
}

impl Chest {
    fn new(game_x: i32, game_y: i32) -> Chest {
        return Chest { game_x: game_x, game_y: game_y, is_open: false }
    }
}

#[derive(Component)]
struct Player {
    game_x: Option<i32>,
    game_y: Option<i32>,
    is_animating: bool,
    direction: Direction
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

    fn move_with_direction(&mut self, direction: Direction) {
        if direction == Direction::Left {
            self.move_with_animation(self.game_x.unwrap()-1, self.game_y.unwrap());
        }
        else if direction == Direction::Right {
            self.move_with_animation(self.game_x.unwrap()+1, self.game_y.unwrap());
        }
    }

    fn move_without_animation(&mut self, game_x: i32, game_y: i32) -> Vec2 {
        self.game_x = Some(game_x);
        self.game_y = Some(game_y);

        self.check_if_outdoor();

        let res = vec2(9. + (game_x*50-RIGHT) as f32, (game_y*50-TOP) as f32);
        return res;
    }

    fn move_with_animation(&mut self, game_x: i32, game_y: i32) {
        self.game_x = Some(game_x);
        self.game_y = Some(game_y);

        self.check_if_outdoor();
    }

    fn animate(&mut self, current_position: &Vec3) -> Vec3 {
        if self.game_x.is_none() || self.game_y.is_none() { return vec3(0., 0., 0.); }

        let target = Vec2::new(9. + (self.game_x.unwrap()*50-RIGHT) as f32, (self.game_y.unwrap()*50-TOP) as f32);

        if target.distance(current_position.truncate()) < ANIMATION_SPEED {
            self.is_animating = false;
            self.direction = Direction::No;
            return target.extend(0.);
        }

        self.is_animating = true;

        // si il bouge sur l'axe des Y
        if target.x == current_position.x {
            if target.y > current_position.y {
                return vec3(current_position.x, current_position.y+ANIMATION_SPEED, 0.);
            } else {
                return vec3(current_position.x, current_position.y-ANIMATION_SPEED, 0.);
            }
        }

        // Axe des X
        else {
            if target.x > current_position.x {
                self.direction = Direction::Right;
                return vec3(current_position.x+ANIMATION_SPEED, current_position.y, 0.);
            } else {
                self.direction = Direction::Left;
                return vec3(current_position.x-ANIMATION_SPEED, current_position.y, 0.);
            }
        }
    }
}

#[derive(PartialEq)]
enum Direction {
    Left,
    Right,
    No,
}


fn tick_event_listener(
    mut event: EventReader<TickEvent>,
) {
    if event.read().last().is_none() { return; }
    println!("tick !");
}

fn change_level_event_listener(
    mut events: EventReader<ChangeLevelEvent>,
    mut level_res: ResMut<CurrentLevel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    mut despawn_blue_door_query: Query<Entity, With<BlueDoor>>,
    mut despawn_red_door_query: Query<Entity, With<RedDoor>>,
    mut despawn_wall_query: Query<Entity, With<Wall>>,
    mut despawn_chest_query: Query<Entity, With<Chest>>,
) {
    if events.read().last().is_none() { return; }
    level_res.level += 1;
    let current_level = level_res.level;

    // destroy ancient level
    {
        if !despawn_blue_door_query.is_empty() {
            for entity in &mut despawn_blue_door_query { commands.entity(entity).despawn(); }
        }
        if !despawn_red_door_query.is_empty() {
            for entity in &mut despawn_red_door_query { commands.entity(entity).despawn(); }
        }
        if !despawn_chest_query.is_empty() {
            for entity in &mut despawn_chest_query { commands.entity(entity).despawn(); }
        }
        if !despawn_wall_query.is_empty() {
            for entity in &mut despawn_wall_query { commands.entity(entity).despawn(); }
        }
    }

    // build new level
    {
        let wall_tex = asset_server.load("textures/walls/dungeon-wall.png");
        let blue_door_tex = asset_server.load("textures/walls/door-blue.png");
        let red_door_tex = asset_server.load("textures/walls/door-red.png");
        let chest_tex = asset_server.load("textures/object/chest-1.png");

        let level_map = fs::read_to_string(format!("assets/map/level-{}", current_level))
            .expect("Erreur... Nous n'avons pas pu trouver le fichier de niveau.");
        // il faudra changer ça pour la version web
        
        let level_map: Vec<&str> = level_map.split_whitespace().collect();

        let mut game_x;
        let mut game_y = level_map.len() as i32 -1;

        for line in level_map {
            game_x = 0;
            for block in line.chars() {
                let block_pos = vec2(25. +(game_x*50-RIGHT) as f32, 25. + (game_y*50-TOP) as f32);
                match block {
                    '1' => {
                        commands.spawn((
                            SpriteBundle {
                                texture: wall_tex.clone(),
                                transform: Transform {
                                    translation: block_pos.extend(0.),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1., 1., 1.),
                                    custom_size: Some(Vec2::new(50., 50.,)),
                                    ..default()
                                },
                                ..default()
                            },
                            Wall { game_x, game_y },
                        ));
                    }
                    '&' => {
                        let player_query = player_query.single_mut();
                        let mut player_transform = player_query.0;
                        let mut player = player_query.1;
                        player_transform.translation = player.move_without_animation(game_x, game_y).extend(0.);
                    }
                    'B' => {
                        commands.spawn((
                            SpriteBundle {
                                texture: blue_door_tex.clone(),
                                transform: Transform {
                                    translation: block_pos.extend(0.),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1., 1., 1.),
                                    custom_size: Some(Vec2::new(50., 50.,)),
                                    ..default()
                                },
                                ..default()
                            },
                            BlueDoor { game_x, game_y },
                        ));
                    }
                    'R' => {
                        commands.spawn((
                            SpriteBundle {
                                texture: red_door_tex.clone(),
                                transform: Transform {
                                    translation: block_pos.extend(0.),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1., 1., 1.),
                                    custom_size: Some(Vec2::new(50., 50.,)),
                                    ..default()
                                },
                                ..default()
                            },
                            RedDoor { game_x, game_y },
                        ));
                    }
                    'C' => {
                        let block_pos = Vec2::new(block_pos.x, block_pos.y-5.);
                        commands.spawn((
                            SpriteBundle {
                                texture: chest_tex.clone(),
                                transform: Transform {
                                    translation: block_pos.extend(0.),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1., 1., 1.),
                                    custom_size: Some(Vec2::new(50., 50.,)),
                                    ..default()
                                },
                                ..default()
                            },
                            Chest::new(game_x, game_y),
                        ));
                    }
                    _ => (),
                }
                game_x += 1;
            }
            game_y -= 1;
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut change_level_event: EventWriter<ChangeLevelEvent>
) {
    commands.spawn((
        Camera2dBundle::default(),
        PixelZoom::FitSize {
            width: 900,
            height: 600,
        },
        PixelViewport,
    ));

    // Player
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
        Player { game_x: None, game_y: None, is_animating: false, direction: Direction::No },
    ));

    //walls
    {
        let vertical_wall_size = vec2(WALL_THICKNESS, WALL_BLOCK_HEIGHT + WALL_THICKNESS);
        let horizontal_wall_size = vec2(WALL_BLOCK_WIDTH + WALL_THICKNESS, WALL_THICKNESS);
        
        //left wall
        commands.spawn(SpriteBundle {
            transform: Transform {
                translation: vec3(LEFT_WALL, 0.0, 0.0),
                ..default()
            },
            sprite: Sprite {
                color: WALL_COLOR,
                custom_size: Some(vertical_wall_size),
                ..default()
            },
            ..default()
        }
    );

        //right wall
        commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: vec3(RIGHT_WALL, 0.0, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(vertical_wall_size),
                    ..default()
                },
                ..default()
            }
        );

        //bottom wall
        commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: vec3(0.0, BOTTOM_WALL, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(horizontal_wall_size),
                    ..default()
                },
                ..default()
            },
        );

        //top wall
        commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: vec3(0.0, TOP_WALL, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(horizontal_wall_size),
                    ..default()
                },
                ..default()
            });
    }

    change_level_event.send(ChangeLevelEvent);
}

fn move_player(
    mut player: Query<&mut Player>,
    mut player_transform: Query<&mut Transform, With<Player>>,

    blue_door_query: Query<&BlueDoor>,
    red_door_query: Query<&RedDoor>,

    buttons: Res<Input<MouseButton>>,
    mut begin_click: ResMut<BeginClick>,
    q_windows: Query<&Window, With<PrimaryWindow>>,

    mut change_level_event: EventWriter<ChangeLevelEvent>,
    mut tick_event: EventWriter<TickEvent>,

    input: Res<Input<KeyCode>>,
) {
    let mut player = player.single_mut();
    if player.game_x.is_none() || player.game_y.is_none() { return; }

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

    if (input.pressed(KeyCode::Left) || mouse_left) && !player.is_animating {
        player.move_with_direction(Direction::Left);
        tick_event.send(TickEvent);
    }
    else if (input.pressed(KeyCode::Right) || mouse_right) && !player.is_animating {
        player.move_with_direction(Direction::Right);
        tick_event.send(TickEvent);
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
        if red_door.game_x == player.game_x.unwrap() && red_door.game_y == player.game_y.unwrap() {
            println!("RD");
            change_level_event.send(ChangeLevelEvent);
            return;
        }
    }
}

fn animate_entity(
    mut player_query: Query<(&mut Transform, &mut Player, &mut Handle<Image>)>,
    asset_server: Res<AssetServer>,
) {
    let player_query = player_query.single_mut();
    let mut player_transform = player_query.0;
    let mut player = player_query.1;
    let mut player_handle = player_query.2;
    if player.game_x.is_none() || player.game_y.is_none() { return; }

    player_transform.translation = player.animate(&player_transform.translation);

    let image_index = if chrono::Local::now().timestamp_millis() % 300 > 150 {1} else {2};

    if player.direction == Direction::Left {
        *player_handle = asset_server.load(format!("textures/entity/hero-left-{}.png", image_index));
    } 
    else if player.direction == Direction::Right {
        *player_handle = asset_server.load(format!("textures/entity/hero-right-{}.png", image_index));
    }
    else if player.direction == Direction::No {
        *player_handle = asset_server.load("textures/entity/hero1.png");
    }
}