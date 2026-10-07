use crate::command::Command;
use crate::speech::Transcription;
use bevy::prelude::*;

pub const ARENA: Vec2 = Vec2::new(880.0, 540.0);
const PLAYER: f32 = 36.0;
const ENEMY: f32 = 26.0;
const STEP: f32 = 96.0;
const REACH: f32 = 420.0;
const FREEZE: f32 = 2.5;
const EVERY: f32 = 1.5;
const SPEED: f32 = 38.0;
const BOLT: f32 = 0.25;

#[derive(Resource)]
pub struct Game {
    pub score: u32,
    pub health: i32,
    pub over: bool,
    pub command: Command,
    pub frozen: f32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            score: 0,
            health: 3,
            over: false,
            command: Command::Unknown,
            frozen: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct Spawner {
    timer: Timer,
    entropy: u32,
}

impl Default for Spawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(EVERY, TimerMode::Repeating),
            entropy: 0x9E37_79B9,
        }
    }
}

impl Spawner {
    fn next(&mut self) -> f32 {
        self.entropy ^= self.entropy << 13;
        self.entropy ^= self.entropy >> 17;
        self.entropy ^= self.entropy << 5;
        (self.entropy >> 8) as f32 / 16_777_216.0 - 0.5
    }
}

type Chasers<'w, 's> = Query<'w, 's, (Entity, &'static Transform), (With<Enemy>, Without<Player>)>;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Bolt(pub f32);

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Player,
        Sprite::from_color(Color::srgb(0.35, 0.85, 0.55), Vec2::splat(PLAYER)),
        Transform::from_xyz(0.0, -ARENA.y / 2.0 + PLAYER, 1.0),
    ));
}

pub fn spawn(
    time: Res<Time>,
    game: Res<Game>,
    mut spawner: ResMut<Spawner>,
    mut commands: Commands,
) {
    if game.over || !spawner.timer.tick(time.delta()).just_finished() {
        return;
    }
    let room = ARENA.x / 2.0 - ENEMY;
    commands.spawn((
        Enemy,
        Sprite::from_color(Color::srgb(0.95, 0.35, 0.4), Vec2::splat(ENEMY)),
        Transform::from_xyz(
            spawner.next() * 2.0 * room,
            ARENA.y / 2.0 - ENEMY / 2.0,
            0.9,
        ),
    ));
}

pub fn march(
    time: Res<Time>,
    game: Res<Game>,
    players: Query<&Transform, With<Player>>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    let speed = if game.frozen > 0.0 { 0.0 } else { SPEED };
    for mut enemy in &mut enemies {
        let direction = (player.translation - enemy.translation).truncate();
        if direction.length() > 1.0 {
            enemy.translation += (direction.normalize() * speed * time.delta_secs()).extend(0.0);
        }
    }
}

pub fn strike(
    mut game: ResMut<Game>,
    players: Query<&Transform, With<Player>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut commands: Commands,
) {
    if game.over {
        return;
    }
    let Ok(player) = players.single() else {
        return;
    };
    for (enemy, transform) in &enemies {
        if transform.translation.distance(player.translation) < (PLAYER + ENEMY) / 2.0 {
            commands.entity(enemy).despawn();
            game.health -= 1;
            game.over = game.health <= 0;
            return;
        }
    }
}

pub fn fade(time: Res<Time>, mut bolts: Query<(Entity, &mut Bolt)>, mut commands: Commands) {
    for (entity, mut bolt) in &mut bolts {
        bolt.0 -= time.delta_secs();
        if bolt.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn steer(keys: Res<ButtonInput<KeyCode>>, mut players: Query<&mut Transform, With<Player>>) {
    let Ok(mut player) = players.single_mut() else {
        return;
    };
    let mut step = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        step.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        step.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        step.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        step.y -= 1.0;
    }
    if step != Vec2::ZERO {
        walk(&mut player, step.normalize().extend(0.0) * STEP);
    }
}

pub fn thaw(time: Res<Time>, mut game: ResMut<Game>) {
    game.frozen = (game.frozen - time.delta_secs()).max(0.0);
}

pub fn apply(
    mut heard: MessageReader<Transcription>,
    mut game: ResMut<Game>,
    mut players: Query<&mut Transform, With<Player>>,
    enemies: Chasers,
    mut commands: Commands,
) {
    for transcription in heard.read() {
        game.command = transcription.command;
        if transcription.command == Command::Restart {
            for (enemy, _) in &enemies {
                commands.entity(enemy).despawn();
            }
            *game = Game::default();
            continue;
        }
        if game.over {
            continue;
        }
        let Ok(mut player) = players.single_mut() else {
            continue;
        };
        let step = match transcription.command {
            Command::Left => Vec3::new(-STEP, 0.0, 0.0),
            Command::Right => Vec3::new(STEP, 0.0, 0.0),
            Command::Up => Vec3::new(0.0, STEP, 0.0),
            Command::Down => Vec3::new(0.0, -STEP, 0.0),
            Command::Freeze => {
                game.frozen = FREEZE;
                Vec3::ZERO
            }
            Command::Fire => {
                let target = enemies
                    .iter()
                    .map(|(entity, transform)| {
                        (transform.translation.distance(player.translation), entity)
                    })
                    .filter(|(distance, _)| *distance <= REACH)
                    .fold(None, |best: Option<(f32, Entity)>, candidate| match best {
                        Some(best) if best.0 <= candidate.0 => Some(best),
                        _ => Some(candidate),
                    });
                if let Some((_, enemy)) = target {
                    commands.entity(enemy).despawn();
                    game.score += 10;
                    commands.spawn((
                        Bolt(BOLT),
                        Sprite::from_color(Color::srgb(1.0, 0.95, 0.6), Vec2::splat(PLAYER * 1.6)),
                        Transform::from_translation(player.translation)
                            .with_scale(Vec3::splat(1.2)),
                    ));
                }
                Vec3::ZERO
            }
            _ => Vec3::ZERO,
        };
        walk(&mut player, step);
    }
}

fn walk(player: &mut Transform, step: Vec3) {
    player.translation += step;
    player.translation.x = player
        .translation
        .x
        .clamp(-ARENA.x / 2.0 + PLAYER, ARENA.x / 2.0 - PLAYER);
    player.translation.y = player
        .translation
        .y
        .clamp(-ARENA.y / 2.0 + PLAYER, ARENA.y / 2.0 - PLAYER);
}
