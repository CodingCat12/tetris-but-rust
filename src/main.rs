use bevy::color::palettes::tailwind::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use bevy::window::WindowResolution;
use rand::distr::StandardUniform;
use rand::prelude::*;

const BLOCK_SIZE: f32 = 32.0;

const GRID_HEIGHT: i32 = 22;
const GRID_WIDTH: i32 = 10;

const TETROMINOS: [TetrominoKind; 7] = [
    TetrominoKind::I,
    TetrominoKind::O,
    TetrominoKind::T,
    TetrominoKind::S,
    TetrominoKind::Z,
    TetrominoKind::J,
    TetrominoKind::L,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rotation {
    North,
    South,
    West,
    East,
}

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Component)]
struct ScoreText;

#[rustfmt::skip]
#[allow(clippy::type_complexity)]
const TETROMINO_SHAPES: [(TetrominoKind, [(Rotation, [IVec2; 4]); 4]); 7] = [
    (
        TetrominoKind::I,
        [
            (Rotation::North, [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(2, 0)]),
            (Rotation::East,  [ivec2(1, 1), ivec2(1, 0), ivec2(1, -1), ivec2(1, -2)]),
            (Rotation::South, [ivec2(-1, -1), ivec2(0, -1), ivec2(1, -1), ivec2(2, -1)]),
            (Rotation::West,  [ivec2(0, 1), ivec2(0, 0), ivec2(0, -1), ivec2(0, -2)]),
        ],
    ),
    (
        TetrominoKind::O,
        [
            (Rotation::North, [ivec2(0, 0), ivec2(1, 0), ivec2(0, -1), ivec2(1, -1)]),
            (Rotation::East,  [ivec2(0, 0), ivec2(1, 0), ivec2(0, -1), ivec2(1, -1)]),
            (Rotation::South, [ivec2(0, 0), ivec2(1, 0), ivec2(0, -1), ivec2(1, -1)]),
            (Rotation::West,  [ivec2(0, 0), ivec2(1, 0), ivec2(0, -1), ivec2(1, -1)]),
        ],
    ),
    (
        TetrominoKind::T,
        [
            (Rotation::North, [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(0, 1)]),
            (Rotation::East,  [ivec2(0, 1), ivec2(0, 0), ivec2(0, -1), ivec2(1, 0)]),
            (Rotation::South, [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(0, -1)]),
            (Rotation::West,  [ivec2(0, 1), ivec2(0, 0), ivec2(0, -1), ivec2(-1, 0)]),
        ],
    ),
    (
        TetrominoKind::S,
        [
            (Rotation::North, [ivec2(0, 0), ivec2(1, 0), ivec2(-1, -1), ivec2(0, -1)]),
            (Rotation::East,  [ivec2(0, 1), ivec2(0, 0), ivec2(1, 0), ivec2(1, -1)]),
            (Rotation::South, [ivec2(0, 0), ivec2(1, 0), ivec2(-1, -1), ivec2(0, -1)]),
            (Rotation::West,  [ivec2(0, 1), ivec2(0, 0), ivec2(1, 0), ivec2(1, -1)]),
        ],
    ),
    (
        TetrominoKind::Z,
        [
            (Rotation::North, [ivec2(-1, 0), ivec2(0, 0), ivec2(0, -1), ivec2(1, -1)]),
            (Rotation::East,  [ivec2(1, 1), ivec2(1, 0), ivec2(0, 0), ivec2(0, -1)]),
            (Rotation::South, [ivec2(-1, 0), ivec2(0, 0), ivec2(0, -1), ivec2(1, -1)]),
            (Rotation::West,  [ivec2(1, 1), ivec2(1, 0), ivec2(0, 0), ivec2(0, -1)]),
        ],
    ),
    (
        TetrominoKind::J,
        [
            (Rotation::North, [ivec2(0, 1), ivec2(0, 0), ivec2(0, -1), ivec2(-1, -1)]),
            (Rotation::East,  [ivec2(-1, 1), ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0)]),
            (Rotation::South, [ivec2(1, 1), ivec2(0, 1), ivec2(0, 0), ivec2(0, -1)]),
            (Rotation::West,  [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(1, -1)]),
        ],
    ),
    (
        TetrominoKind::L,
        [
            (Rotation::North, [ivec2(0, 1), ivec2(0, 0), ivec2(0, -1), ivec2(1, -1)]),
            (Rotation::East,  [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(-1, -1)]),
            (Rotation::South, [ivec2(-1, 1), ivec2(0, 1), ivec2(0, 0), ivec2(0, -1)]),
            (Rotation::West,  [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(1, 1)]),
        ],
    ),
];

