use crate::direction::Direction;
use crate::ghost::{Ghost, Mode};
use crate::maze::{Maze, Pellet, TILE, Tile};
use crate::utterance::Utterance;
use crate::walker::{Walker, step_of};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

const LIVES: i32 = 3;
const PLAYER_SPEED: f32 = 6.5;
const GHOST_SPEED: f32 = 5.2;
const GHOST_GAIN: f32 = 0.35;
const GHOST_CEILING: f32 = 6.2;
const FRIGHT_SPEED: f32 = 3.2;
const FRIGHT: f32 = 7.0;
const FLASH: f32 = 2.0;
const SCATTER: f32 = 7.0;
const CHASE: f32 = 20.0;
const DOT_POINTS: u32 = 10;
const POWER_POINTS: u32 = 50;
const GHOST_POINTS: u32 = 200;
const CATCH: f32 = 0.6;
const READY: f32 = 2.0;
const PAUSE: f32 = 2.0;
const RESPAWN: f32 = 1.5;
const PAC_SIZE: f32 = 27.0;
const PAC_MOUTH: f32 = 90.0;
const PAC_SEGMENTS: u32 = 64;
const CHEWS: f32 = 6.0;
const GHOST_SIZE: f32 = 27.0;
const DOT_SIZE: f32 = 6.0;
const POWER_SIZE: f32 = 17.0;
pub const LIFT: f32 = 32.0;

const WALL_COLOR: Color = Color::srgb(0.16, 0.24, 0.62);
const DOOR_COLOR: Color = Color::srgb(0.9, 0.7, 0.85);
const PAC_COLOR: Color = Color::srgb(0.98, 0.85, 0.2);
const DOT_COLOR: Color = Color::srgb(0.93, 0.9, 0.79);
const POWER_COLOR: Color = Color::srgb(1.0, 0.96, 0.6);
const FRIGHT_COLOR: Color = Color::srgb(0.24, 0.36, 0.95);
const FLASH_COLOR: Color = Color::srgb(0.95, 0.95, 0.98);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Stage {
    Ready(f32),
    Play,
    Caught(f32),
    Cleared(f32),
    Over,
}

#[derive(Resource)]
pub struct Game {
    stage: Stage,
    score: u32,
    lives: i32,
    level: u32,
    mode: Mode,
    mode_left: f32,
    fright: f32,
    pellets: HashMap<IVec2, Entity>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            stage: Stage::Ready(READY),
            score: 0,
            lives: LIVES,
            level: 1,
            mode: Mode::Chase,
            mode_left: CHASE,
            fright: 0.0,
            pellets: HashMap::new(),
        }
    }
}

impl Game {
    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn lives(&self) -> i32 {
        self.lives
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    fn speed(&self, frightened: bool) -> f32 {
        if frightened {
            FRIGHT_SPEED
        } else {
            (GHOST_SPEED + GHOST_GAIN * (self.level - 1) as f32).min(GHOST_CEILING)
        }
    }

    fn eat(&mut self, commands: &mut Commands, at: IVec2) {
        let Some(entity) = self.pellets.remove(&at) else {
            return;
        };
        commands.entity(entity).despawn();
        match Maze::pellet(at) {
            Some(Pellet::Dot) => self.score += DOT_POINTS,
            Some(Pellet::Power) => {
                self.score += POWER_POINTS;
                self.fright = FRIGHT;
            }
            None => {}
        }
    }

    fn reset(&mut self, pac: &mut Pac, ghosts: &mut Ghosts<'_, '_>) {
        self.fright = 0.0;
        pac.walk = Walker::at(Maze::start());
        pac.wanted = None;
        for (mut ghost, mut place, mut sprite) in &mut *ghosts {
            ghost.reset();
            place.translation = Maze::point(ghost.walk.position()).extend(2.0);
            sprite.color = ghost.tint;
        }
    }
}

#[derive(Component)]
pub struct Pac {
    pub walk: Walker,
    wanted: Option<Direction>,
    facing: Direction,
}

#[derive(Resource)]
pub struct PacFace {
    closed: Handle<Mesh>,
    open: Handle<Mesh>,
}

fn sector(radius: f32, mouth: f32) -> Mesh {
    CircularSector::from(Arc2d {
        radius,
        half_angle: (180.0 - mouth / 2.0).to_radians(),
    })
    .mesh()
    .resolution(PAC_SEGMENTS)
    .build()
}

fn spin(direction: Direction) -> f32 {
    let step = step_of(direction).as_vec2();
    Vec2::new(step.x, -step.y).to_angle() + FRAC_PI_2
}

type Pacs<'w, 's> = Query<'w, 's, (&'static mut Pac, &'static mut Transform)>;
type Ghosts<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Ghost,
        &'static mut Transform,
        &'static mut Sprite,
    ),
    Without<Pac>,
