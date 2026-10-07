use crate::command::Command;
use crate::mel::{Mel, SAMPLE_RATE};
use crate::microphone::Microphone;
use crate::model::dims::Dims;
use crate::model::{DecoderModel, EncoderModel};
use crate::resample::Resampler;
use crate::source;
use crate::tokenizer::{self, Vocabulary};
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

pub const UTTERANCE: usize = 30;
const SILENCE: f32 = 0.004;

#[derive(Message)]
pub struct Transcription {
    pub text: String,
    pub command: Command,
}

#[derive(Resource, Default)]
pub struct PushToTalk {
    pub held: bool,
    pub pressing: bool,
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
    language: String,
    text: String,
    seconds: f32,
}

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
        let mel = Mel::of(&directory, dims.mel_bins);
        let vocabulary = Vocabulary::of(&directory);
        let weights = std::fs::read(directory.join("model.safetensors"))
            .unwrap_or_else(|error| panic!("the checkpoint of {}: {error}", directory.display()));
        let encoder = EncoderModel::load(runtime, &dims, &weights);
        let decoder = DecoderModel::load(runtime, &dims, &weights);
        let microphone = Microphone::open();
        println!(
            "{}: {} layers of {} numbers, {} mel bins, {} tokens, loaded in {:.1} s",
            source::model(),
            dims.encoder_layers + dims.decoder_layers,
            dims.state,
            dims.mel_bins,
            dims.vocab,
            started.elapsed().as_secs_f32(),
        );
        println!(
            "microphone: {} Hz, the graph reads {SAMPLE_RATE} Hz",
            microphone.rate(),
        );
        println!("say left, right, up, down, fire, freeze or restart");
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
            prefix: vec![0; 64],
            issued: Vec::new(),
            cursor: 0,
            language: "en".to_string(),
            text: String::new(),
            seconds: 0.0,
        }
    }

    pub fn state(&self) -> &'static str {
        match self.listening {
            Listening::Idle => "按住说话",
            Listening::Recording => "正在听",
            Listening::Detecting => "识别中",
            Listening::Decoding => "转写中",
        }
    }

    pub fn busy(&self) -> bool {
        !matches!(self.listening, Listening::Idle)
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn seconds(&self) -> f32 {
        self.seconds
    }

    fn begin(&mut self) {
        self.microphone.drain();
        self.utterance.clear();
        self.text.clear();
        self.listening = Listening::Recording;
        self.seconds = 0.0;
    }

    fn capture(&mut self) {
        let ceiling = UTTERANCE * self.microphone.rate() as usize;
        let heard = self.microphone.drain();
        self.seconds += heard.len() as f32 / self.microphone.rate() as f32;
        self.utterance.extend(heard);
        self.utterance.truncate(ceiling);
    }

    fn finish(&mut self, runtime: &NeuraRuntime) {
        let recording = self.seconds;
        let energy = {
            let samples = &self.utterance;
            (samples.iter().map(|sample| sample * sample).sum::<f32>()
                / samples.len().max(1) as f32)
                .sqrt()
        };
        if energy < SILENCE {
            self.listening = Listening::Idle;
            self.text.clear();
            println!(
                "{recording:.1} s of sound at rms {energy:.4}, quieter than the {SILENCE} a reading asks for"
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
        self.seconds = recording;
        println!(
            "{:.1} s of speech, the encoder reads it in {:.2} s",
            self.seconds, encoded,
        );
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
                let (best, code) = self
                    .vocabulary
                    .languages()
                    .iter()
                    .map(|(id, code)| (logits[*id as usize], code.as_str()))
                    .fold(
                        (f32::MIN, "en"),
                        |best, read| if read.0 > best.0 { read } else { best },
                    );
                let _ = best;
                self.language = code.to_string();
                let language = self
                    .vocabulary
                    .languages()
                    .iter()
                    .find(|(_, name)| name == code)
                    .map(|(id, _)| *id)
                    .expect("a language of this vocabulary");
                self.prefix[1] = language;
                self.prefix[2] = tokenizer::TRANSCRIBE;
                self.prefix[3] = tokenizer::NO_TIMESTAMPS;
                self.cursor = 3;
                self.listening = Listening::Decoding;
                None
            }
            Listening::Decoding => {
                if token == tokenizer::EOT || token > tokenizer::NO_TIMESTAMPS {
                    return self.stop();
                }
                self.issued.push(token);
                self.cursor += 1;
                if self.cursor as usize >= self.prefix.len() {
                    return self.stop();
                }
                self.prefix[self.cursor as usize] = token;
                None
            }
            _ => None,
        }
    }

    fn stop(&mut self) -> Option<Transcription> {
        self.text = self.vocabulary.text(&self.issued);
        self.listening = Listening::Idle;
        let text = self.text.trim().to_string();
        let command = Command::read(&text);
        println!("heard: {text:?} -> {}", command.name());
        Some(Transcription { text, command })
    }
}

pub fn listen(runtime: Res<NeuraRuntime>, mut speech: ResMut<Speech>, talk: Res<PushToTalk>) {
    let held = talk.held;
    match (held, &speech.listening) {
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
