use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use crate::game_constants::{TITLE, WIDTH, HEIGHT, NUM_BLOCKS_X, NUM_BLOCKS_Y, TEXTURE_SIZE};
use crate::game_types::{GameMap, Presence, PieceMatrix};
use crate::components::{Piece, Position};
use crate::game_color::GameColor;

mod game_constants;
mod game_color;
mod game_types;
mod components;

fn main() {
    App::new()
        .insert_resource(ClearColor(GameColor::Gray.into()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: TITLE.into(),
                resolution: (WIDTH as f32, HEIGHT as f32).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<GameMap>()
        .add_systems(Startup, (setup_camera, spawn_initial_piece))
        .add_systems(Update, (handle_input, draw_blocks))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_initial_piece(mut commands: Commands) {
    commands.spawn((
        Piece {
            states: [0b0000_0110_0110_0000, 0, 0, 0],
            color: GameColor::Red,
            current_state: 0,
        },
        Position { x: NUM_BLOCKS_X as isize / 2 - 1, y: 0 },
    ));
    println!("Spawned initial piece (placeholder)");
}

fn draw_blocks(
    mut commands: Commands,
    game_map: Res<GameMap>,
    query_piece: Query<(&Piece, &Position)>,
    query_existing_blocks: Query<Entity, With<Sprite>>,
) {
    for entity in query_existing_blocks.iter() {
        commands.entity(entity).despawn();
    }

    for y in 0..NUM_BLOCKS_Y {
        for x in 0..NUM_BLOCKS_X {
            if let Presence::Yes(color) = game_map.0[y][x] {
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: color.into(),
                        custom_size: Some(Vec2::new(TEXTURE_SIZE as f32, TEXTURE_SIZE as f32)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        (x as f32 * TEXTURE_SIZE as f32) - (WIDTH as f32 / 2.0) + (TEXTURE_SIZE as f32 / 2.0),
                        (HEIGHT as f32 / 2.0) - (y as f32 * TEXTURE_SIZE as f32) - (TEXTURE_SIZE as f32 / 2.0),
                        0.0,
                    ),
                    ..default()
                });
            }
        }
    }

    if let Ok((piece, position)) = query_piece.get_single() {
        let piece_matrix = get_block_matrix(piece.states[piece.current_state]);
        for my in 0..4 {
            for mx in 0..4 {
                if let Presence::Yes(color) = piece_matrix[my][mx] {
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: color.into(),
                            custom_size: Some(Vec2::new(TEXTURE_SIZE as f32, TEXTURE_SIZE as f32)),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            ((position.x + mx as isize) as f32 * TEXTURE_SIZE as f32) - (WIDTH as f32 / 2.0) + (TEXTURE_SIZE as f32 / 2.0),
                            (HEIGHT as f32 / 2.0) - ((position.y + my as isize) as f32 * TEXTURE_SIZE as f32) - (TEXTURE_SIZE as f32 / 2.0),
                            0.0,
                        ),
                        ..default()
                    });
                }
            }
        }
    }
}

fn get_block_matrix(num: u16) -> PieceMatrix {
    let mut res = [[Presence::No; 4]; 4];
    for i in 0..16 {
        if num & (1u16 << (15 - i)) > 0 {
            res[i / 4][i % 4] = Presence::Yes(GameColor::Red);
        }
    }
    res
}

fn handle_input(keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::ArrowLeft) {
        println!("Left key pressed");
    }
    if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::ArrowRight) {
        println!("Right key pressed");
    }
    if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::ArrowUp) {
        println!("Up key pressed");
    }
    if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::ArrowDown) {
        println!("Down key pressed");
    }
    if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::Space) {
        println!("Space key pressed");
    }
}