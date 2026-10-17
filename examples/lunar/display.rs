use crate::env::{Ending, Lander};
use crate::policy::{ACTIONS, Greedy};
use crate::train::Training;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const SEED: u64 = 0x1u64;
const BANNER: u32 = 110;

#[derive(Resource)]
pub struct Display {
    env: Lander,
    greedy: Greedy,
    pub logits: [f32; ACTIONS],
    pub landed: u64,
    pub episodes: u64,
    pub ending: Option<Ending>,
    age: u32,
}

impl Display {
    pub fn build(runtime: &NeuraRuntime, training: &Training) -> Self {
        Self {
            env: Lander::new(SEED, true),
            greedy: Greedy::share(runtime, training.model()),
            logits: [0.0; ACTIONS],
            landed: 0,
            episodes: 0,
            ending: None,
            age: 0,
        }
    }

    pub fn env(&self) -> &Lander {
        &self.env
    }

    pub fn replay(&mut self) {
        self.ending = None;
        self.age = 0;
        self.env.reset();
    }

    pub fn step(&mut self, runtime: &NeuraRuntime) {
        let observation = self.env.observation();
        let guess = self.greedy.choose(runtime, &observation);
        self.logits = guess.logits;
        let moved = self.env.step(guess.action);
        self.env.fade();
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
            if self.ending == Some(Ending::Landed) {
                self.landed += 1;
            }
            self.env.reset();
        }
    }
}

pub fn advance(runtime: Res<NeuraRuntime>, mut display: ResMut<Display>) {
    display.step(&runtime);
}
