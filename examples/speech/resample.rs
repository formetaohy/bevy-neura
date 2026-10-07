const TAPS: usize = 32;
const CUTOFF: f64 = 0.95;

pub struct Resampler {
    phases: Vec<f32>,
    from: u32,
    to: u32,
}

impl Resampler {
    pub fn new(from: u32, to: u32) -> Self {
        assert!(
            from > 0 && to > 0,
            "a resampler reads {from} samples a second and writes {to}",
        );
        let half = (TAPS / 2) as f64;
        let cutoff = CUTOFF * (to as f64 / from as f64).min(1.0);
        let phases = to / divisor(from, to);
        let mut table = Vec::with_capacity((phases as usize) * TAPS);
        for phase in 0..phases {
            for tap in 0..TAPS {
                let offset = phase as f64 / phases as f64 + tap as f64 - half;
                table.push((cutoff * sinc(cutoff * offset) * window(offset, half)) as f32);
            }
        }
        Self {
            phases: table,
            from,
            to,
        }
    }

    pub fn len(&self, samples: usize) -> usize {
        samples * self.to as usize / self.from as usize
    }

    pub fn resample(&self, input: &[f32]) -> Vec<f32> {
        let phases = self.phases.len() / TAPS;
        let half = (TAPS / 2) as i64;
        let mut out = Vec::with_capacity(self.len(input.len()));
        for step in 0..out.capacity() {
            let numerator = step as u64 * self.from as u64;
            let base = (numerator / self.to as u64) as i64;
            let phase = ((numerator % self.to as u64) * phases as u64 / self.to as u64) as usize;
            let taps = &self.phases[phase * TAPS..(phase + 1) * TAPS];
            let mut sum = 0.0f32;
            for (tap, weight) in taps.iter().enumerate() {
                let at = base + tap as i64 - half;
                if (0..input.len() as i64).contains(&at) {
                    sum += weight * input[at as usize];
                }
            }
            out.push(sum);
        }
        out
    }
}

fn divisor(left: u32, right: u32) -> u32 {
    let (mut left, mut right) = (left, right);
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn sinc(value: f64) -> f64 {
    if value.abs() < 1e-9 {
        1.0
    } else {
        let angle = std::f64::consts::PI * value;
        angle.sin() / angle
    }
}

fn window(offset: f64, half: f64) -> f64 {
    if offset.abs() > half {
        return 0.0;
    }
    let ratio = std::f64::consts::PI * offset / half;
    0.42 + 0.5 * ratio.cos() + 0.08 * (2.0 * ratio).cos()
}
