const MULTIPLIER: u128 = 0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645;
const UNIT: f64 = 1.0 / (1u64 << 53) as f64;

pub struct Random {
    state: u128,
    stream: u128,
}

impl Random {
    pub fn seeded(seed: u64) -> Self {
        let mut mixer = Mixer(seed);
        let state = u128::from(mixer.next()) | (u128::from(mixer.next()) << 64);
        let stream = u128::from(mixer.next()) | (u128::from(mixer.next()) << 64);
        let mut random = Self {
            state,
            stream: (stream << 1) | 1,
        };
        random.word();
        random
    }

    pub fn unit(&mut self) -> f64 {
        (self.word() >> 11) as f64 * UNIT
    }

    pub fn uniform(&mut self, low: f32, high: f32) -> f32 {
        assert!(low < high, "a uniform draw spans {low} to {high}");
        low + (high - low) * self.unit() as f32
    }

    pub fn below(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "a draw below {bound} picks nothing");
        ((u128::from(self.word()) * bound as u128) >> 64) as usize
    }

    pub fn bits(&mut self) -> u32 {
        (self.word() >> 32) as u32
    }

    fn word(&mut self) -> u64 {
        let previous = self.state;
        self.state = previous.wrapping_mul(MULTIPLIER).wrapping_add(self.stream);
        let xorshifted = ((previous >> 64) as u64) ^ previous as u64;
        xorshifted.rotate_right(((previous >> 122) as u32) & 63)
    }
}

struct Mixer(u64);

impl Mixer {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut word = self.0;
        word = (word ^ (word >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        word = (word ^ (word >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        word ^ (word >> 31)
    }
}
