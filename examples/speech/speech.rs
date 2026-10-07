use crate::mel::{Mel, SAMPLE_RATE};
use crate::microphone::Microphone;
use crate::model::dims::Dims;
use crate::model::{DecoderModel, EncoderModel};
use crate::resample::Resampler;
use crate::source;
use crate::tokenizer::{self, Vocabulary};
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const UTTERANCE: usize = 30;
const SILENCE: f32 = 0.004;

#[derive(Message)]
pub struct Transcription {
    pub text: String,
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
    listening: Listening,
    utterance: Vec<f32>,
    prefix: Vec<u32>,
    issued: Vec<u32>,
    cursor: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Listening {
    Idle,
    Recording,
    Detecting,
    Decoding,
}

impl Speech {
    pub fn load(runtime: &NeuraRuntime) -> Self {
        let directory = source::directory();
        source::ensure(&directory);
        let started = std::time::Instant::now();
        let dims = Dims::of(&directory);
        let weights = std::fs::read(directory.join("model.safetensors"))
            .unwrap_or_else(|error| panic!("the checkpoint of {}: {error}", directory.display()));
        let encoder = EncoderModel::load(runtime, &dims, &weights);
        let decoder = DecoderModel::load(runtime, &dims, &weights);
        let mel = Mel::of(&directory, dims.mel_bins);
        let vocabulary = Vocabulary::of(&directory);
        let microphone = Microphone::open();
        println!(
            "{}: {} layers of {} numbers, loaded in {:.1} s",
            source::MODEL,
            dims.encoder_layers + dims.decoder_layers,
            dims.state,
            started.elapsed().as_secs_f32(),
        );
        let prefix = vec![0; dims.prefix() as usize];
        Self {
            resampler: Resampler::new(microphone.rate(), SAMPLE_RATE),
            mel,
            vocabulary,
            encoder,
            decoder,
            microphone,
            dims,
            listening: Listening::Idle,
            utterance: Vec::new(),
            prefix,
            issued: Vec::new(),
            cursor: 0,
        }
    }

    pub fn state(&self) -> &'static str {
        match self.listening {
            Listening::Idle => "idle",
            Listening::Recording => "listening",
            Listening::Detecting => "reading",
            Listening::Decoding => "transcribing",
        }
    }

    pub fn busy(&self) -> bool {
        !matches!(self.listening, Listening::Idle)
    }

    fn begin(&mut self) {
        self.microphone.drain();
        self.utterance.clear();
        self.listening = Listening::Recording;
    }

    fn capture(&mut self) {
        let ceiling = UTTERANCE * self.microphone.rate() as usize;
        let heard = self.microphone.drain();
        self.utterance.extend(heard);
        self.utterance.truncate(ceiling);
    }

    fn finish(&mut self, runtime: &NeuraRuntime) {
        let seconds = self.utterance.len() as f32 / self.microphone.rate() as f32;
        let energy = (self
            .utterance
            .iter()
            .map(|sample| sample * sample)
            .sum::<f32>()
            / self.utterance.len().max(1) as f32)
            .sqrt();
        if energy < SILENCE {
            self.listening = Listening::Idle;
            println!(
                "{seconds:.1} s of sound at rms {energy:.4}, quieter than the {SILENCE} a reading asks for"
            );
            return;
        }
        let start = std::time::Instant::now();
        let audio = self.resampler.resample(&self.utterance);
        let spectrogram = self.mel.spectrogram(&audio);
        let run = self.encoder.run(runtime, spectrogram);
        let cross = self.encoder.cross(runtime);
        self.decoder.carry(runtime, &cross);
        let encoded = run.seconds() + start.elapsed().as_secs_f64();
        self.prefix = vec![0; self.dims.prefix() as usize];
        self.prefix[0] = tokenizer::SOT;
        self.issued.clear();
        self.cursor = 0;
        self.listening = Listening::Detecting;
        println!("{seconds:.1} s of speech, the encoder reads it in {encoded:.2} s");
    }

    fn advance(&mut self, runtime: &NeuraRuntime) -> Option<Transcription> {
        let slots = self
            .prefix
            .iter()
            .map(|token| *token as f32)
            .collect::<Vec<f32>>();
        let (token, _) = self.decoder.step(runtime, slots, self.cursor);
        match self.listening {
            Listening::Detecting => {
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
                self.listening = Listening::Decoding;
                None
            }
            Listening::Decoding => {
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

    fn stop(&mut self) -> Transcription {
        let text = self.vocabulary.text(&self.issued).trim().to_string();
        self.listening = Listening::Idle;
        println!("heard: {text:?}");
        Transcription { text }
    }
}

pub fn listen(
    keys: Res<ButtonInput<KeyCode>>,
    runtime: Res<NeuraRuntime>,
    mut speech: ResMut<Speech>,
) {
    match (keys.pressed(KeyCode::Space), speech.listening) {
        (true, Listening::Idle) => speech.begin(),
        (true, Listening::Recording) => speech.capture(),
        (false, Listening::Recording) => speech.finish(&runtime),
        _ => {}
    }
}

pub fn decode(
    runtime: Res<NeuraRuntime>,
    mut speech: ResMut<Speech>,
    mut heard: MessageWriter<Transcription>,
) {
    if let Some(transcription) = speech.advance(&runtime) {
        heard.write(transcription);
    }
}
