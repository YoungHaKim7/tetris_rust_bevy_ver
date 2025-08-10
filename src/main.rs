use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use crate::game_constants::{TITLE, WIDTH, HEIGHT, NUM_BLOCKS_X, NUM_BLOCKS_Y, TEXTURE_SIZE};
use crate::game_types::{GameMap, Presence, PieceMatrix, PieceType};
use crate::components::{Piece, Position};
use crate::game_color::GameColor;
use rand::Rng;
use rand::thread_rng;
use std::time::Duration;

mod game_constants;
mod game_color;
mod game_types;
mod components;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct Score {
    pub value: u32,
}

#[derive(Resource, Default)]
pub struct Level {
    pub value: u32,
    pub lines_cleared_in_level: u32,
}

// New marker component for score display
#[derive(Component)]
struct ScoreDisplay;

// New marker component for level display
#[derive(Component)]
struct LevelDisplay;

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
        .init_resource::<Score>() // Add Score resource
        .init_resource::<Level>() // Add Level resource
        .init_state::<GameState>()
        .add_systems(Startup, (setup_camera, spawn_initial_piece, setup_ui, setup_game_over_ui)) // Add setup_game_over_ui here
        .add_systems(Update, (handle_input, draw_blocks, clear_lines, update_score_display, update_gravity_speed, update_level_display)) // Add update_level_display here
        .add_systems(FixedUpdate, move_piece_down.run_if(in_state(GameState::Playing)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_initial_piece(mut commands: Commands, game_map: Res<GameMap>, mut game_state: ResMut<NextState<GameState>>) {
    let new_piece = Piece::random();
    let initial_position = Position { x: NUM_BLOCKS_X as isize / 2 - 1, y: 0 };

    if can_move(&new_piece, &initial_position, initial_position.y, &game_map) {
        commands.spawn(( 
            new_piece,
            initial_position,
        ));
        println!("Spawned initial piece (random)");
    } else {
        println!("Game Over! Cannot spawn new piece.");
        game_state.set(GameState::GameOver);
    }
}

// System to draw blocks
fn draw_blocks(
    mut commands: Commands,
    game_map: Res<GameMap>,
    query_piece: Query<(&Piece, &Position)>,
    query_existing_blocks: Query<Entity, With<Sprite>>,
) {
    // Despawn all existing block sprites to redraw
    for entity in query_existing_blocks.iter() {
        commands.entity(entity).despawn();
    }

    // Draw GameMap blocks
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

    // Draw current piece blocks
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

// Helper function to convert u16 to PieceMatrix (copied from original piece.rs)
fn get_block_matrix(num: u16) -> PieceMatrix {
    let mut res = [[Presence::No; 4]; 4];
    for i in 0..16 {
        if num & (1u16 << (15 - i)) > 0 {
            res[i / 4][i % 4] = Presence::Yes(GameColor::Red); // Default to Red for now, will use piece.color later
        }
    }
    res
}

fn move_piece_down(
    mut commands: Commands,
    mut query_piece: Query<(Entity, &mut Piece, &mut Position)>,
    mut game_map: ResMut<GameMap>, // Make game_map mutable
) {
    if let Ok((entity, piece, mut position)) = query_piece.get_single_mut() {
        let new_y = position.y + 1;
        if can_move(&piece, &position, new_y, &game_map) {
            position.y = new_y;
            println!("Piece moved down to y: {}", position.y);
        } else {
            // Collision detected, finalize piece placement
            let piece_matrix = get_block_matrix(piece.states[piece.current_state]);
            for my in 0..4 {
                for mx in 0..4 {
                    if let Presence::Yes(color) = piece_matrix[my][mx] {
                        let map_x = position.x + mx as isize;
                        let map_y = position.y + my as isize;
                        if map_x >= 0 && map_x < NUM_BLOCKS_X as isize && map_y >= 0 && map_y < NUM_BLOCKS_Y as isize {
                            game_map.0[map_y as usize][map_x as usize] = Presence::Yes(color);
                        }
                    }
                }
            }
            commands.entity(entity).despawn(); // Despawn the piece entity
            // TODO: Trigger line clearing
            commands.spawn(( // Spawn new piece
                Piece::random(),
                Position { x: NUM_BLOCKS_X as isize / 2 - 1, y: 0 },
            ));
            println!("Piece landed at y: {}", position.y);
            println!("Piece finalized and added to game map.");
        }
    }
}

// Helper function to check if a piece can move to a new position
fn can_move(piece: &Piece, current_pos: &Position, new_y: isize, game_map: &GameMap) -> bool {
    let piece_matrix = get_block_matrix(piece.states[piece.current_state]);
    for my in 0..4 {
        for mx in 0..4 {
            if let Presence::Yes(_) = piece_matrix[my][mx] {
                let block_x = current_pos.x + mx as isize;
                let block_y = new_y + my as isize;

                // Check collision with bottom boundary
                if block_y >= NUM_BLOCKS_Y as isize {
                    return false;
                }

                // Check collision with existing blocks on the game map
                if block_x >= 0 && block_x < NUM_BLOCKS_X as isize && block_y >= 0 {
                    if let Presence::Yes(_) = game_map.0[block_y as usize][block_x as usize] {
                        return false;
                    }
                }
            }
        }
    }
    true
}

// From<PieceType> for Piece implementation
impl From<PieceType> for Piece {
    fn from(piece_type: PieceType) -> Piece {
        use self::PieceType::*;

        let def = Piece::default();

        match piece_type {
            L => Piece {
                states: [17504, 1856, 1570, 736],
                color: GameColor::Orange,
                ..def
            },
            J => Piece {
                states: [8800, 1136, 1604, 3616],
                color: GameColor::Blue,
                ..def
            },
            S => Piece {
                states: [17952, 1728, 17952, 1728],
                color: GameColor::Green,
                ..def
            },
            Z => Piece {
                states: [9792, 3168, 9792, 3168],
                color: GameColor::Red,
                ..def
            },
            T => Piece {
                states: [17984, 3648, 19520, 19968],
                color: GameColor::Purple,
                ..def
            },
            I => Piece {
                states: [17476, 3840, 17476, 3840],
                color: GameColor::Cyan,
                ..def
            },
            O => Piece {
                states: [1632, 1632, 1632, 1632],
                color: GameColor::Yellow,
                ..def
            },
        }
    }
}

impl Piece {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let piece_type = match rng.gen_range(0..7) {
            0 => PieceType::L,
            1 => PieceType::J,
            2 => PieceType::S,
            3 => PieceType::Z,
            4 => PieceType::T,
            5 => PieceType::I,
            _ => PieceType::O,
        };
        Piece::from(piece_type)
    }
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

// New system to clear full lines
fn clear_lines(mut game_map: ResMut<GameMap>, mut score: ResMut<Score>, mut level: ResMut<Level>) { // Add level as a parameter
    let mut lines_cleared = 0;
    let mut rows_to_clear = Vec::new();

    // Find full lines
    for y in 0..NUM_BLOCKS_Y {
        let mut is_full = true;
        for x in 0..NUM_BLOCKS_X {
            if let Presence::No = game_map.0[y][x] {
                is_full = false;
                break;
            }
        }
        if is_full {
            rows_to_clear.push(y);
        }
    }

    // Clear lines and shift down
    for &row_to_clear in rows_to_clear.iter().rev() { // Iterate in reverse to avoid index issues
        lines_cleared += 1;
        // Remove the full row
        game_map.0.remove(row_to_clear);
        // Add a new empty row at the top
        game_map.0.insert(0, vec![Presence::No; NUM_BLOCKS_X]);
    }

    if lines_cleared > 0 {
        score.value += lines_cleared as u32 * 100; // Example scoring: 100 points per line
        level.lines_cleared_in_level += lines_cleared as u32;
        if level.lines_cleared_in_level >= 10 { // Advance level every 10 lines
            level.value += 1;
            level.lines_cleared_in_level = 0;
        }
        println!("Cleared {} lines! Current score: {}", lines_cleared, score.value);
    }
}

// New system to set up UI
fn setup_ui(mut commands: Commands) {
    commands.spawn(( 
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 40.0,
                color: Color::WHITE,
                ..default()
            }),
            TextSection::new(
                "\nLevel: ",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 40.0,
                color: Color::WHITE,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        ScoreDisplay,
        LevelDisplay,
    ));
}

// New system to update score display
fn update_score_display(score: Res<Score>, mut query_text: Query<&mut Text, With<ScoreDisplay>>) {
    if score.is_changed() {
        if let Some(mut text) = query_text.iter_mut().next() {
            text.sections[1].value = score.value.to_string();
        }
    }
}

// Component to mark the game over message
#[derive(Component)]
struct GameOverMessage;

// New system to set up Game Over UI
fn setup_game_over_ui(mut commands: Commands) {
    commands.spawn(( 
        TextBundle::from_section(
            "GAME OVER",
            TextStyle {
                font_size: 100.0,
                color: Color::RED,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            left: Val::Percent(20.0),
            ..default()
        }),
        GameOverMessage,
    ));
}

// New system to display Game Over message
fn display_game_over_message(
    game_state: Res<State<GameState>>,
    mut query_game_over_message: Query<&mut Visibility, With<GameOverMessage>>,
) {
    if game_state.get() == &GameState::GameOver {
        if let Some(mut visibility) = query_game_over_message.iter_mut().next() {
            *visibility = Visibility::Visible;
        }
    }
}

// New system to update gravity speed based on level
fn update_gravity_speed(
    level: Res<Level>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    if level.is_changed() {
        let new_speed = 1.0 - (level.value as f32 * 0.05); // Example: speed up by 5% per level
        fixed_time.set_wrap_period(Duration::from_secs_f32(new_speed.max(0.1))); // Minimum speed
        println!("Gravity speed updated to: {}s", new_speed.max(0.1));
    }
}

// New system to update level display
fn update_level_display(level: Res<Level>, mut query_text: Query<&mut Text, With<LevelDisplay>>) {
    if level.is_changed() {
        if let Some(mut text) = query_text.iter_mut().next() {
            text.sections[3].value = level.value.to_string(); // Accessing index 3 for Level value
        }
    }
}