fn main() {
    App::new()
        .add_event::<Tick>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1920.0, 1280.0),
                title: "Tetris but Rust".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (send_tick, gravity, clear_lines).chain())
        .add_systems(
            Update,
            (
                update_sprites,
                handle_movement,
                handle_rotation,
                ghost_piece,
                update_score_text,
            ),
        )
        .run();
}

#[derive(Resource)]
struct Ticker(Timer);

#[derive(Component)]
struct Active;

#[derive(Resource, Default)]
struct Grid {
    tiles: HashMap<IVec2, Color>,
}

#[derive(Event, Default)]
struct Tick;

#[derive(Component, Clone)]
struct Tetromino {
    position: IVec2,
    color: Color,
    kind: TetrominoKind,
    rotation: Rotation,
}

impl Tetromino {
    fn new() -> Self {
        let kind = rand::random::<TetrominoKind>();
        Tetromino {
            position: IVec2::new(GRID_WIDTH / 2, GRID_HEIGHT),
            color: kind.color(),
            kind,
            rotation: Rotation::North,
        }
    }

    fn move_left(&mut self) {
        self.position.x -= 1;
    }

    fn move_right(&mut self) {
        self.position.x += 1;
    }

    fn move_up(&mut self) {
        self.position.y += 1;
    }

    fn move_down(&mut self) {
        self.position.y -= 1;
    }

    fn rotate_left(&mut self) {
        self.rotation = match self.rotation {
            Rotation::North => Rotation::West,
            Rotation::West => Rotation::South,
            Rotation::South => Rotation::East,
            Rotation::East => Rotation::North,
        }
    }

    fn rotate_right(&mut self) {
        self.rotation = match self.rotation {
            Rotation::North => Rotation::East,
            Rotation::East => Rotation::South,
            Rotation::South => Rotation::West,
            Rotation::West => Rotation::North,
        }
    }

    fn occupied_tiles(&self) -> [IVec2; 4] {
        for (kind, rest) in TETROMINO_SHAPES {
            if kind != self.kind {
                continue;
            }

            for (rotation, tiles) in rest {
                if rotation != self.rotation {
                    continue;
                }

                return tiles.map(|position| position + self.position);
            }
        }

        unreachable!()
    }

    fn is_in_ground(&self) -> bool {
        self.occupied_tiles().iter().any(|&tile_pos| tile_pos.y < 0)
    }

