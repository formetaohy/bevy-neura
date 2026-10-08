use crate::env::{Ending, Lander};
use crate::policy::{ACTIONS, Greedy};
use crate::train::Training;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const SEED: u64 = 0x1u64;
const HISTORY: usize = 32;
const BANNER: u32 = 110;

#[derive(Resource)]
pub struct Display {
    env: Lander,
    greedy: Greedy,
    pub value: f32,
    pub logits: [f32; ACTIONS],
    pub returned: f32,
    pub history: Vec<f32>,
    pub best: f32,
    pub episodes: u64,
    pub ending: Option<Ending>,
    age: u32,
}

impl Display {
    pub fn build(runtime: &NeuraRuntime, training: &Training) -> Self {
        Self {
            env: Lander::new(SEED, true),
            greedy: Greedy::share(runtime, training.model()),
            value: 0.0,
            logits: [0.0; ACTIONS],
            returned: 0.0,
            history: Vec::new(),
            best: f32::MIN,
            episodes: 0,
            ending: None,
            age: 0,
        }
    }

    pub fn env(&self) -> &Lander {
        &self.env
    }

    pub fn replay(&mut self) {
        self.returned = 0.0;
        self.ending = None;
        self.age = 0;
        self.env.reset();
    }

    pub fn mean(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().sum::<f32>() / self.history.len() as f32
    }
    pub fn fresh(&self) -> bool {
        self.ending.is_some()
    }

    pub fn step(&mut self, runtime: &NeuraRuntime) {
        let observation = self.env.observation();
        let guess = self.greedy.choose(runtime, &observation);
        self.value = guess.value;
        self.logits = guess.logits;
        let moved = self.env.step(guess.action);
        self.env.fade();
        self.returned += moved.reward;
        if self.ending.is_some() {
            self.age += 1;
            if self.age >= BANNER {
                self.ending = None;
            }
        }
        if moved.terminated || moved.truncated {
            self.ending = self.env.ending();
            self.age = 0;
            self.episodes += 1;
            self.best = self.best.max(self.returned);
            self.history.push(self.returned);
            if self.history.len() > HISTORY {
                self.history.remove(0);
            }
            self.returned = 0.0;
            self.env.reset();
        }
    }
}

pub fn build(runtime: Res<NeuraRuntime>, training: Res<Training>, mut commands: Commands) {
    commands.insert_resource(Display::build(&runtime, &training));
}

pub fn advance(runtime: Res<NeuraRuntime>, mut display: ResMut<Display>) {
    display.step(&runtime);
}
