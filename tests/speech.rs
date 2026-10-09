use bevy::prelude::*;
use bevy_neura::{NeuraPlugin, NeuraRuntime};
use neura::{MemoryRequest, RuntimeRequest};
use safetensors::SafeTensors;
use std::path::{Path, PathBuf};

#[path = "../examples/speech/mel.rs"]
mod mel;
#[path = "../examples/speech/model/mod.rs"]
mod model;
#[path = "../examples/speech/resample.rs"]
mod resample;
#[path = "../examples/speech/tokenizer.rs"]
mod tokenizer;

use mel::Mel;
use model::dims::Dims;
use model::{DecoderModel, EncoderModel};
use tokenizer::Vocabulary;

const DEVICE_HEAP: u64 = 1 << 30;
const READBACK: u64 = 1 << 23;

fn app(heap_bytes: u64) -> App {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::new(RuntimeRequest {
        memory: MemoryRequest {
            heap_bytes,
            readback_bytes: READBACK,
            readback_slots: 4,
        },
        ..RuntimeRequest::default()
    }));
    app
}

fn data(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/speech")
        .join(name)
}

fn bytes(path: PathBuf) -> Vec<u8> {
    std::fs::read(&path).unwrap_or_else(|error| panic!("no {}: {error}", path.display()))
}

fn floats(reference: &SafeTensors<'_>, name: &str) -> Vec<f32> {
    let view = reference
        .tensor(name)
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    view.data()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|word| f32::from_le_bytes(*word))
        .collect()
}

fn assert_close(what: &str, actual: &[f32], expected: &[f32], tolerance: f32) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{what} answers {} numbers where the reference holds {}",
        actual.len(),
        expected.len(),
    );
    let error = actual
        .iter()
        .zip(expected)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0f32, f32::max);
    assert!(
        error <= tolerance,
        "{what} reads {error} away from the reference where {tolerance} holds",
    );
}

fn synthetic(count: usize) -> Vec<f32> {
    let mut entropy = 0x9E37_79B9u32;
    (0..count)
        .map(|_| {
            entropy ^= entropy << 13;
            entropy ^= entropy >> 17;
            entropy ^= entropy << 5;
            (entropy >> 8) as f32 / 16_777_216.0 - 0.5
        })
        .collect()
}

fn read_wave(path: &Path) -> Vec<f32> {
    let mut reader = hound::WavReader::open(path).expect("a wave file");
    let spec = reader.spec();
    assert_eq!(
        (spec.channels, spec.sample_rate, spec.bits_per_sample),
        (1, mel::SAMPLE_RATE, 16),
        "a wave file holds one channel of sixteen bit samples at {} Hz",
        mel::SAMPLE_RATE,
    );
    reader
        .samples::<i16>()
        .map(|sample| sample.expect("a sample") as f32 / 32768.0)
        .collect()
}

#[test]
fn a_spectrogram_matches_the_reference() {
    let directory = data("mel");
    let store = bytes(directory.join("golden.safetensors"));
    let reference = SafeTensors::deserialize(&store).expect("golden");
    let expected = floats(&reference, "mel");
    let seconds = 1;
    let frames = seconds * 100;
    let bins = expected.len() / frames;
    let mel = Mel::of(&directory, bins as u32);
    let audio = synthetic(mel::SAMPLE_RATE as usize * seconds);
    let spectrogram = mel.spectrogram(&audio);
    assert_eq!(spectrogram.len(), bins * mel.frames());
    let mut expected_bands = Vec::with_capacity(expected.len());
    for band in 0..bins {
        expected_bands.extend_from_slice(&expected[band * frames..(band + 1) * frames]);
    }
    let mut actual = Vec::with_capacity(expected.len());
    for band in 0..bins {
        actual.extend_from_slice(&spectrogram[band * mel.frames()..band * mel.frames() + frames]);
    }
    assert_close("a spectrogram", &actual, &expected_bands, 1e-3);
}

#[test]
fn a_vocabulary_names_the_pieces_of_its_model() {
    let directory = data("vocabulary");
    let ids: Vec<u32> = serde_json::from_slice(&bytes(directory.join("ids.json"))).expect("ids");
    let vocabulary = Vocabulary::of(&directory);
    let expected = std::fs::read_to_string(directory.join("expected.txt")).expect("expected text");
    assert_eq!(vocabulary.text(&ids), expected);
    assert_eq!(vocabulary.languages().len(), 2);
}

fn answers_of(
    app: &App,
    directory: &Path,
    golden: &SafeTensors<'_>,
    mel_input: Vec<f32>,
    prompt: &[u32],
    tolerance: f32,
) -> (Vec<u32>, String) {
    let runtime = app.world().resource::<NeuraRuntime>();
    let dims = Dims::of(directory);
    let weights = bytes(directory.join("model.safetensors"));
    let encoder = EncoderModel::load(runtime, &dims, &weights);
    encoder.run(runtime, mel_input);
    let cross = encoder.cross(runtime);
    for (index, (keys, values)) in cross.iter().enumerate() {
        assert_close(
            &format!("the keys of layer {index}"),
            keys,
            &floats(golden, &format!("key.{index}")),
            tolerance,
        );
        assert_close(
            &format!("the values of layer {index}"),
            values,
            &floats(golden, &format!("value.{index}")),
            tolerance,
        );
    }
    let decoder = DecoderModel::load(runtime, &dims, &weights);
    decoder.carry(runtime, &cross);
    let expected_tokens = floats(golden, "tokens");
    let expected_logits = floats(golden, "logits");
    let vocabulary = dims.vocab as usize;
    let mut prefix = vec![0u32; dims.prefix() as usize];
    prefix[..prompt.len()].copy_from_slice(prompt);
    let mut cursor = prompt.len() as u32 - 1;
    let mut issued = Vec::new();
    for step in 0..expected_tokens.len() {
        let tokens = prefix
            .iter()
            .map(|token| *token as f32)
            .collect::<Vec<f32>>();
        let (token, _) = decoder.step(runtime, tokens, cursor);
        let logits = decoder.logits(runtime);
        assert_close(
            &format!("the logits of step {step}"),
            &logits,
            &expected_logits[step * vocabulary..(step + 1) * vocabulary],
            tolerance * 10.0,
        );
        issued.push(token);
        cursor += 1;
        prefix[cursor as usize] = token;
    }
    let path = directory.join("vocab.json");
    let text = if path.exists() {
        Vocabulary::of(directory).text(&issued)
    } else {
        String::new()
    };
    (issued, text)
}

