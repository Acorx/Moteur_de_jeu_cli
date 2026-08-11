use serde::{Deserialize, Serialize};

/// SplitMix64, défini uniquement par des opérations entières modulo 2^64.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub const fn state(self) -> u64 {
        self.state
    }

    pub const fn from_state(state: u64) -> Self {
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_vector_seed_zero() {
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.next_u64(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(rng.next_u64(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(rng.next_u64(), 0x06c4_5d18_8009_454f);
        assert_eq!(rng.next_u64(), 0xf88b_b8a8_724c_81ec);
    }

    #[test]
    fn state_restoration_is_exact() {
        let mut first = SplitMix64::new(42);
        first.next_u64();
        let mut restored = SplitMix64::from_state(first.state());
        assert_eq!(first.next_u64(), restored.next_u64());
    }
}
