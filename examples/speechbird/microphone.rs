use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};
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
        std::thread::spawn(move || match listen(buffer) {
            Ok((rate, stream)) => {
                let _ = ready.send(Ok(rate));
                let _held = stream;
                std::thread::park();
            }
            Err(error) => {
                let _ = ready.send(Err(error));
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
    let capture = Capture {
        samples: buffer,
        channels: supported.channels() as usize,
        ceiling: HOLD * rate as usize,
    };
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => open::<f32>(&device, supported.config(), capture),
        cpal::SampleFormat::F64 => open::<f64>(&device, supported.config(), capture),
        cpal::SampleFormat::I8 => open::<i8>(&device, supported.config(), capture),
        cpal::SampleFormat::I16 => open::<i16>(&device, supported.config(), capture),
        cpal::SampleFormat::I32 => open::<i32>(&device, supported.config(), capture),
        cpal::SampleFormat::U8 => open::<u8>(&device, supported.config(), capture),
        cpal::SampleFormat::U16 => open::<u16>(&device, supported.config(), capture),
        cpal::SampleFormat::U32 => open::<u32>(&device, supported.config(), capture),
        format => return Err(format!("the microphone reads {format}")),
    }
    .map_err(|error| format!("the microphone does not stream: {error}"))?;
    stream
        .play()
        .map_err(|error| format!("the microphone does not play: {error}"))?;
    Ok((rate, stream))
}

fn open<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    capture: Capture,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    device.build_input_stream(
        config,
        move |frames: &[T], _: &cpal::InputCallbackInfo| capture.push(frames),
        |error| eprintln!("the microphone reports {error}"),
        None,
    )
}

struct Capture {
    samples: Arc<Mutex<VecDeque<f32>>>,
    channels: usize,
    ceiling: usize,
}

impl Capture {
    fn push<T>(&self, frames: &[T])
    where
        T: SizedSample,
        f32: FromSample<T>,
    {
        let mut samples = self
            .samples
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for frame in frames.chunks(self.channels) {
            let sample = frame
                .iter()
                .map(|sample| f32::from_sample(*sample))
                .sum::<f32>();
            samples.push_back(sample / self.channels as f32);
        }
        while samples.len() > self.ceiling {
            samples.pop_front();
        }
    }
}
