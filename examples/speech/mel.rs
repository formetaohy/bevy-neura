use rustfft::{Fft, FftPlanner, num_complex::Complex};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

pub const SAMPLE_RATE: u32 = 16000;
pub const N_FFT: usize = 400;
pub const HOP: usize = 160;
pub const CHUNK: usize = 30 * SAMPLE_RATE as usize;
pub const TAPS: usize = N_FFT / 2 + 1;
pub const FLOOR: f32 = 1e-10;
pub const SPREAD: f32 = 8.0;
pub const ORIGIN: f32 = 4.0;

pub struct Mel {
    bins: usize,
    filters: Vec<f32>,
    window: Vec<f32>,
    fft: Arc<dyn Fft<f32>>,
}

impl Mel {
    pub fn of(directory: &Path, bins: u32) -> Self {
        let path = directory.join("preprocessor_config.json");
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("the model holds no {}: {error}", path.display()));
        let config: Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{} holds no json: {error}", path.display()));
        let filters: Vec<Vec<f32>> = serde_json::from_value(
            config
                .get("mel_filters")
                .unwrap_or_else(|| panic!("{} holds no mel_filters", path.display()))
                .clone(),
        )
        .unwrap_or_else(|error| panic!("{} holds no mel filterbank: {error}", path.display()));
        assert_eq!(
            filters.len(),
            bins as usize,
            "the filterbank of {} holds {} bands where the model reads {bins}",
            path.display(),
            filters.len(),
        );
        for band in &filters {
            assert_eq!(
                band.len(),
                TAPS,
                "a band of the filterbank of {} taps {} bins where a window of {N_FFT} numbers holds {TAPS}",
                path.display(),
                band.len(),
            );
        }
        Self {
            bins: bins as usize,
            filters: filters.into_iter().flatten().collect(),
            window: (0..N_FFT)
                .map(|tap| 0.5 - 0.5 * (std::f32::consts::TAU * tap as f32 / N_FFT as f32).cos())
                .collect(),
            fft: FftPlanner::new().plan_fft_forward(N_FFT),
        }
    }

    pub fn frames(&self) -> usize {
        CHUNK / HOP
    }

    pub fn spectrogram(&self, audio: &[f32]) -> Vec<f32> {
        assert!(
            audio.len() <= CHUNK,
            "an utterance of {} samples outruns the {CHUNK} samples of {} seconds a spectrogram holds",
            audio.len(),
            CHUNK / SAMPLE_RATE as usize,
        );
        let padded = self.padded(audio);
        let frames = self.frames();
        let mut mel = vec![0.0f32; self.bins * frames];
        let mut spectrum = vec![Complex::new(0.0f32, 0.0); N_FFT];
        let mut magnitudes = [0.0f32; TAPS];
        for frame in 0..frames {
            let start = frame * HOP;
            for (tap, value) in spectrum.iter_mut().enumerate() {
                *value = Complex::new(padded[start + tap] * self.window[tap], 0.0);
            }
            self.fft.process(&mut spectrum);
            for (bin, value) in magnitudes.iter_mut().enumerate() {
                *value = spectrum[bin].norm_sqr();
            }
            for band in 0..self.bins {
                let taps = &self.filters[band * TAPS..(band + 1) * TAPS];
                let mut sum = 0.0f32;
                for (tap, magnitude) in taps.iter().zip(&magnitudes) {
                    sum += tap * magnitude;
                }
                mel[band * frames + frame] = sum;
            }
        }
        self.logarithm(&mut mel);
        mel
    }

    fn padded(&self, audio: &[f32]) -> Vec<f32> {
        let reflection = N_FFT / 2;
        let mut chunk = audio.to_vec();
        chunk.resize(CHUNK, 0.0);
        let mut padded = Vec::with_capacity(CHUNK + 2 * reflection);
        padded.extend((1..=reflection).rev().map(|step| chunk[step]));
        padded.extend_from_slice(&chunk);
        padded.extend((1..=reflection).map(|step| chunk[CHUNK - 1 - step]));
        padded
    }

    fn logarithm(&self, mel: &mut [f32]) {
        let mut largest = f32::MIN;
        for value in mel.iter_mut() {
            *value = value.max(FLOOR).log10();
            largest = largest.max(*value);
        }
        let smallest = largest - SPREAD;
        for value in mel.iter_mut() {
            *value = (*value).max(smallest);
            *value = (*value + ORIGIN) / ORIGIN;
        }
    }
}
