use crate::dataset::Dataset;
use crate::digit;
use crate::learner::{BATCH, Learner};
use crate::recognizer::Recognizer;
use bevy_neura::NeuraRuntime;
use bevy_neura_example::random::Random;
use neura::CheckpointFile;
use std::path::{Path, PathBuf};

const SCHEDULE: [f32; 6] = [1e-3, 1e-3, 1e-3, 3e-4, 3e-4, 1e-4];
pub const EPOCHS: u32 = SCHEDULE.len() as u32;
pub const VALIDATION: u32 = 1000;
const SEED: u64 = 0x5eed_0002;

pub enum Progress {
    Working(f32),
    Done,
}

enum Stage {
    Learn { epoch: u32, cursor: u32 },
    Validate { epoch: u32, cursor: u32, right: u32 },
    Restore(PathBuf),
    Done,
}

pub struct Training {
    dataset: Option<Dataset>,
    learner: Option<Learner>,
    recognizer: Recognizer,
    stage: Stage,
    random: Random,
    order: Vec<u32>,
    lines: Vec<f32>,
    answers: Vec<f32>,
    digits: Vec<f32>,
    judged: Vec<u8>,
    loss: f32,
}

impl Training {
    pub fn learning(runtime: &NeuraRuntime, dataset: Dataset) -> Self {
        Self::of(
            runtime,
            Some(dataset),
            Stage::Learn {
                epoch: 0,
                cursor: 0,
            },
        )
    }

    pub fn loaded(runtime: &NeuraRuntime, checkpoint: &Path) -> Self {
        Self::of(runtime, None, Stage::Restore(checkpoint.to_path_buf()))
    }

    fn of(runtime: &NeuraRuntime, dataset: Option<Dataset>, stage: Stage) -> Self {
        let learner = Learner::attach(runtime);
        let recognizer = Recognizer::share(runtime, learner.model());
        let side = digit::SIDE * digit::SIDE;
        Self {
            dataset,
            learner: Some(learner),
            recognizer,
            stage,
            random: Random::seeded(SEED),
            order: Vec::new(),
            lines: vec![0.0; BATCH as usize * side],
            answers: vec![0.0; BATCH as usize],
            digits: vec![0.0; VALIDATION as usize * side],
            judged: vec![0; VALIDATION as usize],
            loss: 0.0,
        }
    }

    pub fn advance(&mut self, runtime: &NeuraRuntime) -> Progress {
        let stage = std::mem::replace(&mut self.stage, Stage::Done);
        match stage {
            Stage::Learn { epoch, cursor } => self.learn(runtime, epoch, cursor),
            Stage::Validate {
                epoch,
                cursor,
                right,
            } => self.validate(runtime, epoch, cursor, right),
            Stage::Restore(checkpoint) => self.restore(runtime, &checkpoint),
            Stage::Done => Progress::Done,
        }
    }

    pub fn save(&self, runtime: &NeuraRuntime, path: &Path) {
        runtime.save(
            self.learner
                .as_ref()
                .expect("a training saves the learning it holds")
                .model()
                .weights(),
            path,
        );
    }

    pub fn recognizer(self) -> Recognizer {
        self.recognizer
    }

    fn learn(&mut self, runtime: &NeuraRuntime, epoch: u32, cursor: u32) -> Progress {
        let steps = self.steps();
        assert!(
            cursor < steps,
            "an epoch of {steps} batches holds no batch {cursor}",
        );
        if cursor == 0 {
            self.shuffle();
        }
        self.fill(cursor);
        let loss = self
            .learner
            .as_ref()
            .expect("a learning holds the learner it steps")
            .step(
                runtime,
                &self.lines,
                &self.answers,
                SCHEDULE[epoch as usize],
            );
        self.loss += loss;
        let cursor = cursor + 1;
        self.stage = if cursor < steps {
            Stage::Learn { epoch, cursor }
        } else {
            Stage::Validate {
                epoch,
                cursor: 0,
                right: 0,
            }
        };
        Progress::Working(self.progress(epoch, cursor as f32))
    }

