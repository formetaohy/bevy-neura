use crate::env::OBSERVATION;
use crate::random::Random;

pub const ENVS: u32 = 64;
pub const STEPS: u32 = 64;
pub const ROWS: u32 = ENVS * STEPS;
const FLOOR: f32 = 1e-8;

pub struct Record {
    pub observation: [f32; OBSERVATION],
    pub following: [f32; OBSERVATION],
    pub action: f32,
    pub log_probability: f32,
    pub value: f32,
    pub reward: f32,
    pub terminated: bool,
    pub truncated: bool,
}

#[derive(Default)]
pub struct Batch {
    pub observations: Vec<f32>,
    pub actions: Vec<f32>,
    pub advantages: Vec<f32>,
    pub targets: Vec<f32>,
    pub log_probabilities: Vec<f32>,
}

pub struct Rollout {
    observations: Vec<f32>,
    actions: Vec<f32>,
    log_probabilities: Vec<f32>,
    values: Vec<f32>,
    rewards: Vec<f32>,
    terminated: Vec<f32>,
    advantages: Vec<f32>,
    targets: Vec<f32>,
    patched: Vec<Option<f32>>,
    terminal_observations: Vec<f32>,
    terminal_rows: Vec<u32>,
    order: Vec<u32>,
    cursor: usize,
}

impl Default for Rollout {
    fn default() -> Self {
        Self {
            observations: vec![0.0; ROWS as usize * OBSERVATION],
            actions: vec![0.0; ROWS as usize],
            log_probabilities: vec![0.0; ROWS as usize],
            values: vec![0.0; ROWS as usize],
            rewards: vec![0.0; ROWS as usize],
            terminated: vec![0.0; ROWS as usize],
            advantages: vec![0.0; ROWS as usize],
            targets: vec![0.0; ROWS as usize],
            patched: vec![None; ROWS as usize],
            terminal_observations: Vec::new(),
            terminal_rows: Vec::new(),
            order: (0..ROWS).collect(),
            cursor: 0,
        }
    }
}

impl Rollout {
    pub fn full(&self) -> bool {
        self.cursor == ROWS as usize
    }

    pub fn clear(&mut self) {
        self.terminal_observations.clear();
        self.terminal_rows.clear();
        self.patched.fill(None);
        self.cursor = 0;
    }

    pub fn record(&mut self, record: Record) {
        assert!(!self.full(), "a rollout of {ROWS} steps takes no more");
        let at = self.cursor;
        self.observations[at * OBSERVATION..(at + 1) * OBSERVATION]
            .copy_from_slice(&record.observation);
        self.actions[at] = record.action;
        self.log_probabilities[at] = record.log_probability;
        self.values[at] = record.value;
        self.rewards[at] = record.reward;
        self.terminated[at] = if record.terminated { 1.0 } else { 0.0 };
        if record.truncated && !record.terminated {
            self.terminal_observations
                .extend_from_slice(&record.following);
            self.terminal_rows.push(at as u32);
        }
        self.cursor += 1;
    }

    pub fn terminal_observations(&self) -> &[f32] {
        &self.terminal_observations
    }

    pub fn accept_bootstrap(&mut self, values: &[f32]) {
        assert_eq!(
            values.len(),
            self.terminal_rows.len(),
            "a rollout bootstraps {} episodes with {} values",
            self.terminal_rows.len(),
            values.len(),
        );
        for (row, value) in self.terminal_rows.iter().zip(values) {
            self.patched[*row as usize] = Some(*value);
        }
    }

    pub fn finish(&mut self, last_values: &[f32], gamma: f32, lambda: f32) {
        assert!(self.full(), "a rollout of {} steps ends full", self.cursor);
        assert_eq!(
            last_values.len(),
            ENVS as usize,
            "a rollout bootstraps {ENVS} environments with {} values",
            last_values.len(),
        );
        let mut running = vec![0.0f32; ENVS as usize];
        for step in (0..STEPS as usize).rev() {
            for env in 0..ENVS as usize {
                let row = step * ENVS as usize + env;
                let value = self.values[row];
                let following = match self.patched[row] {
                    Some(bootstrap) => bootstrap,
                    None if step + 1 < STEPS as usize => self.values[row + ENVS as usize],
                    None => last_values[env],
                };
                let alive = 1.0 - self.terminated[row];
                let delta = self.rewards[row] + gamma * following * alive - value;
                running[env] = delta + gamma * lambda * alive * running[env];
                self.advantages[row] = running[env];
                self.targets[row] = running[env] + value;
            }
        }
        let count = ROWS as f32;
        let mean = self.advantages.iter().sum::<f32>() / count;
        let variance = self
            .advantages
            .iter()
            .map(|advantage| (advantage - mean).powi(2))
            .sum::<f32>()
            / count;
        let deviation = variance.sqrt() + FLOOR;
        for advantage in &mut self.advantages {
            *advantage = (*advantage - mean) / deviation;
        }
    }

    pub fn shuffle(&mut self, random: &mut Random) {
        for index in (1..self.order.len()).rev() {
            let swap = random.below(index + 1);
            self.order.swap(index, swap);
        }
    }

    pub fn order(&self) -> &[u32] {
        &self.order
    }

    pub fn gather(&self, batch: &mut Batch, rows: &[u32]) {
        batch.observations.clear();
        batch.actions.clear();
        batch.advantages.clear();
        batch.targets.clear();
        batch.log_probabilities.clear();
        for row in rows {
            let at = *row as usize;
            batch
                .observations
                .extend_from_slice(&self.observations[at * OBSERVATION..(at + 1) * OBSERVATION]);
            batch.actions.push(self.actions[at]);
            batch.advantages.push(self.advantages[at]);
            batch.targets.push(self.targets[at]);
            batch.log_probabilities.push(self.log_probabilities[at]);
        }
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn targets(&self) -> &[f32] {
        &self.targets
    }
}
