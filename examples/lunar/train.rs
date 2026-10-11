use crate::env::{Action, Lander, OBSERVATION};
use crate::learner::{Diagnostics, GAMMA, LAMBDA, Learner};
use crate::policy::Sampler;
use crate::random::Random;
use crate::rollout::{ENVS, Record, Rollout};
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;
use std::time::Duration;

const SEED: u64 = 0x5eed_0001;
const HISTORY: usize = 128;
const RECENT: usize = 50;
pub const SLOW: u32 = 8;
pub const TURBO: u32 = 32;

#[derive(Resource)]
pub struct Training {
    envs: Vec<Lander>,
    sampler: Sampler,
    learner: Learner,
    rollout: Rollout,
    random: Random,
    plane: Vec<f32>,
    returned: Vec<f32>,
    pub steps: u32,
    pub history: Vec<f32>,
    pub recent: Vec<f32>,
    pub best: f32,
    pub episodes: u64,
    pub samples: u64,
    pub updates: u64,
    pub diagnostics: Diagnostics,
    pub explained: f32,
    clock: Duration,
    measured: u64,
    pub steps_per_second: f32,
}

impl Training {
    pub fn of(learner: Learner, sampler: Sampler) -> Self {
        let mut random = Random::seeded(SEED);
        let envs = (0..ENVS)
            .map(|_| Lander::new(random.bits() as u64, false))
            .collect();
        Self {
            envs,
            sampler,
            learner,
            rollout: Rollout::default(),
            random,
            plane: vec![0.0; ENVS as usize * OBSERVATION],
            returned: vec![0.0; ENVS as usize],
            steps: SLOW,
            history: Vec::new(),
            recent: Vec::new(),
            best: f32::MIN,
            episodes: 0,
            samples: 0,
            updates: 0,
            diagnostics: Diagnostics::default(),
            explained: 0.0,
            clock: Duration::ZERO,
            measured: 0,
            steps_per_second: 0.0,
        }
    }

    pub fn model(&self) -> &bevy_neura::Model {
        self.learner.model()
    }

    pub fn mean(&self) -> f32 {
        if self.recent.is_empty() {
            return 0.0;
        }
        self.recent.iter().sum::<f32>() / self.recent.len() as f32
    }

    pub fn advance(&mut self, runtime: &NeuraRuntime, elapsed: Duration) {
        self.clock += elapsed;
        for _ in 0..self.steps {
            self.vector_step(runtime);
        }
        if self.clock >= Duration::from_millis(500) {
            self.steps_per_second =
                (self.samples - self.measured) as f32 / self.clock.as_secs_f32();
            self.measured = self.samples;
            self.clock = Duration::ZERO;
        }
    }

    fn observe(&mut self) {
        for env in 0..ENVS as usize {
            self.plane[env * OBSERVATION..(env + 1) * OBSERVATION]
                .copy_from_slice(&self.envs[env].observation());
        }
    }

    fn vector_step(&mut self, runtime: &NeuraRuntime) {
        self.observe();
        let seed = f32::from_bits(self.random.bits());
        let sample = self.sampler.sample(runtime, &self.plane, seed);
        for env in 0..ENVS as usize {
            let action = Action::of(sample.actions[env] as usize);
            let before = env * OBSERVATION;
            let observation: [f32; OBSERVATION] = self.plane[before..before + OBSERVATION]
                .try_into()
                .expect("an observation holds eight numbers");
            let moved = self.envs[env].step(action);
            self.rollout.record(Record {
                observation,
                following: moved.observation,
                action: sample.actions[env],
                log_probability: sample.log_probabilities[env],
                value: sample.values[env],
                reward: moved.reward,
                terminated: moved.terminated,
                truncated: moved.truncated,
            });
            self.returned[env] += moved.reward;
            if moved.terminated || moved.truncated {
                self.episode(self.returned[env]);
                self.returned[env] = 0.0;
                self.envs[env].reset();
            }
        }
        self.samples += u64::from(ENVS);
        if self.rollout.full() {
            self.learn(runtime);
        }
    }

    fn learn(&mut self, runtime: &NeuraRuntime) {
        let terminal = self.rollout.terminal_observations().to_vec();
        if !terminal.is_empty() {
            let seed = f32::from_bits(self.random.bits());
            let values = self.sampler.value(runtime, &terminal, seed);
            self.rollout.accept_bootstrap(&values);
        }
        self.observe();
        let seed = f32::from_bits(self.random.bits());
        let last = self.sampler.value(runtime, &self.plane, seed);
        self.rollout.finish(&last, GAMMA, LAMBDA);
        self.explained = explained(self.rollout.values(), self.rollout.targets());
        self.diagnostics = self
            .learner
            .update(runtime, &mut self.rollout, &mut self.random);
        self.updates += 1;
        self.rollout.clear();
    }

    fn episode(&mut self, reward: f32) {
        self.episodes += 1;
        self.best = self.best.max(reward);
        self.recent.push(reward);
        if self.recent.len() > RECENT {
            self.recent.remove(0);
        }
        self.history.push(reward);
        if self.history.len() > HISTORY {
            self.history.remove(0);
        }
    }
}

fn explained(values: &[f32], targets: &[f32]) -> f32 {
    let count = targets.len() as f32;
    let mean = targets.iter().sum::<f32>() / count;
    let variance = targets
        .iter()
        .map(|target| (target - mean).powi(2))
        .sum::<f32>()
        / count;
    let error = values
        .iter()
        .zip(targets)
        .map(|(value, target)| (value - target).powi(2))
        .sum::<f32>()
        / count;
    if variance < 1e-6 {
        return 0.0;
    }
    1.0 - error / variance
}

pub fn advance(time: Res<Time>, runtime: Res<NeuraRuntime>, mut training: ResMut<Training>) {
    training.advance(&runtime, time.delta());
}
