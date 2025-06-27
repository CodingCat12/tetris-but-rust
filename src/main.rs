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
    shape: [[bool; 4]; 4],
}

impl Tetromino {
    fn new() -> Self {
        let kind = rand::random::<TetrominoKind>();
        Tetromino {
            position: IVec2::new(GRID_WIDTH / 2, GRID_HEIGHT),
            color: kind.color(),
            shape: kind.shape(),
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
        for i in 0..4 {
            for j in i + 1..4 {
                let tmp = self.shape[i][j];
                self.shape[i][j] = self.shape[j][i];
                self.shape[j][i] = tmp;
            }
        }

        self.shape.reverse();
    }

    fn rotate_right(&mut self) {
        for i in 0..4 {
            for j in i + 1..4 {
                let tmp = self.shape[i][j];
                self.shape[i][j] = self.shape[j][i];
                self.shape[j][i] = tmp;
            }
        }

        for row in self.shape.iter_mut() {
            row.reverse();
        }
    }

    fn occupied_tiles(&self) -> Vec<IVec2> {
        let mut tiles = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                if self.shape[y][x] {
                    tiles.push(self.position + IVec2::new(x as i32, y as i32));
                }
            }
        }
        tiles
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

#[derive(Clone, Copy)]
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

    const fn shape(&self) -> [[bool; 4]; 4] {
        match *self {
            TetrominoKind::I => [
                [false, false, false, false],
                [true, true, true, true],
                [false, false, false, false],
                [false, false, false, false],
            ],
            TetrominoKind::O => [
                [false, false, false, false],
                [false, true, true, false],
                [false, true, true, false],
                [false, false, false, false],
            ],
            TetrominoKind::T => [
                [false, true, false, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            TetrominoKind::S => [
                [false, true, true, false],
                [true, true, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            TetrominoKind::Z => [
                [true, true, false, false],
                [false, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            TetrominoKind::J => [
                [true, false, false, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
            TetrominoKind::L => [
                [false, false, true, false],
                [true, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
        }
    }
}

fn setup(mut commands: Commands, mut event_writer: EventWriter<Tick>) {
    commands.spawn(Camera2d);

    commands.insert_resource(Ticker(Timer::from_seconds(0.5, TimerMode::Repeating)));
    commands.init_resource::<Grid>();

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

fn handle_rotation(
    mut active_tetromino: Query<&mut Tetromino, With<Active>>,
    grid: Res<Grid>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut tetromino) = active_tetromino.single_mut() {
        let mut new_tetromino = tetromino.clone();

        if input.just_pressed(KeyCode::KeyQ) {
            new_tetromino.rotate_left();
        }

        if input.just_pressed(KeyCode::KeyE) {
            new_tetromino.rotate_right();
        }

        let mut i = 0;
        while new_tetromino
            .occupied_tiles()
            .iter()
            .any(|pos| grid.tiles.contains_key(pos))
            || new_tetromino.is_in_ground()
            || new_tetromino.is_in_wall()
        {
            if i % 2 == 0 {
                new_tetromino.position.x += i;
            } else {
                new_tetromino.position.x -= i;
            }
            i += 1;
        }

        *tetromino = new_tetromino;
    }
}

fn clear_lines(mut grid: ResMut<Grid>) {
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
