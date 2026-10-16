use crate::canvas::{self, Canvas};
use crate::digit::Digit;
use crate::ink::Ink;
use crate::puzzle::Puzzle;
use crate::random::Random;
use crate::recognizer::{Guess, Recognizer};
use crate::view::CanvasNode;
use bevy::picking::backend::HitData;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const READ_DELAY: f32 = 0.6;
const HOLD: f32 = 1.0;
const SEED: u64 = 0x5eed_0003;

#[derive(Resource)]
pub struct Game {
    recognizer: Recognizer,
    random: Random,
    puzzle: Puzzle,
    canvas: Canvas,
    reading: Option<Guess>,
    verdict: Option<bool>,
    idle: f32,
    hold: f32,
    right: u32,
    missed: u32,
}

impl Game {
    pub fn of(recognizer: Recognizer) -> Self {
        let mut random = Random::seeded(SEED);
        let puzzle = Puzzle::draw(&mut random);
        Self {
            recognizer,
            random,
            puzzle,
            canvas: Canvas::new(),
            reading: None,
            verdict: None,
            idle: 0.0,
            hold: 0.0,
            right: 0,
            missed: 0,
        }
    }

    pub fn expression(&self) -> String {
        self.puzzle.text()
    }

    pub fn reading(&self) -> Option<Guess> {
        self.reading
    }

    pub fn verdict(&self) -> Option<bool> {
        self.verdict
    }

    pub fn right(&self) -> u32 {
        self.right
    }

    pub fn missed(&self) -> u32 {
        self.missed
    }

    pub fn ink(&self) -> &Ink {
        self.canvas.ink()
    }

    pub fn revision(&self) -> u32 {
        self.canvas.revision()
    }

    fn press(&mut self, at: Vec2) {
        match self.verdict {
            Some(true) => self.fresh(),
            Some(false) => self.retry(),
            None => {}
        }
        self.canvas.press(at);
        self.idle = 0.0;
    }

    fn drag(&mut self, at: Vec2) {
        let before = self.canvas.revision();
        self.canvas.drag(at);
        if self.canvas.revision() != before {
            self.idle = 0.0;
        }
    }

    fn lift(&mut self) {
        self.canvas.lift();
    }

    fn judge(&mut self, guess: Guess) {
        self.reading = Some(guess);
        let right = u32::from(guess.digit) == self.puzzle.answer();
        self.verdict = Some(right);
        self.hold = 0.0;
        if right {
            self.right += 1;
        } else {
            self.missed += 1;
        }
    }

    fn fresh(&mut self) {
        self.puzzle = Puzzle::draw(&mut self.random);
        self.retry();
    }

    fn retry(&mut self) {
        self.canvas.clear();
        self.reading = None;
        self.verdict = None;
        self.idle = 0.0;
        self.hold = 0.0;
    }
}

pub fn touch(
    mut game: ResMut<Game>,
    boxes: Query<Entity, With<CanvasNode>>,
    mut presses: MessageReader<Pointer<Press>>,
    mut moves: MessageReader<Pointer<Move>>,
    mut releases: MessageReader<Pointer<Release>>,
) {
    let held = boxes
        .single()
        .expect("the board carries one box to write in");
    for event in presses.read() {
        if event.entity == held {
            game.press(at(&event.hit));
        }
    }
    for event in moves.read() {
        if event.entity == held {
            game.drag(at(&event.hit));
        }
    }
    for event in releases.read() {
        if event.entity == held {
            game.lift();
        }
    }
}

pub fn read(runtime: Res<NeuraRuntime>, time: Res<Time>, mut game: ResMut<Game>) {
    game.idle += time.delta_secs();
    if game.reading.is_some() || game.canvas.drawing() || !game.canvas.drawn() {
        return;
    }
    if game.idle < READ_DELAY {
        return;
    }
    let Some(digit) = Digit::drawn(&game.canvas) else {
        return;
    };
    let guess = game
        .recognizer
        .read(&runtime, digit.values())
        .into_iter()
        .next()
        .expect("a reading of one digit answers one guess");
    game.judge(guess);
}

pub fn advance(time: Res<Time>, mut game: ResMut<Game>) {
    if game.verdict != Some(true) {
        return;
    }
    game.hold += time.delta_secs();
    if game.hold >= HOLD {
        game.fresh();
    }
}

fn at(hit: &HitData) -> Vec2 {
    canvas::placed(
        hit.position
            .expect("the box reports where a pointer touches it")
            .truncate(),
    )
}
