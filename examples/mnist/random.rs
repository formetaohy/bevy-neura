pub struct Random {
    state: u64,
}

impl Random {
    pub fn seeded(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn below(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "a draw below {bound} picks nothing");
        ((u128::from(self.word()) * bound as u128) >> 64) as usize
    }

    fn word(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut word = self.state;
        word = (word ^ (word >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        word = (word ^ (word >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        word ^ (word >> 31)
    }
}
