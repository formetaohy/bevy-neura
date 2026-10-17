use crate::board::{CELLS, SYMMETRIES};
use bevy_neura_example::random::Random;

#[derive(Clone, Copy, Default)]
pub struct Transition {
    pub state: [f32; CELLS],
    pub following: [f32; CELLS],
    pub action: u8,
    pub reward: f32,
    pub done: bool,
}

pub struct Batch {
    pub states: Vec<f32>,
    pub following: Vec<f32>,
    pub actions: Vec<f32>,
    pub rewards: Vec<f32>,
    pub done: Vec<bool>,
}

impl Batch {
    fn of(rows: usize) -> Self {
        Self {
            states: Vec::with_capacity(rows * CELLS),
            following: Vec::with_capacity(rows * CELLS),
            actions: Vec::with_capacity(rows),
            rewards: Vec::with_capacity(rows),
            done: Vec::with_capacity(rows),
        }
    }
}

pub struct Replay {
    transitions: Vec<Transition>,
    cursor: usize,
    length: usize,
}

impl Replay {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "a replay of {capacity} moves holds none");
        Self {
            transitions: vec![Transition::default(); capacity],
            cursor: 0,
            length: 0,
        }
    }

    pub fn ready(&self, rows: usize) -> bool {
        self.length >= rows
    }

    pub fn push(&mut self, transition: Transition) {
        if self.length < self.transitions.len() {
            self.transitions[self.length] = transition;
            self.length += 1;
            return;
        }
        self.transitions[self.cursor] = transition;
        self.cursor = (self.cursor + 1) % self.transitions.len();
    }

    pub fn sample(&self, random: &mut Random, rows: usize) -> Batch {
        assert!(
            rows > 0 && rows <= self.length,
            "a sample of {rows} moves reads a replay of {}",
            self.length,
        );
        let mut batch = Batch::of(rows);
        for _ in 0..rows {
            let transition = self.transitions[random.below(self.length)];
            let symmetry = SYMMETRIES[random.below(SYMMETRIES.len())];
            let mut state = [0.0; CELLS];
            let mut following = [0.0; CELLS];
            for (at, (mark, after)) in transition
                .state
                .iter()
                .zip(&transition.following)
                .enumerate()
            {
                state[symmetry[at]] = *mark;
                following[symmetry[at]] = *after;
            }
            batch.states.extend_from_slice(&state);
            batch.following.extend_from_slice(&following);
            batch
                .actions
                .push(symmetry[transition.action as usize] as f32);
            batch.rewards.push(transition.reward);
            batch.done.push(transition.done);
        }
        batch
    }
}
