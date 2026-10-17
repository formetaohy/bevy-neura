use crate::board::{Board, CELLS, Outcome};
use crate::learner::{BATCH, Learner};
use crate::net::{BOUND, Player};
use crate::replay::{Replay, Transition};
use bevy_neura::NeuraRuntime;
use bevy_neura_example::random::Random;

pub const GAMES_PER_FRAME: u32 = BOUND;
pub const TOTAL_GAMES: u32 = 96_000;

const UPDATES: u32 = 8;
const REPLAY: usize = 1 << 17;
const SEED: u64 = 0x5eed_0004;
const EPSILON_START: f32 = 1.0;
const EPSILON_END: f32 = 0.05;
const RATE_START: f32 = 5e-4;
const RATE_END: f32 = 1e-4;

pub struct Training {
    learner: Learner,
    player: Player,
    replay: Replay,
    random: Random,
    games: u32,
}

impl Training {
    pub fn attach(runtime: &NeuraRuntime) -> Self {
        let learner = Learner::attach(runtime);
        let player = Player::share(runtime, learner.model());
        Self {
            learner,
            player,
            replay: Replay::new(REPLAY),
            random: Random::seeded(SEED),
            games: 0,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.games as f32 / TOTAL_GAMES as f32).clamp(0.0, 1.0)
    }

    pub fn done(&self) -> bool {
        self.games >= TOTAL_GAMES
    }

    pub fn advance(&mut self, runtime: &NeuraRuntime) {
        if self.done() {
            return;
        }
        let games = GAMES_PER_FRAME.min(TOTAL_GAMES - self.games);
        self.play(runtime, games);
        for _ in 0..UPDATES {
            self.learn(runtime);
        }
    }

    pub fn choose(&self, runtime: &NeuraRuntime, board: &Board) -> usize {
        let legal = board.legal().collect::<Vec<_>>();
        assert!(!legal.is_empty(), "a board that takes a mark holds a move");
        let q = self.player.q(runtime, &board.perspective(), 1);
        best(&q, &legal)
    }

    fn play(&mut self, runtime: &NeuraRuntime, games: u32) {
        let mut boards = vec![Board::start(); games as usize];
        let mut active = (0..boards.len()).collect::<Vec<_>>();
        while !active.is_empty() {
            let rows = active.len() as u32;
            let mut states = Vec::with_capacity(rows as usize * CELLS);
            for index in &active {
                states.extend_from_slice(&boards[*index].perspective());
            }
            let q = self.player.q(runtime, &states, rows);
            let epsilon = self.epsilon();
            let mut following = Vec::with_capacity(active.len());
            for (row, index) in active.into_iter().enumerate() {
                let board = &mut boards[index];
                let legal = board.legal().collect::<Vec<_>>();
                assert!(!legal.is_empty(), "a running board holds a move");
                let action = if self.random.unit() < f64::from(epsilon) {
                    legal[self.random.below(legal.len())]
                } else {
                    best(&q[row * CELLS..(row + 1) * CELLS], &legal)
                };
                let state = board.perspective();
                let outcome = board.play(action);
                let after = board.perspective();
                let (reward, done) = match outcome {
                    Outcome::Won => (1.0, true),
                    Outcome::Drawn => (0.0, true),
                    Outcome::Ongoing => (0.0, false),
                };
                self.replay.push(Transition {
                    state,
                    following: after,
                    action: action as u8,
                    reward,
                    done,
                });
                if !done {
                    following.push(index);
                }
            }
            active = following;
        }
        self.games += games;
    }

    fn learn(&mut self, runtime: &NeuraRuntime) {
        if !self.replay.ready(BATCH as usize) {
            return;
        }
        let batch = self.replay.sample(&mut self.random, BATCH as usize);
        let following = self.learner.target_q(runtime, &batch.following);
        let mut targets = Vec::with_capacity(BATCH as usize);
        for row in 0..BATCH as usize {
            let mut target = batch.rewards[row];
            if !batch.done[row] {
                let base = row * CELLS;
                let mut value = f32::NEG_INFINITY;
                for at in 0..CELLS {
                    if batch.following[base + at] == 0.0 {
                        value = value.max(following[base + at]);
                    }
                }
                assert!(
                    value.is_finite(),
                    "a running board of {CELLS} cells holds no move",
                );
                target -= value;
            }
            assert!(target.is_finite(), "a target of {target} reaches no score");
            targets.push(target);
        }
        self.learner.update(
            runtime,
            &batch.states,
            &batch.actions,
            &targets,
            self.rate(),
        );
    }

    fn epsilon(&self) -> f32 {
        EPSILON_END + (EPSILON_START - EPSILON_END) * (1.0 - self.progress()).powi(3)
    }

    fn rate(&self) -> f32 {
        RATE_END + (RATE_START - RATE_END) * (1.0 - self.progress())
    }
}

fn best(q: &[f32], legal: &[usize]) -> usize {
    assert!(!legal.is_empty(), "a choice of no move reaches no cell");
    let mut choice = legal[0];
    let mut value = q[choice];
    for at in &legal[1..] {
        if q[*at] > value {
            value = q[*at];
            choice = *at;
        }
    }
    choice
}
