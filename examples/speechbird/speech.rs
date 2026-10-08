use crate::mel::{Mel, SAMPLE_RATE};
use crate::microphone::Microphone;
use crate::model::dims::Dims;
use crate::model::{DecoderModel, EncoderModel};
use crate::resample::Resampler;
use crate::source;
use crate::tokenizer::{self, Vocabulary};
use crate::utterance::Utterance;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;
use std::path::Path;

const START: f32 = 0.01;
const STOP: f32 = 0.006;
const QUIET: f32 = 0.35;
const LONGEST: f32 = 5.0;
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

#[derive(Resource)]
pub struct Speech {
    mel: Mel,
    vocabulary: Vocabulary,
    encoder: EncoderModel,
    decoder: DecoderModel,
    microphone: Microphone,
    resampler: Resampler,
    dims: Dims,
    state: State,
    level: f32,
    utterance: Vec<f32>,
    quiet: f32,
    prefix: Vec<u32>,
    issued: Vec<u32>,
    cursor: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Idle,
    Recording,
    Reading,
    Transcribing,
}

impl Speech {
    pub fn open(
        microphone: Microphone,
        checkpoint: Checkpoint,
        encoder: EncoderModel,
        decoder: DecoderModel,
    ) -> Self {
        microphone.drain();
        let prefix = vec![0; checkpoint.dims.prefix() as usize];
        println!(
            "{}: {} layers of {} numbers, listening at {} Hz",
            source::MODEL,
            checkpoint.dims.encoder_layers + checkpoint.dims.decoder_layers,
            checkpoint.dims.state,
            microphone.rate(),
        );
        Self {
            resampler: Resampler::new(microphone.rate(), SAMPLE_RATE),
            mel: checkpoint.mel,
            vocabulary: checkpoint.vocabulary,
            encoder,
            decoder,
            microphone,
            dims: checkpoint.dims,
            state: State::Idle,
            level: 0.0,
            utterance: Vec::new(),
            quiet: 0.0,
            prefix,
            issued: Vec::new(),
            cursor: 0,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    fn begin(&mut self, block: &[f32]) {
        self.utterance.clear();
        self.utterance.extend_from_slice(block);
        self.quiet = 0.0;
        self.state = State::Recording;
    }

    fn track(&mut self, seconds: f32, energy: f32) {
        let reach = if energy > self.level { ATTACK } else { RELEASE };
        self.level += (energy - self.level) * (1.0 - (-seconds / reach).exp());
    }

    fn capture(&mut self, block: &[f32]) {
        self.utterance.extend_from_slice(block);
    }

    fn elapse(&mut self, seconds: f32, loud: bool) {
        self.quiet = if loud { 0.0 } else { self.quiet + seconds };
    }

    fn settled(&self) -> bool {
        let seconds = self.utterance.len() as f32 / self.microphone.rate() as f32;
        self.quiet >= QUIET || seconds >= LONGEST
    }

    fn finish(&mut self, runtime: &NeuraRuntime) {
        let seconds = self.utterance.len() as f32 / self.microphone.rate() as f32;
        let started = std::time::Instant::now();
        let audio = self.resampler.resample(&self.utterance);
        let spectrogram = self.mel.spectrogram(&audio);
        let run = self.encoder.run(runtime, spectrogram);
        let cross = self.encoder.cross(runtime);
        self.decoder.carry(runtime, &cross);
        let encoded = run.seconds() + started.elapsed().as_secs_f64();
        self.prefix = vec![0; self.dims.prefix() as usize];
        self.prefix[0] = tokenizer::SOT;
        self.issued.clear();
        self.cursor = 0;
        self.state = State::Reading;
        println!("{seconds:.1} s of speech, the encoder reads it in {encoded:.2} s");
    }

    fn advance(&mut self, runtime: &NeuraRuntime) -> Option<Utterance> {
        let slots = self
            .prefix
            .iter()
            .map(|token| *token as f32)
            .collect::<Vec<f32>>();
        let (token, _) = self.decoder.step(runtime, slots, self.cursor);
        match self.state {
            State::Reading => {
                let logits = self.decoder.logits(runtime);
                let language = self
                    .vocabulary
                    .languages()
                    .iter()
                    .max_by(|left, right| {
                        logits[left.0 as usize].total_cmp(&logits[right.0 as usize])
                    })
                    .map(|(token, _)| *token)
                    .expect("a vocabulary of a speech model holds a language");
                self.prefix[1] = language;
                self.prefix[2] = tokenizer::TRANSCRIBE;
                self.prefix[3] = tokenizer::NO_TIMESTAMPS;
                self.cursor = 3;
                self.state = State::Transcribing;
                None
            }
            State::Transcribing => {
                if token == tokenizer::EOT || token > tokenizer::NO_TIMESTAMPS {
                    return Some(self.stop());
                }
                self.issued.push(token);
                self.cursor += 1;
                if self.cursor as usize >= self.prefix.len() {
                    return Some(self.stop());
                }
                self.prefix[self.cursor as usize] = token;
                None
            }
            _ => None,
        }
    }

    fn stop(&mut self) -> Utterance {
        let text = self.vocabulary.text(&self.issued).trim().to_string();
        self.state = State::Idle;
        println!("heard: {text:?}");
        Utterance { text }
    }
}

pub fn listen(time: Res<Time>, runtime: Res<NeuraRuntime>, mut speech: ResMut<Speech>) {
    let seconds = time.delta_secs();
    let block = speech.microphone.drain();
    let energy = if block.is_empty() { 0.0 } else { rms(&block) };
    speech.track(seconds, energy);
    if block.is_empty() {
        if speech.state == State::Recording {
            speech.elapse(seconds, false);
            if speech.settled() {
                speech.finish(&runtime);
            }
        }
        return;
    }
    match speech.state {
        State::Idle => {
            if energy >= START {
                speech.begin(&block);
            }
        }
        State::Recording => {
            let seconds = block.len() as f32 / speech.microphone.rate() as f32;
            speech.capture(&block);
            speech.elapse(seconds, energy >= STOP);
            if speech.settled() {
                speech.finish(&runtime);
            }
        }
        State::Reading | State::Transcribing => {}
    }
}

pub fn decode(
    runtime: Res<NeuraRuntime>,
    mut speech: ResMut<Speech>,
    mut utterances: MessageWriter<Utterance>,
) {
    if let Some(utterance) = speech.advance(&runtime) {
        utterances.write(utterance);
    }
}

fn rms(block: &[f32]) -> f32 {
    (block.iter().map(|sample| sample * sample).sum::<f32>() / block.len() as f32).sqrt()
}
