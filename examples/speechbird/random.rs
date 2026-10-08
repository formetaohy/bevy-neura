pub struct Random(u32);

impl Default for Random {
    fn default() -> Self {
        Self::seeded(0x2f1b_3c5d)
    }
}

impl Random {
    pub const fn seeded(seed: u32) -> Self {
        Self(seed)
    }

    pub fn unit(&mut self) -> f32 {
        (self.word() >> 8) as f32 / (1u32 << 24) as f32
    }

    pub fn uniform(&mut self, low: f32, high: f32) -> f32 {
        assert!(low < high, "a draw from {low} to {high} spans nothing");
        low + (high - low) * self.unit()
    }

    pub fn below(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "a draw below {bound} picks nothing");
        ((u64::from(self.word()) * bound as u64) >> 32) as usize
    }

    pub fn word(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.0
    }
}
