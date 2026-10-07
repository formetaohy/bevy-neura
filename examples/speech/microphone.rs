use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, mpsc};

const HOLD: usize = 30;

pub struct Microphone {
    samples: Arc<Mutex<VecDeque<f32>>>,
    rate: u32,
}

impl Microphone {
    pub fn open() -> Self {
        let samples = Arc::new(Mutex::new(VecDeque::new()));
        let (ready, waiting) = mpsc::channel();
        let buffer = Arc::clone(&samples);
        std::thread::spawn(move || {
            let opened = listen(buffer);
            match &opened {
                Ok((rate, _stream)) => {
                    let _ = ready.send(Ok(*rate));
                    std::thread::park();
                }
                Err(error) => {
                    let _ = ready.send(Err(error.clone()));
                }
            }
        });
        let rate = waiting
            .recv()
            .expect("the microphone thread answers once")
            .unwrap_or_else(|error| panic!("the microphone does not open: {error}"));
        Self { samples, rate }
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn drain(&self) -> Vec<f32> {
        let mut samples = self
            .samples
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        samples.drain(..).collect()
    }
}

fn listen(buffer: Arc<Mutex<VecDeque<f32>>>) -> Result<(u32, cpal::Stream), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "no device of this host records".to_string())?;
    let supported = device
        .default_input_config()
        .map_err(|error| format!("the microphone reads no configuration: {error}"))?;
    let rate = supported.sample_rate();
    let channels = supported.channels() as usize;
    let config = supported.config();
    let hold = Arc::clone(&buffer);
    let push = move |frames: &[f32]| {
        let mut samples = hold.lock().unwrap_or_else(|error| error.into_inner());
        for frame in frames.chunks(channels) {
            samples.push_back(frame.iter().sum::<f32>() / channels as f32);
        }
        let ceiling = HOLD * rate as usize;
        while samples.len() > ceiling {
            samples.pop_front();
        }
    };
    let errors = |error| eprintln!("the microphone reports {error}");
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            config,
            move |frames: &[f32], _: &cpal::InputCallbackInfo| push(frames),
            errors,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            config,
            move |frames: &[i16], _: &cpal::InputCallbackInfo| {
                push(
                    &frames
                        .iter()
                        .map(|frame| *frame as f32 / 32768.0)
                        .collect::<Vec<f32>>(),
                )
            },
            errors,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            config,
            move |frames: &[u16], _: &cpal::InputCallbackInfo| {
                push(
                    &frames
                        .iter()
                        .map(|frame| *frame as f32 / 32768.0 - 1.0)
                        .collect::<Vec<f32>>(),
                )
            },
            errors,
            None,
        ),
        format => return Err(format!("the microphone reads {format}")),
    }
    .map_err(|error| format!("the microphone does not stream: {error}"))?;
    stream
        .play()
        .map_err(|error| format!("the microphone does not play: {error}"))?;
    Ok((rate, stream))
}
