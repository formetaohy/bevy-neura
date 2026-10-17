use crate::board::{Board, Outcome};
use crate::training::Training;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const THINK: f32 = 0.35;
const HOLD: f32 = 2.2;
const SHOW_MOVE: f32 = 0.30;
const SHOW_HOLD: f32 = 0.8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Win,
    Draw,
    Loss,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Learning,
    Human,
    Computer,
    Done(Verdict),
}

#[derive(Component)]
pub struct Cell(pub usize);

#[derive(Resource)]
pub struct Game {
    training: Training,
    board: Board,
    stage: Stage,
    timer: f32,
    showcase_over: bool,
}

impl Game {
    pub fn attach(runtime: &NeuraRuntime) -> Self {
        Self {
            training: Training::attach(runtime),
            board: Board::start(),
            stage: Stage::Learning,
            timer: 0.0,
            showcase_over: false,
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn progress(&self) -> f32 {
        self.training.progress()
    }

    pub fn message(&self) -> &'static str {
        match self.stage {
            Stage::Learning => "learning",
            Stage::Human => "your move",
            Stage::Computer => "thinking",
            Stage::Done(Verdict::Win) => "you win",
            Stage::Done(Verdict::Draw) => "draw",
            Stage::Done(Verdict::Loss) => "you lose",
        }
    }

    pub fn click(&mut self, at: usize) {
        if self.stage != Stage::Human || self.board.cells()[at] != 0 {
            return;
        }
        self.timer = 0.0;
        self.stage = match self.board.play(at) {
            Outcome::Won => Stage::Done(Verdict::Win),
            Outcome::Drawn => Stage::Done(Verdict::Draw),
            Outcome::Ongoing => Stage::Computer,
        };
    }

    pub fn advance(&mut self, runtime: &NeuraRuntime, elapsed: f32) {
        match self.stage {
            Stage::Learning => self.learn(runtime, elapsed),
            Stage::Computer => {
                self.timer += elapsed;
                if self.timer >= THINK {
                    self.timer = 0.0;
                    let action = self.training.choose(runtime, &self.board);
                    self.stage = match self.board.play(action) {
                        Outcome::Won => Stage::Done(Verdict::Loss),
                        Outcome::Drawn => Stage::Done(Verdict::Draw),
                        Outcome::Ongoing => Stage::Human,
                    };
                }
            }
            Stage::Done(_) => {
                self.timer += elapsed;
                if self.timer >= HOLD {
                    self.board = Board::start();
                    self.stage = Stage::Human;
                    self.timer = 0.0;
                }
            }
            Stage::Human => {}
        }
    }

    fn learn(&mut self, runtime: &NeuraRuntime, elapsed: f32) {
        self.training.advance(runtime);
        if self.training.done() {
            self.board = Board::start();
            self.stage = Stage::Human;
            self.showcase_over = false;
            self.timer = 0.0;
            return;
        }
        self.timer += elapsed;
        if self.showcase_over {
            if self.timer >= SHOW_HOLD {
                self.board = Board::start();
                self.showcase_over = false;
                self.timer = 0.0;
            }
            return;
        }
        if self.timer < SHOW_MOVE {
            return;
        }
        let action = self.training.choose(runtime, &self.board);
        self.showcase_over = self.board.play(action) != Outcome::Ongoing;
        self.timer = 0.0;
    }
}

pub fn advance(runtime: Res<NeuraRuntime>, time: Res<Time>, mut game: ResMut<Game>) {
    game.advance(&runtime, time.delta_secs());
}

pub fn click(mut game: ResMut<Game>, cells: Query<(&Cell, &Interaction), Changed<Interaction>>) {
    for (cell, interaction) in &cells {
        if *interaction == Interaction::Pressed {
            game.click(cell.0);
        }
    }
}