>;

pub fn setup(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, LIFT, 0.0)));
    for (at, tile) in Maze::tiles() {
        let color = match tile {
            Tile::Wall => WALL_COLOR,
            Tile::Door => DOOR_COLOR,
            Tile::Floor | Tile::House => continue,
        };
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(TILE)),
            Transform::from_translation(Maze::center(at).extend(0.0)),
        ));
    }
    fill(&mut commands, &mut game);
    let face = PacFace {
        closed: meshes.add(sector(PAC_SIZE / 2.0, 0.0)),
        open: meshes.add(sector(PAC_SIZE / 2.0, PAC_MOUTH)),
    };
    commands.spawn((
        Pac {
            walk: Walker::at(Maze::start()),
            wanted: None,
            facing: Direction::Left,
        },
        Mesh2d(face.open.clone()),
        MeshMaterial2d(materials.add(ColorMaterial::from(PAC_COLOR))),
        Transform::from_translation(Maze::center(Maze::start()).extend(3.0))
            .with_rotation(Quat::from_rotation_z(spin(Direction::Left))),
    ));
    commands.insert_resource(face);
    for ghost in Ghost::roster() {
        commands.spawn((
            Sprite::from_color(ghost.tint, Vec2::splat(GHOST_SIZE)),
            Transform::from_translation(Maze::point(ghost.walk.position()).extend(2.0)),
            ghost,
        ));
    }
}

fn fill(commands: &mut Commands, game: &mut Game) {
    for (_, entity) in game.pellets.drain() {
        commands.entity(entity).despawn();
    }
    for (at, _) in Maze::tiles() {
        let Some(pellet) = Maze::pellet(at) else {
            continue;
        };
        let (color, size) = match pellet {
            Pellet::Dot => (DOT_COLOR, DOT_SIZE),
            Pellet::Power => (POWER_COLOR, POWER_SIZE),
        };
        let entity = commands
            .spawn((
                Sprite::from_color(color, Vec2::splat(size)),
                Transform::from_translation(Maze::center(at).extend(1.0)),
            ))
            .id();
        game.pellets.insert(at, entity);
    }
}

pub fn direct(
    keys: Res<ButtonInput<KeyCode>>,
    mut utterances: MessageReader<Utterance>,
    mut pacs: Query<&mut Pac>,
) {
    let mut orders = utterances
        .read()
        .filter_map(|utterance| Direction::read(&utterance.text))
        .collect::<Vec<Direction>>();
    orders.extend(keys.get_just_pressed().filter_map(|key| pressed(*key)));
    let Some(order) = orders.pop() else {
        return;
    };
    let Ok(mut pac) = pacs.single_mut() else {
        return;
    };
    pac.wanted = Some(order);
    pac.facing = order;
}

