use crate::mel::{Mel, SAMPLE_RATE};
use crate::microphone::Microphone;
use crate::model::dims::Dims;
use crate::model::{DecoderModel, EncoderModel};
use crate::resample::Resampler;
use crate::source;
use crate::tokenizer::{self, Vocabulary};
use crate::utterance::Utterance;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy_neura::NeuraRuntime;
use std::path::Path;

const LONGEST: f32 = 28.0;
const SHORTEST: f32 = 0.3;
const ATTACK: f32 = 0.03;
const RELEASE: f32 = 0.40;

pub struct Checkpoint {
    pub dims: Dims,
    pub mel: Mel,
    pub vocabulary: Vocabulary,
    weights: Vec<u8>,
}

impl Checkpoint {
    pub fn read(directory: &Path) -> Self {
        let dims = Dims::of(directory);
        let mel = Mel::of(directory, dims.mel_bins);
        let vocabulary = Vocabulary::of(directory);
        let path = directory.join("model.safetensors");
        let weights =
            std::fs::read(&path).unwrap_or_else(|error| panic!("no {}: {error}", path.display()));
        Self {
            dims,
            mel,
            vocabulary,
            weights,
        }
    }

    pub fn weights(&self) -> &[u8] {
        &self.weights
    }
}

#[derive(Component)]
pub struct Record;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Idle,
    Recording,
    Reading,
    Writing,
}

#[derive(Resource)]
pub struct Speech {
    mel: Mel,
    vocabulary: Vocabulary,
    encoder: EncoderModel,
    decoder: DecoderModel,
    microphone: Microphone,
    resampler: Resampler,
    dims: Dims,
    stage: Stage,
    samples: Vec<f32>,
    level: f32,
    prefix: Vec<u32>,
    issued: Vec<u32>,
    language: u32,
    cursor: u32,
}

impl Speech {
    pub fn open(
        microphone: Microphone,
        checkpoint: Checkpoint,
        encoder: EncoderModel,
        decoder: DecoderModel,
    ) -> Self {
        microphone.drain();
        let dims = checkpoint.dims;
        println!(
            "{}: {} layers of {} numbers, listening at {} Hz",
            source::MODEL,
            dims.encoder_layers + dims.decoder_layers,
            dims.state,
            microphone.rate(),
        );
        let prefix = vec![0; dims.prefix() as usize];
        Self {
            resampler: Resampler::new(microphone.rate(), SAMPLE_RATE),
            mel: checkpoint.mel,
            vocabulary: checkpoint.vocabulary,
            encoder,
            decoder,
            microphone,
            stage: Stage::Idle,
            samples: Vec::new(),
            level: 0.0,
            prefix,
            issued: Vec::new(),
            language: tokenizer::SOT + 1,
            cursor: 0,
            dims,
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    fn seconds(&self) -> f32 {
        self.samples.len() as f32 / self.microphone.rate() as f32
    }

    fn begin(&mut self, block: &[f32]) {
        self.samples.clear();
        self.samples.extend_from_slice(block);
        self.stage = Stage::Recording;
    }

    fn collect(&mut self, block: &[f32]) {
        self.samples.extend_from_slice(block);
    }

    fn discard(&mut self) {
        self.samples.clear();
        self.stage = Stage::Idle;
    }

    fn track(&mut self, elapsed: f32, energy: f32) {
        let reach = if energy > self.level { ATTACK } else { RELEASE };
        self.level += (energy - self.level) * (1.0 - (-elapsed / reach).exp());
    }

    fn read(&mut self, runtime: &NeuraRuntime) {
        let seconds = self.seconds();
        let audio = self.resampler.resample(&self.samples);
        let spectrogram = self.mel.spectrogram(&audio);
        let run = self.encoder.run(runtime, spectrogram);
        let cross = self.encoder.cross(runtime);
        self.decoder.carry(runtime, &cross);
        println!(
            "{seconds:.1} s of speech, the encoder reads it in {:.2} s",
            run.seconds(),
        );
        self.samples.clear();
        self.prefix = vec![0; self.dims.prefix() as usize];
        self.prefix[0] = tokenizer::SOT;
        self.issued.clear();
        self.cursor = 0;
        self.stage = Stage::Reading;
    }

    fn step(&mut self, runtime: &NeuraRuntime) -> Option<Utterance> {
        match self.stage {
            Stage::Reading | Stage::Writing => {}
            _ => return None,
        }
        let slots = self
            .prefix
            .iter()
            .map(|token| *token as f32)
            .collect::<Vec<f32>>();
        let (token, _) = self.decoder.step(runtime, slots, self.cursor);
        match self.stage {
            Stage::Reading => {
                let logits = self.decoder.logits(runtime);
                self.language = self
                    .vocabulary
                    .languages()
                    .iter()
                    .max_by(|left, right| {
                        logits[left.0 as usize].total_cmp(&logits[right.0 as usize])
                    })
                    .map(|(token, _)| *token)
                    .expect("a vocabulary of a speech model holds a language");
                self.prefix[1] = self.language;
                self.prefix[2] = tokenizer::TRANSCRIBE;
                self.prefix[3] = tokenizer::NO_TIMESTAMPS;
                self.cursor = 3;
                self.stage = Stage::Writing;
                None
            }
            Stage::Writing => {
                if token == tokenizer::EOT || token > tokenizer::NO_TIMESTAMPS {
                    return Some(self.finish());
                }
                self.issued.push(token);
                self.cursor += 1;
                if self.cursor as usize >= self.prefix.len() {
                    return Some(self.finish());
                }
                self.prefix[self.cursor as usize] = token;
                None
            }
            _ => None,
        }
    }

    fn finish(&mut self) -> Utterance {
        let text = self.vocabulary.text(&self.issued).trim().to_string();
        let language = self
            .vocabulary
            .languages()
            .iter()
            .find(|(token, _)| *token == self.language)
            .map(|(_, code)| code.clone())
            .unwrap_or_default();
        println!("{language}: {text:?} of {} tokens", self.issued.len());
        self.issued.clear();
        self.stage = Stage::Idle;
        Utterance { text }
    }
}

pub fn capture(
    time: Res<Time>,
    runtime: Res<NeuraRuntime>,
    holding: Query<(), (With<Record>, With<Pressed>)>,
    mut speech: ResMut<Speech>,
) {
    let held = !holding.is_empty();
    let block = speech.microphone.drain();
    let energy = if block.is_empty() { 0.0 } else { rms(&block) };
    speech.track(time.delta_secs(), energy);
    match speech.stage {
        Stage::Idle => {
            if held {
                speech.begin(&block);
            }
        }
        Stage::Recording => {
            speech.collect(&block);
            if held {
                if speech.seconds() >= LONGEST {
                    speech.read(&runtime);
                }
            } else if speech.seconds() >= SHORTEST {
                speech.read(&runtime);
            } else {
                speech.discard();
            }
        }
        Stage::Reading | Stage::Writing => {}
    }
}

pub fn decode(
    runtime: Res<NeuraRuntime>,
    mut speech: ResMut<Speech>,
    mut utterances: MessageWriter<Utterance>,
) {
    if let Some(utterance) = speech.step(&runtime) {
        utterances.write(utterance);
    }
}

fn rms(block: &[f32]) -> f32 {
    (block.iter().map(|sample| sample * sample).sum::<f32>() / block.len() as f32).sqrt()
}