#[test]
fn a_small_model_answers_the_reference() {
    let directory = data("fixture");
    let store = bytes(data("fixture_golden.safetensors"));
    let golden = SafeTensors::deserialize(&store).expect("golden");
    let app = app(1 << 26);
    let mel_input = floats(&golden, "mel");
    let (issued, _) = answers_of(&app, &directory, &golden, mel_input, &[1, 2, 3, 4], 1e-4);
    let expected = floats(&golden, "tokens")
        .into_iter()
        .map(|token| token as u32)
        .collect::<Vec<u32>>();
    assert_eq!(issued, expected, "a reading names another token");
}

#[test]
fn a_downloaded_model_answers_the_reference() {
    let Some(reference) = std::env::var_os("SPEECH_REFERENCE_DIR").map(PathBuf::from) else {
        eprintln!("a downloaded model is checked against SPEECH_REFERENCE_DIR");
        return;
    };
    let model = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/speech/whisper-tiny");
    let store = bytes(reference.join("golden.safetensors"));
    let golden = SafeTensors::deserialize(&store).expect("golden");
    let audio = read_wave(&reference.join("speech.wav"));
    let dims = Dims::of(&model);
    let mel = Mel::of(&model, dims.mel_bins);
    let spectrogram = mel.spectrogram(&audio);
    assert_close("a spectrogram", &spectrogram, &floats(&golden, "mel"), 1e-3);
    let app = app(DEVICE_HEAP);
    let prompt = [
        tokenizer::SOT,
        tokenizer::SOT + 1,
        tokenizer::TRANSCRIBE,
        tokenizer::NO_TIMESTAMPS,
    ];
    let (issued, text) = answers_of(&app, &model, &golden, spectrogram, &prompt, 1e-3);
    let expected = std::fs::read_to_string(reference.join("transcript.txt")).expect("transcript");
    assert_eq!(text, expected);
    let expected_tokens = floats(&golden, "tokens")
        .into_iter()
        .map(|token| token as u32)
        .collect::<Vec<u32>>();
    assert_eq!(issued, expected_tokens, "a reading names another token");
}

#[test]
fn a_model_knows_silence_from_speech() {
    let model = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/speech/whisper-tiny");
    if !model.join("model.safetensors").exists() {
        eprintln!("a downloaded model is checked when target/speech holds whisper-tiny");
        return;
    }
    let mut cases = vec![(
        "three seconds of silence",
        vec![0.0; 3 * mel::SAMPLE_RATE as usize],
        false,
    )];
    if let Some(reference) = std::env::var_os("SPEECH_REFERENCE_DIR").map(PathBuf::from) {
        cases.push((
            "the reference speech",
            read_wave(&reference.join("speech.wav")),
            true,
        ));
    }
    let app = app(DEVICE_HEAP);
    let runtime = app.world().resource::<NeuraRuntime>();
    let dims = Dims::of(&model);
    let mel = Mel::of(&model, dims.mel_bins);
    let weights = bytes(model.join("model.safetensors"));
    let encoder = EncoderModel::load(runtime, &dims, &weights);
    let decoder = DecoderModel::load(runtime, &dims, &weights);
    for (what, audio, speech) in cases {
        encoder.run(runtime, mel.spectrogram(&audio));
        let cross = encoder.cross(runtime);
        decoder.carry(runtime, &cross);
        let mut prefix = vec![0u32; dims.prefix() as usize];
        prefix[0] = tokenizer::SOT;
        let prompt = prefix
            .iter()
            .map(|token| *token as f32)
            .collect::<Vec<f32>>();
        decoder.step(runtime, prompt, 0);
        assert_eq!(decoder.hears_speech(runtime), speech, "{what}");
    }
}

#[test]
fn a_resampler_carries_the_tones_it_holds_and_leaves_the_tones_it_cannot() {
    let tone = |frequency: f32, rate: u32, seconds: f32| {
        (0..(rate as f32 * seconds) as usize)
            .map(|index| {
                (std::f32::consts::TAU * frequency * index as f32 / rate as f32).sin() * 0.5
            })
            .collect::<Vec<f32>>()
    };
    let peak = |samples: &[f32]| {
        samples[1000..]
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0f32, f32::max)
    };
    let resampler = resample::Resampler::new(48000, 16000);
    let carried = resampler.resample(&tone(1000.0, 48000, 1.0));
    assert_eq!(carried.len(), 16000);
    assert!(
        (peak(&carried) - 0.5).abs() < 0.03,
        "a tone of 1000 Hz reads {} through the resampler",
        peak(&carried),
    );
    let rejected = resampler.resample(&tone(10000.0, 48000, 1.0));
    assert!(
        peak(&rejected) < 0.06,
        "a tone of 10000 Hz reads {} past the resampler",
        peak(&rejected),
    );
    let equal = resample::Resampler::new(16000, 16000).resample(&tone(1000.0, 16000, 1.0));
    assert_eq!(equal.len(), 16000);
    assert!((peak(&equal) - 0.5).abs() < 0.03);
}