    fn is_in_wall(&self) -> bool {
        self.occupied_tiles()
            .iter()
            .any(|&tile_pos| tile_pos.x < 0 || tile_pos.x >= GRID_WIDTH)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TetrominoKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Distribution<TetrominoKind> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TetrominoKind {
        *TETROMINOS.choose(rng).unwrap()
    }
}

impl TetrominoKind {
    const fn color(&self) -> Color {
        match *self {
            TetrominoKind::I => Color::Srgba(CYAN_500),
            TetrominoKind::O => Color::Srgba(YELLOW_300),
            TetrominoKind::T => Color::Srgba(PURPLE_700),
            TetrominoKind::S => Color::Srgba(GREEN_500),
            TetrominoKind::Z => Color::Srgba(RED_600),
            TetrominoKind::J => Color::Srgba(BLUE_700),
            TetrominoKind::L => Color::Srgba(ORANGE_500),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut event_writer: EventWriter<Tick>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);

    commands.insert_resource(Ticker(Timer::from_seconds(0.5, TimerMode::Repeating)));
    commands.init_resource::<Grid>();
    commands.init_resource::<Score>();

    commands.spawn((Tetromino::new(), Active));

    commands.insert_resource(ClearColor(Color::from(NEUTRAL_950)));
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            commands.spawn((
                Sprite {
                    color: Color::from(ZINC_900),
                    custom_size: Some(Vec2::splat(BLOCK_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(
                    x as f32 * BLOCK_SIZE - GRID_WIDTH as f32 / 2.0 * BLOCK_SIZE,
                    y as f32 * BLOCK_SIZE - GRID_HEIGHT as f32 / 2.0 * BLOCK_SIZE,
                    -2.0,
                ),
            ));
        }
    }

    // Initial tick
    event_writer.write_default();

    let mut ui_root = commands.spawn(Node {
        padding: UiRect::all(Val::Px(32.0)),
        ..default()
    });

    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 40.0,
        ..default()
    };

    ui_root.with_child((
        Text::new("Score: "),
        text_font.clone(),
        children![(TextSpan::default(), text_font.clone(), ScoreText)],
    ));
}

fn update_score_text(mut query: Query<&mut TextSpan, With<ScoreText>>, score: Res<Score>) {
    if let Ok(mut text) = query.single_mut() {
        **text = score.0.to_string()
    }
}

fn send_tick(
    mut ticker: ResMut<Ticker>,
    time: Res<Time<Fixed>>,
    mut event_writer: EventWriter<Tick>,
) {
    ticker.0.tick(time.delta());

    if ticker.0.finished() {
        event_writer.write_default();
    }
}

fn gravity(
    mut active_tetromino: Query<(&mut Tetromino, Entity), With<Active>>,
    mut grid: ResMut<Grid>,
    mut commands: Commands,
    mut event_reader: EventReader<Tick>,
) {
    for _tick in event_reader.read() {
        if let Ok((mut tetromino, entity)) = active_tetromino.single_mut() {
            let mut new_tetromino = tetromino.clone();
            new_tetromino.move_down();
            let color = tetromino.color;

            let mut is_in_other_tile = false;
            for pos in new_tetromino.occupied_tiles() {
                if grid.tiles.contains_key(&pos) {
                    is_in_other_tile = true;
                    break;
                }
            }

            if !new_tetromino.is_in_ground() && !is_in_other_tile {
                *tetromino = new_tetromino;
            } else {
                for IVec2 { x, y } in tetromino.occupied_tiles() {
                    grid.tiles.insert(IVec2::new(x, y), color);
                }

                commands.entity(entity).despawn();

                commands.spawn((Tetromino::new(), Active));
            }
        }
    }
}

#[derive(Component)]
struct Redraw;

fn ghost_piece(
    mut commands: Commands,
    active_tetromino: Query<&Tetromino, With<Active>>,
    inactive_tetromino: Query<Entity, (With<Tetromino>, Without<Active>)>,
    grid: Res<Grid>,
) {
    for entity in inactive_tetromino {
        commands.entity(entity).despawn();
    }

    if let Ok(active) = active_tetromino.single().cloned() {
        let mut ghost_tetromino = Tetromino {
            color: Color::from(GRAY_500),
            ..active
        };

        while !ghost_tetromino
            .occupied_tiles()
            .iter()
            .any(|tile| grid.tiles.contains_key(tile))
            && !ghost_tetromino.is_in_ground()
        {
            ghost_tetromino.move_down();
        }

        ghost_tetromino.move_up();

        commands.spawn(ghost_tetromino);
    }
}

fn update_sprites(
    mut commands: Commands,
    tetrominoes: Query<(&Tetromino, Option<&Active>)>,
    sprites: Query<Entity, With<Redraw>>,
    grid: Res<Grid>,
) {
    for sprite in sprites {
        commands.entity(sprite).despawn();
    }

    for (tetromino, active) in tetrominoes {
        let color = tetromino.color;

        for IVec2 { x, y } in tetromino.occupied_tiles() {
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(BLOCK_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(
                    x as f32 * BLOCK_SIZE - GRID_WIDTH as f32 / 2.0 * BLOCK_SIZE,
                    y as f32 * BLOCK_SIZE - GRID_HEIGHT as f32 / 2.0 * BLOCK_SIZE,
                    // render active tetrominoes in front of inactive ones
                    if active.is_some() { 1.0 } else { 0.0 },
                ),
                Redraw,
            ));
        }
    }