    fn validate(
        &mut self,
        runtime: &NeuraRuntime,
        epoch: u32,
        cursor: u32,
        mut right: u32,
    ) -> Progress {
        let dataset = self
            .dataset
            .as_ref()
            .expect("a validation reads the digits it holds");
        let side = digit::SIDE * digit::SIDE;
        let rows = (VALIDATION as usize).min(self.test_size() as usize - cursor as usize);
        for row in 0..rows {
            let (digits, answer) = dataset.test(cursor + row as u32);
            let line = &mut self.digits[row * side..(row + 1) * side];
            for (at, value) in digits.iter().enumerate() {
                line[at] = f32::from(*value) / 255.0;
            }
            self.judged[row] = answer;
        }
        for (row, guess) in self
            .recognizer
            .read(runtime, &self.digits[..rows * side])
            .into_iter()
            .enumerate()
        {
            if guess.digit == self.judged[row] {
                right += 1;
            }
        }
        let cursor = cursor + rows as u32;
        if cursor < dataset.test_size() {
            self.stage = Stage::Validate {
                epoch,
                cursor,
                right,
            };
            return Progress::Working(
                self.progress(epoch, self.steps() as f32 + chunks(cursor) as f32),
            );
        }
        let accuracy = f64::from(right) / f64::from(dataset.test_size());
        let loss = self.loss / self.steps() as f32;
        self.loss = 0.0;
        println!(
            "epoch {} of {EPOCHS}: the batches held a loss of {loss:.4}, the held out digits read right {:.2}%",
            epoch + 1,
            accuracy * 100.0,
        );
        let epoch = epoch + 1;
        self.stage = if epoch < EPOCHS {
            Stage::Learn { epoch, cursor: 0 }
        } else {
            Stage::Done
        };
        Progress::Working(self.progress(epoch, 0.0))
    }

    fn restore(&mut self, runtime: &NeuraRuntime, checkpoint: &Path) -> Progress {
        let learner = self
            .learner
            .as_ref()
            .expect("a restore fills the learner it holds");
        runtime.restore(learner.model().weights(), &CheckpointFile::open(checkpoint));
        self.dataset = None;
        self.stage = Stage::Done;
        Progress::Done
    }

    fn steps(&self) -> u32 {
        self.dataset
            .as_ref()
            .expect("a learning walks the batches of the digits it holds")
            .train_size()
            / BATCH
    }

    fn progress(&self, epoch: u32, done: f32) -> f32 {
        let units = (self.steps() + self.test_chunks()) as f32;
        ((epoch as f32 * units + done) / (EPOCHS as f32 * units)).clamp(0.0, 1.0)
    }

    fn test_size(&self) -> u32 {
        self.dataset
            .as_ref()
            .expect("a validation reads the digits it holds")
            .test_size()
    }

    fn test_chunks(&self) -> u32 {
        chunks(self.test_size())
    }

    fn shuffle(&mut self) {
        let size = self
            .dataset
            .as_ref()
            .expect("a shuffle orders the digits it holds")
            .train_size();
        self.order = (0..size).collect();
        for at in (1..self.order.len()).rev() {
            let swap = self.random.below(at + 1);
            self.order.swap(at, swap);
        }
    }

    fn fill(&mut self, cursor: u32) {
        let dataset = self
            .dataset
            .as_ref()
            .expect("a batch reads the digits it holds");
        let side = digit::SIDE * digit::SIDE;
        let start = (cursor * BATCH) as usize;
        for row in 0..BATCH as usize {
            let (digits, answer) = dataset.train(self.order[start + row]);
            let line = &mut self.lines[row * side..(row + 1) * side];
            for (at, value) in digits.iter().enumerate() {
                line[at] = f32::from(*value) / 255.0;
            }
            self.answers[row] = f32::from(answer);
        }
    }
}

fn chunks(digits: u32) -> u32 {
    digits.div_ceil(VALIDATION)
}
