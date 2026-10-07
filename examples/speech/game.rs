use crate::command::Command;
use crate::speech::{Speech, Transcription};
use bevy::prelude::*;
use bevy::text::FontSize;

const ARENA: Vec2 = Vec2::new(880.0, 540.0);
const START: Vec2 = Vec2::new(0.0, -ARENA.y / 2.0 + PLAYER);
const PLAYER: f32 = 36.0;
const ENEMY: f32 = 26.0;
const STEP: f32 = 96.0;
const REACH: f32 = 420.0;
const LIFE: i32 = 3;
const FREEZE: f32 = 2.5;
const EVERY: f32 = 1.5;
const SPEED: f32 = 38.0;
const GOLDEN: f32 = 0.618_034;
const PLAYER_COLOR: Color = Color::srgb(0.35, 0.85, 0.55);
const ENEMY_COLOR: Color = Color::srgb(0.95, 0.35, 0.4);
const HUD_COLOR: Color = Color::srgb(0.72, 0.78, 0.92);
const HUD_LISTENING: Color = Color::srgb(0.95, 0.45, 0.45);

#[derive(Resource)]
pub struct Game {
    score: u32,
    life: i32,
    frozen: f32,
    timer: Timer,
    spread: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            score: 0,
            life: LIFE,
            frozen: 0.0,
            timer: Timer::from_seconds(EVERY, TimerMode::Repeating),
            spread: 0,
        }
    }
}

impl Game {
    fn over(&self) -> bool {
        self.life <= 0
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Hud;

type Pursuers<'w, 's> = Query<'w, 's, (Entity, &'static Transform), (With<Enemy>, Without<Player>)>;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Player,
        Sprite::from_color(PLAYER_COLOR, Vec2::splat(PLAYER)),
        Transform::from_translation(START.extend(1.0)),
    ));
    commands.spawn((
        Hud,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(HUD_COLOR),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(14.0),
            ..default()
        },
    ));
}

pub fn spawn(time: Res<Time>, mut game: ResMut<Game>, mut commands: Commands) {
    if game.over() || !game.timer.tick(time.delta()).just_finished() {
        return;
    }
    game.spread = game.spread.wrapping_add(1);
    let room = ARENA.x / 2.0 - ENEMY;
    let place = (game.spread as f32 * GOLDEN).fract() - 0.5;
    commands.spawn((
        Enemy,
        Sprite::from_color(ENEMY_COLOR, Vec2::splat(ENEMY)),
        Transform::from_xyz(place * 2.0 * room, ARENA.y / 2.0 - ENEMY / 2.0, 0.9),
    ));
}

pub fn march(
    time: Res<Time>,
    mut game: ResMut<Game>,
    players: Query<&Transform, With<Player>>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {
    game.frozen = (game.frozen - time.delta_secs()).max(0.0);
    let Ok(player) = players.single() else {
        return;
    };
    let speed = if game.frozen > 0.0 || game.over() {
        0.0
    } else {
        SPEED
    };
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
    if game.over() {
        return;
    }
    let Ok(player) = players.single() else {
        return;
    };
    for (enemy, transform) in &enemies {
        if transform.translation.distance(player.translation) < (PLAYER + ENEMY) / 2.0 {
            commands.entity(enemy).despawn();
            game.life -= 1;
            return;
        }
    }
}

pub fn steer(keys: Res<ButtonInput<KeyCode>>, mut players: Query<&mut Transform, With<Player>>) {
    let step = Vec2::new(
        axis(
            &keys,
            [KeyCode::ArrowRight, KeyCode::KeyD],
            [KeyCode::ArrowLeft, KeyCode::KeyA],
        ),
        axis(
            &keys,
            [KeyCode::ArrowUp, KeyCode::KeyW],
            [KeyCode::ArrowDown, KeyCode::KeyS],
        ),
    );
    if step == Vec2::ZERO {
        return;
    }
    let Ok(mut player) = players.single_mut() else {
        return;
    };
    walk(&mut player, step.normalize().extend(0.0) * STEP);
}

pub fn act(
    keys: Res<ButtonInput<KeyCode>>,
    mut heard: MessageReader<Transcription>,
    mut game: ResMut<Game>,
    pursuers: Pursuers,
    mut players: Query<&mut Transform, With<Player>>,
    mut commands: Commands,
) {
    let mut orders = heard
        .read()
        .map(|transcription| Command::read(&transcription.text))
        .collect::<Vec<Command>>();
    if keys.just_pressed(KeyCode::KeyR) {
        orders.push(Command::Restart);
    }
    for order in orders {
        if order == Command::Restart {
            for (enemy, _) in &pursuers {
                commands.entity(enemy).despawn();
            }
            *game = Game::default();
            if let Ok(mut player) = players.single_mut() {
                player.translation = START.extend(1.0);
            }
            continue;
        }
        if game.over() {
            continue;
        }
        let Ok(mut player) = players.single_mut() else {
            continue;
        };
        let step = match order {
            Command::Left => Vec3::new(-STEP, 0.0, 0.0),
            Command::Right => Vec3::new(STEP, 0.0, 0.0),
            Command::Up => Vec3::new(0.0, STEP, 0.0),
            Command::Down => Vec3::new(0.0, -STEP, 0.0),
            Command::Freeze => {
                game.frozen = FREEZE;
                Vec3::ZERO
            }
            Command::Fire => {
                let target = pursuers
                    .iter()
                    .map(|(enemy, transform)| {
                        (transform.translation.distance(player.translation), enemy)
                    })
                    .filter(|(distance, _)| *distance <= REACH)
                    .min_by(|left, right| left.0.total_cmp(&right.0));
                if let Some((_, enemy)) = target {
                    commands.entity(enemy).despawn();
                    game.score += 10;
                }
                Vec3::ZERO
            }
            _ => Vec3::ZERO,
        };
        walk(&mut player, step);
    }
}

pub fn paint(
    speech: Option<Res<Speech>>,
    game: Res<Game>,
    mut heard: MessageReader<Transcription>,
    mut last: Local<String>,
    mut huds: Query<(&mut Text, &mut TextColor), With<Hud>>,
) {
    for transcription in heard.read() {
        *last = format!(
            "heard \"{}\" -> {}",
            transcription.text,
            Command::read(&transcription.text).name(),
        );
    }
    let state = match (speech.as_ref(), game.over()) {
        (None, _) => "loading the model",
        (Some(_), true) => "game over, say restart or press R",
        (Some(speech), false) => speech.state(),
    };
    let heard = if last.is_empty() {
        "heard nothing yet"
    } else {
        last.as_str()
    };
    let listening = speech.as_ref().is_some_and(|speech| speech.busy());
    for (mut text, mut color) in &mut huds {
        text.0 = format!(
            "score {}   life {}   [{state}]\n{heard}\nhold SPACE and say left, right, up, down, fire, freeze or restart",
            game.score,
            game.life.max(0),
        );
        color.0 = if listening { HUD_LISTENING } else { HUD_COLOR };
    }
}

fn axis(keys: &ButtonInput<KeyCode>, positive: [KeyCode; 2], negative: [KeyCode; 2]) -> f32 {
    (keys.any_pressed(positive) as i32 - keys.any_pressed(negative) as i32) as f32
}

fn walk(player: &mut Transform, step: Vec3) {
    let room = ARENA / 2.0 - Vec2::splat(PLAYER);
    player.translation = Vec3::new(
        (player.translation.x + step.x).clamp(-room.x, room.x),
        (player.translation.y + step.y).clamp(-room.y, room.y),
        player.translation.z,
    );
}