    for (&IVec2 { x, y }, &color) in &grid.tiles {
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(BLOCK_SIZE - 1.0)),
                ..default()
            },
            Transform::from_xyz(
                x as f32 * BLOCK_SIZE - GRID_WIDTH as f32 / 2.0 * BLOCK_SIZE,
                y as f32 * BLOCK_SIZE - GRID_HEIGHT as f32 / 2.0 * BLOCK_SIZE,
                0.0,
            ),
            Redraw,
        ));
    }
}

fn handle_movement(
    mut active_tetromino: Query<&mut Tetromino, With<Active>>,
    grid: Res<Grid>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut tetromino) = active_tetromino.single_mut() {
        let mut new_tetromino = tetromino.clone();

        if input.just_pressed(KeyCode::KeyA) {
            new_tetromino.move_left();
        }

        if input.just_pressed(KeyCode::KeyD) {
            new_tetromino.move_right();
        }

        if input.pressed(KeyCode::KeyS) {
            new_tetromino.move_down();
        }

        if !new_tetromino
            .occupied_tiles()
            .iter()
            .any(|pos| grid.tiles.contains_key(pos))
            && !new_tetromino.is_in_ground()
            && !new_tetromino.is_in_wall()
        {
            *tetromino = new_tetromino;
        }
    }
}

const fn get_wall_kick_offsets(kind: TetrominoKind, from: Rotation, to: Rotation) -> [IVec2; 5] {
    match kind {
        TetrominoKind::I => match (from, to) {
            (Rotation::North, Rotation::East) => [
                IVec2::ZERO,
                IVec2::new(-2, 0),
                IVec2::new(1, 0),
                IVec2::new(-2, -1),
                IVec2::new(1, 2),
            ],
            (Rotation::East, Rotation::North) => [
                IVec2::ZERO,
                IVec2::new(2, 0),
                IVec2::new(-1, 0),
                IVec2::new(2, 1),
                IVec2::new(-1, -2),
            ],
            (Rotation::East, Rotation::South) => [
                IVec2::ZERO,
                IVec2::new(-1, 0),
                IVec2::new(2, 0),
                IVec2::new(-1, 2),
                IVec2::new(2, -1),
            ],
            (Rotation::South, Rotation::East) => [
                IVec2::ZERO,
                IVec2::new(1, 0),
                IVec2::new(-2, 0),
                IVec2::new(1, -2),
                IVec2::new(-2, 1),
            ],
            (Rotation::South, Rotation::West) => [
                IVec2::ZERO,
                IVec2::new(2, 0),
                IVec2::new(-1, 0),
                IVec2::new(2, 1),
                IVec2::new(-1, -2),
            ],
            (Rotation::West, Rotation::South) => [
                IVec2::ZERO,
                IVec2::new(-2, 0),
                IVec2::new(1, 0),
                IVec2::new(-2, -1),
                IVec2::new(1, 2),
            ],
            (Rotation::West, Rotation::North) => [
                IVec2::ZERO,
                IVec2::new(1, 0),
                IVec2::new(-2, 0),
                IVec2::new(1, -2),
                IVec2::new(-2, 1),
            ],
            (Rotation::North, Rotation::West) => [
                IVec2::ZERO,
                IVec2::new(-1, 0),
                IVec2::new(2, 0),
                IVec2::new(-1, 2),
                IVec2::new(2, -1),
            ],
            _ => unreachable!(),
        },
        TetrominoKind::O => [IVec2::ZERO; 5],
        _ => match (from, to) {
            (Rotation::North, Rotation::East) => [
                IVec2::ZERO,
                IVec2::new(-1, 0),
                IVec2::new(-1, 1),
                IVec2::new(0, -2),
                IVec2::new(-1, -2),
            ],
            (Rotation::East, Rotation::North) => [
                IVec2::ZERO,
                IVec2::new(1, 0),
                IVec2::new(1, -1),
                IVec2::new(0, 2),
                IVec2::new(1, 2),
            ],
            (Rotation::East, Rotation::South) => [
                IVec2::ZERO,
                IVec2::new(1, 0),
                IVec2::new(1, -1),
                IVec2::new(0, 2),
                IVec2::new(1, 2),
            ],
            (Rotation::South, Rotation::East) => [
                IVec2::ZERO,
                IVec2::new(-1, 0),
                IVec2::new(-1, 1),
                IVec2::new(0, -2),
                IVec2::new(-1, -2),
            ],
            (Rotation::South, Rotation::West) => [
                IVec2::ZERO,
                IVec2::new(1, 0),
                IVec2::new(1, 1),
                IVec2::new(0, -2),
                IVec2::new(1, -2),
            ],
            (Rotation::West, Rotation::South) => [
                IVec2::ZERO,
                IVec2::new(-1, 0),
                IVec2::new(-1, -1),
                IVec2::new(0, 2),
                IVec2::new(-1, 2),
            ],
            (Rotation::West, Rotation::North) => [
                IVec2::ZERO,
                IVec2::new(1, 0),
                IVec2::new(1, -1),
                IVec2::new(0, 2),
                IVec2::new(1, 2),
            ],
            (Rotation::North, Rotation::West) => [
                IVec2::ZERO,
                IVec2::new(-1, 0),
                IVec2::new(-1, 1),
                IVec2::new(0, -2),
                IVec2::new(-1, -2),
            ],
            _ => unreachable!(),
        },
    }
}

