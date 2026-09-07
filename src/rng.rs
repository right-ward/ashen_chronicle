use std::ops::Range;

const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(GAMMA),
        }
    }

    pub fn from_state(state: u64) -> Self {
        Self { state }
    }

    pub fn state(&self) -> u64 {
        self.state
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    pub fn gen_range(&mut self, range: Range<usize>) -> usize {
        debug_assert!(range.start < range.end);
        range.start + (self.next_u64() as usize % (range.end - range.start))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_seeds_produce_identical_sequences() {
        let mut a = DeterministicRng::new(123);
        let mut b = DeterministicRng::new(123);
        assert_eq!(a.next_u64(), b.next_u64());
        assert_eq!(a.next_bool(), b.next_bool());
        assert_eq!(a.gen_range(2..10), b.gen_range(2..10));
    }

    #[test]
    fn state_round_trip_continues_the_same_sequence() {
        let mut original = DeterministicRng::new(456);
        let _ = original.next_u64();
        let saved = original.state();
        let mut restored = DeterministicRng::from_state(saved);
        assert_eq!(original.next_u64(), restored.next_u64());
    }
}
