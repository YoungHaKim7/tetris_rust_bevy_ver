use crate::components::{Piece, Position};
use crate::game_color::GameColor;
use crate::game_constants::{
    HEIGHT, LEVEL_TIMES, NUM_BLOCKS_X, NUM_BLOCKS_Y, NUM_LEVELS, TEXTURE_SIZE, TITLE, WIDTH,
};
use crate::game_types::{GameMap, PieceMatrix, PieceType, Presence};
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use rand::{Rng, rng};
use std::time::Duration;

mod components;
mod game_color;
mod game_constants;
mod game_types;

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
        .insert_resource(Time::<Fixed>::from_seconds(2.0))
        .init_state::<GameState>()
        .add_systems(
            Startup,
            (
                setup_camera,
                spawn_initial_piece,
                setup_ui,
                setup_game_over_ui,
                update_gravity_speed,
            ),
        ) // Add setup_game_over_ui here
        .add_systems(
            Update,
            (
                handle_input,
                draw_blocks,
                clear_lines,
                update_score_display,
                update_gravity_speed,
                update_level_display,
                display_game_over_message.run_if(in_state(GameState::GameOver)),
            ),
        ) // Add update_level_display here
        .add_systems(
            FixedUpdate,
            move_piece_down.run_if(in_state(GameState::Playing)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_piece(commands: &mut Commands, game_map: &GameMap, game_state: &mut NextState<GameState>) {
    let new_piece = Piece::random();
    let initial_position = Position {
        x: NUM_BLOCKS_X as isize / 2 - 1,
        y: 0,
    };

    if can_move(&new_piece, &initial_position, initial_position.y, &game_map) {
        commands.spawn((new_piece, initial_position));
        println!("Spawned new piece");
    } else {
        println!("Game Over! Cannot spawn new piece.");
        game_state.set(GameState::GameOver);
    }
}

fn spawn_initial_piece(
    mut commands: Commands,
    game_map: Res<GameMap>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    spawn_piece(&mut commands, &game_map, &mut game_state);
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
                        (x as f32 * TEXTURE_SIZE as f32) - (WIDTH as f32 / 2.0)
                            + (TEXTURE_SIZE as f32 / 2.0),
                        (HEIGHT as f32 / 2.0)
                            - (y as f32 * TEXTURE_SIZE as f32)
                            - (TEXTURE_SIZE as f32 / 2.0),
                        0.0,
                    ),
                    ..default()
                });
            }
        }
    }

    // Draw current piece blocks
    if let Ok((piece, position)) = query_piece.get_single() {
        let piece_matrix = get_block_matrix(piece.states[piece.current_state], piece.color);
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
                            ((position.x + mx as isize) as f32 * TEXTURE_SIZE as f32)
                                - (WIDTH as f32 / 2.0)
                                + (TEXTURE_SIZE as f32 / 2.0),
                            (HEIGHT as f32 / 2.0)
                                - ((position.y + my as isize) as f32 * TEXTURE_SIZE as f32)
                                - (TEXTURE_SIZE as f32 / 2.0),
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
fn get_block_matrix(num: u16, color: GameColor) -> PieceMatrix {
    let mut res = [[Presence::No; 4]; 4];
    for i in 0..16 {
        if num & (1u16 << (15 - i)) > 0 {
            res[i / 4][i % 4] = Presence::Yes(color);
        }
    }
    res
}

fn move_piece_down(
    mut commands: Commands,
    mut query_piece: Query<(Entity, &mut Piece, &mut Position)>,
    mut game_map: ResMut<GameMap>, // Make game_map mutable
    mut game_state: ResMut<NextState<GameState>>,
) {
    if let Ok((entity, piece, mut position)) = query_piece.get_single_mut() {
        let new_y = position.y + 1;
        if can_move(&piece, &position, new_y, &game_map) {
            position.y = new_y;
            println!("Piece moved down to y: {}", position.y);
        } else {
            // Collision detected, finalize piece placement
            let piece_matrix = get_block_matrix(piece.states[piece.current_state], piece.color);
            for my in 0..4 {
                for mx in 0..4 {
                    if let Presence::Yes(color) = piece_matrix[my][mx] {
                        let map_x = position.x + mx as isize;
                        let map_y = position.y + my as isize;
                        if map_x >= 0
                            && map_x < NUM_BLOCKS_X as isize
                            && map_y >= 0
                            && map_y < NUM_BLOCKS_Y as isize
                        {
                            game_map.0[map_y as usize][map_x as usize] = Presence::Yes(color);
                        }
                    }
                }
            }
            commands.entity(entity).despawn(); // Despawn the piece entity
            spawn_piece(&mut commands, &game_map, &mut game_state);
            println!("Piece landed at y: {}", position.y);
            println!("Piece finalized and added to game map.");
        }
    }
}

// Helper function to check if a piece can move to a new position
fn can_move(piece: &Piece, current_pos: &Position, new_y: isize, game_map: &GameMap) -> bool {
    let piece_matrix = get_block_matrix(piece.states[piece.current_state], piece.color);
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
        let mut rng = rng();
        let piece_type = match rng.random_range(0..7) {
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

fn can_rotate(piece: &Piece, current_pos: &Position, game_map: &GameMap) -> bool {
    let piece_matrix = get_block_matrix(piece.states[piece.current_state], piece.color);
    for my in 0..4 {
        for mx in 0..4 {
            if let Presence::Yes(_) = piece_matrix[my][mx] {
                let block_x = current_pos.x + mx as isize;
                let block_y = current_pos.y + my as isize;

                // Check collision with boundaries
                if block_x < 0
                    || block_x >= NUM_BLOCKS_X as isize
                    || block_y < 0
                    || block_y >= NUM_BLOCKS_Y as isize
                {
                    return false;
                }

                // Check collision with existing blocks on the game map
                if let Presence::Yes(_) = game_map.0[block_y as usize][block_x as usize] {
                    return false;
                }
            }
        }
    }
    true
}

fn can_move_horizontally(
    piece: &Piece,
    current_pos: &Position,
    new_x: isize,
    game_map: &GameMap,
) -> bool {
    let piece_matrix = get_block_matrix(piece.states[piece.current_state], piece.color);
    for my in 0..4 {
        for mx in 0..4 {
            if let Presence::Yes(_) = piece_matrix[my][mx] {
                let block_x = new_x + mx as isize;
                let block_y = current_pos.y + my as isize;

                // Check collision with side boundaries
                if block_x < 0 || block_x >= NUM_BLOCKS_X as isize {
                    return false;
                }

                // Check collision with existing blocks on the game map
                if block_y >= 0
                    && block_y < NUM_BLOCKS_Y as isize
                    && block_x >= 0
                    && block_x < NUM_BLOCKS_X as isize
                {
                    if let Presence::Yes(_) = game_map.0[block_y as usize][block_x as usize] {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn handle_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Position, &mut Piece)>,
    mut game_map: ResMut<GameMap>,
    mut score: ResMut<Score>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if let Ok((entity, mut position, mut piece)) = query.get_single_mut() {
        if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::ArrowLeft) {
            let new_x = position.x - 1;
            if can_move_horizontally(&piece, &position, new_x, &game_map) {
                position.x = new_x;
            }
        }
        if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::ArrowRight) {
            let new_x = position.x + 1;
            if can_move_horizontally(&piece, &position, new_x, &game_map) {
                position.x = new_x;
            }
        }
        if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::ArrowDown) {
            let new_y = position.y + 1;
            if can_move(&piece, &position, new_y, &game_map) {
                position.y = new_y;
            }
        }

        if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::Space) {
            println!("Space key pressed");
            let mut final_y = position.y;
            while can_move(&piece, &position, final_y + 1, &game_map) {
                final_y += 1;
            }

            if final_y > position.y {
                score.value += (final_y - position.y) as u32;
                position.y = final_y;
            }

            // Lock the piece
            let piece_matrix = get_block_matrix(piece.states[piece.current_state], piece.color);
            for my in 0..4 {
                for mx in 0..4 {
                    if let Presence::Yes(color) = piece_matrix[my][mx] {
                        let map_x = position.x + mx as isize;
                        let map_y = position.y + my as isize;
                        if map_x >= 0
                            && map_x < NUM_BLOCKS_X as isize
                            && map_y >= 0
                            && map_y < NUM_BLOCKS_Y as isize
                        {
                            game_map.0[map_y as usize][map_x as usize] = Presence::Yes(color);
                        }
                    }
                }
            }
            commands.entity(entity).despawn();
            spawn_piece(&mut commands, &game_map, &mut game_state);
        }

        if keyboard_input.just_pressed(bevy::input::keyboard::KeyCode::ArrowUp) {
            let old_state = piece.current_state;
            let next_state = (piece.current_state + 1) % 4;
            let next_state_clone = next_state.clone();
            let mut rotated_piece = piece.clone();
            rotated_piece.current_state = next_state_clone;

            if can_rotate(&rotated_piece, &position, &game_map) {
                piece.current_state = next_state;
            } else {
                // If rotation causes collision, revert to old state
                piece.current_state = old_state;
            }
        }
    }
}

// New system to clear full lines
fn clear_lines(mut game_map: ResMut<GameMap>, mut score: ResMut<Score>, mut level: ResMut<Level>) {
    // Add level as a parameter
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
    for &row_to_clear in rows_to_clear.iter().rev() {
        // Iterate in reverse to avoid index issues
        lines_cleared += 1;
        // Remove the full row
        game_map.0.remove(row_to_clear);
        // Add a new empty row at the top
        game_map.0.insert(0, vec![Presence::No; NUM_BLOCKS_X]);
    }

    if lines_cleared > 0 {
        score.value += lines_cleared as u32 * 100; // Example scoring: 100 points per line
        level.lines_cleared_in_level += lines_cleared as u32;
        if level.lines_cleared_in_level >= 10 {
            // Advance level every 10 lines
            level.value += 1;
            level.lines_cleared_in_level = 0;
        }
        println!(
            "Cleared {} lines! Current score: {}",
            lines_cleared, score.value
        );
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
                "
Level: ",
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
    let mut text_bundle = TextBundle::from_section(
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
    });

    text_bundle.visibility = Visibility::Hidden;

    commands.spawn((text_bundle, GameOverMessage));
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
fn update_gravity_speed(level: Res<Level>, mut fixed_time: ResMut<Time<Fixed>>) {
    if level.is_changed() {
        let level_index = level.value as usize;
        if level_index < NUM_LEVELS {
            let new_speed_ms = LEVEL_TIMES[level_index];
            let new_speed_secs = new_speed_ms as f32 / 1000.0;
            fixed_time.set_wrap_period(Duration::from_secs_f32(new_speed_secs));
            println!("Gravity speed updated to: {}s", new_speed_secs);
        }
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