pub fn advance(
    time: Res<Time>,
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut pacs: Pacs,
    mut ghosts: Ghosts,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok((mut pac, mut place)) = pacs.single_mut() else {
        return;
    };
    match game.stage {
        Stage::Ready(left) => {
            game.stage = if left <= dt {
                Stage::Play
            } else {
                Stage::Ready(left - dt)
            };
        }
        Stage::Caught(left) => {
            let left = left - dt;
            if left > 0.0 {
                game.stage = Stage::Caught(left);
            } else if game.lives > 0 {
                game.reset(&mut pac, &mut ghosts);
                game.stage = Stage::Ready(READY);
            } else {
                pac.wanted = None;
                game.stage = Stage::Over;
            }
        }
        Stage::Cleared(left) => {
            let left = left - dt;
            if left > 0.0 {
                game.stage = Stage::Cleared(left);
            } else {
                game.level += 1;
                fill(&mut commands, &mut game);
                game.reset(&mut pac, &mut ghosts);
                game.stage = Stage::Ready(READY);
            }
        }
        Stage::Over => {
            if pac.wanted.is_some() {
                *game = Game::default();
                fill(&mut commands, &mut game);
                game.reset(&mut pac, &mut ghosts);
            }
        }
        Stage::Play => {
            play(&mut game, &mut commands, dt, &mut pac, &mut ghosts);
        }
    }
    place.translation = Maze::point(pac.walk.position()).extend(3.0);
}

fn play(
    game: &mut Game,
    commands: &mut Commands,
    dt: f32,
    pac: &mut Pac,
    ghosts: &mut Ghosts<'_, '_>,
) {
    game.fright = (game.fright - dt).max(0.0);
    game.mode_left -= dt;
    if game.mode_left <= 0.0 {
        (game.mode, game.mode_left) = match game.mode {
            Mode::Scatter => (Mode::Chase, CHASE),
            Mode::Chase => (Mode::Scatter, SCATTER),
        };
    }
    let wanted = pac.wanted;
    pac.walk.travel(PLAYER_SPEED * dt, wanted, |tile, _| {
        let open = |direction: Direction| Maze::passable(tile, tile + step_of(direction), false);
        wanted.filter(|want| open(*want))
    });
    game.eat(commands, pac.walk.tile);
    if game.pellets.is_empty() {
        game.stage = Stage::Cleared(PAUSE);
        return;
    }
    let frightened = game.fright > 0.0;
    let speed = game.speed(frightened);
    let mode = game.mode;
    let pac_at = pac.walk.position();
    let pac_tile = pac.walk.tile;
    for (mut ghost, mut place, mut sprite) in &mut *ghosts {
        if ghost.wait > 0.0 {
            ghost.wait -= dt;
        } else {
            ghost.travel(speed * dt, pac_tile, mode, frightened);
        }
        place.translation = Maze::point(ghost.walk.position()).extend(2.0);
        sprite.color = if frightened {
            flash(game.fright)
        } else {
            ghost.tint
        };
        if ghost.wait > 0.0 || ghost.captive() {
            continue;
        }
        if (ghost.walk.position() - pac_at).length() >= CATCH {
            continue;
        }
        if frightened {
            game.score += GHOST_POINTS;
            ghost.send_home(RESPAWN);
            sprite.color = ghost.tint;
        } else if matches!(game.stage, Stage::Play) {
            game.lives -= 1;
            pac.wanted = None;
            game.stage = Stage::Caught(PAUSE);
        }
    }
}

fn flash(fright: f32) -> Color {
    if fright < FLASH && (fright * 8.0) as i32 % 2 == 0 {
        FLASH_COLOR
    } else {
        FRIGHT_COLOR
    }
}

fn pressed(key: KeyCode) -> Option<Direction> {
    match key {
        KeyCode::ArrowLeft => Some(Direction::Left),
        KeyCode::ArrowRight => Some(Direction::Right),
        KeyCode::ArrowUp => Some(Direction::Up),
        KeyCode::ArrowDown => Some(Direction::Down),
        _ => None,
    }
}

pub fn face(
    time: Res<Time>,
    faces: Res<PacFace>,
    mut pacs: Query<(&Pac, &mut Transform, &mut Mesh2d)>,
) {
    let Ok((pac, mut place, mut mesh)) = pacs.single_mut() else {
        return;
    };
    let facing = pac.facing;
    place.rotation = Quat::from_rotation_z(spin(facing));
    let biting = pac.walk.moving() && (time.elapsed_secs() * CHEWS) as u32 % 2 == 1;
    mesh.0 = if biting {
        faces.closed.clone()
    } else {
        faces.open.clone()
    };
}
