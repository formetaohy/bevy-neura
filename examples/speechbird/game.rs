use crate::bird::{self, Bird};
use crate::flap::Flap;
use crate::pipe::{self, Pipe};
use crate::utterance::Utterance;
use crate::world;
use bevy::prelude::*;

const ENTROPY: u32 = 0x9E37_79B9;
const STEP: f32 = 0.05;

#[derive(Message)]
pub struct Wingbeat;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Ready,
    Play,
    Falling,
    Over,
}

#[derive(Resource)]
pub struct Game {
    stage: Stage,
    score: u32,
    best: u32,
    next: f32,
    gap: f32,
    entropy: u32,
    requested: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            stage: Stage::Ready,
            score: 0,
            best: 0,
            next: 0.0,
            gap: bird::START_Y,
            entropy: ENTROPY,
            requested: false,
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

    pub fn best(&self) -> u32 {
        self.best
    }

    fn start(&mut self) {
        self.stage = Stage::Play;
        self.score = 0;
        self.next = 0.0;
        self.gap = bird::START_Y;
    }

    fn lose(&mut self) {
        self.stage = Stage::Falling;
    }

    fn finish(&mut self) {
        self.stage = Stage::Over;
        self.best = self.best.max(self.score);
    }

    fn drift(&mut self) -> f32 {
        self.entropy = self
            .entropy
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let unit = (self.entropy >> 8) as f32 / (1u32 << 24) as f32;
        let target = pipe::LOW + unit * (pipe::HIGH - pipe::LOW);
        self.gap = (self.gap + (target - self.gap).clamp(-pipe::DRIFT, pipe::DRIFT))
            .clamp(pipe::LOW, pipe::HIGH);
        self.gap
    }
}

pub fn direct(mut utterances: MessageReader<Utterance>, mut game: ResMut<Game>) {
    if utterances
        .read()
        .any(|utterance| Flap::read(&utterance.text).is_some())
    {
        game.requested = true;
    }
}

pub fn advance(
    time: Res<Time>,
    mut commands: Commands,
    mut beats: MessageWriter<Wingbeat>,
    mut game: ResMut<Game>,
    mut birds: Query<(&mut Bird, &mut Transform), Without<Pipe>>,
    mut pipes: Query<(Entity, &mut Transform, &mut Pipe), Without<Bird>>,
) {
    let dt = time.delta_secs().min(STEP);
    let Ok((mut bird, mut place)) = birds.single_mut() else {
        return;
    };
    let mut flapped = false;
    match game.stage {
        Stage::Ready => {
            if game.requested {
                launch(&mut game, &mut bird, &mut place);
                flapped = true;
            } else {
                bird.hover(time.elapsed_secs(), &mut place);
            }
        }
        Stage::Play => {
            if game.requested {
                bird.flap();
                flapped = true;
            }
            if bird.fall(dt, &mut place) {
                game.lose();
            } else {
                play(&mut game, &mut commands, dt, &mut place, &mut pipes);
                if pipes.iter().any(|(_, frame, pipe)| {
                    pipe.touches(frame.translation.xy(), place.translation.xy(), bird::RADIUS)
                }) {
                    game.lose();
                }
            }
        }
        Stage::Falling => {
            if bird.fall(dt, &mut place) {
                game.finish();
            }
        }
        Stage::Over => {
            if game.requested {
                for (entity, _, _) in &mut pipes {
                    commands.entity(entity).despawn();
                }
                launch(&mut game, &mut bird, &mut place);
                flapped = true;
            }
        }
    }
    if flapped {
        beats.write(Wingbeat);
    }
    game.requested = false;
    bird.tilt(&mut place);
}

fn launch(game: &mut Game, bird: &mut Bird, place: &mut Transform) {
    bird.home(place);
    game.start();
    bird.flap();
}

fn play(
    game: &mut Game,
    commands: &mut Commands,
    dt: f32,
    place: &mut Transform,
    pipes: &mut Query<(Entity, &mut Transform, &mut Pipe), Without<Bird>>,
) {
    for (entity, mut frame, mut pipe) in pipes.iter_mut() {
        frame.translation.x -= pipe::SPEED * dt;
        if frame.translation.x + pipe::WIDTH / 2.0 < -world::WIDTH / 2.0 {
            commands.entity(entity).despawn();
        } else if pipe.passed(frame.translation.x, place.translation.x) {
            game.score += 1;
        }
    }
    game.next -= dt;
    if game.next <= 0.0 {
        game.next += pipe::PERIOD;
        pipe::spawn(commands, game.drift());
    }
}