fn handle_rotation(
    mut active_tetromino: Query<&mut Tetromino, With<Active>>,
    grid: Res<Grid>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut tetromino) = active_tetromino.single_mut() {
        let mut new_tetromino = tetromino.clone();
        if input.just_pressed(KeyCode::KeyQ) {
            new_tetromino.rotate_left();
        } else if input.just_pressed(KeyCode::KeyE) {
            new_tetromino.rotate_right();
        } else {
            return;
        };

        let offsets =
            get_wall_kick_offsets(tetromino.kind, tetromino.rotation, new_tetromino.rotation);

        for offset in offsets {
            new_tetromino.position += offset;

            if !new_tetromino
                .occupied_tiles()
                .iter()
                .any(|pos| grid.tiles.contains_key(pos))
                && !new_tetromino.is_in_ground()
                && !new_tetromino.is_in_wall()
            {
                *tetromino = new_tetromino;
                break;
            } else {
                new_tetromino.position -= offset
            }
        }
    }
}

fn clear_lines(mut grid: ResMut<Grid>, mut score: ResMut<Score>) {
    let mut row_counts = [0; GRID_HEIGHT as usize];

    for pos in grid.tiles.keys() {
        if pos.y >= 0 && pos.y < GRID_HEIGHT {
            row_counts[pos.y as usize] += 1;
        }
    }

    let full_rows: Vec<i32> = row_counts
        .iter()
        .enumerate()
        .filter_map(|(y, &count)| {
            if count == GRID_WIDTH {
                Some(y as i32)
            } else {
                None
            }
        })
        .collect();

    if full_rows.is_empty() {
        return;
    }

    for &y in &full_rows {
        grid.tiles.retain(|pos, _| pos.y != y);
    }

    score.0 += match full_rows.len() {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => unreachable!(),
    };

    for &cleared_y in &full_rows {
        let mut new_tiles = HashMap::new();

        for (pos, color) in grid.tiles.drain() {
            let new_pos = if pos.y > cleared_y {
                pos - IVec2::Y * full_rows.len() as i32
            } else {
                pos
            };
            new_tiles.insert(new_pos, color);
        }

        grid.tiles = new_tiles;
    }
}
