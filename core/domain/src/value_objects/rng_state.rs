/// Deterministic random state tracked per session.
#[derive(Debug, Clone, Copy)]
pub struct RngState {
    seed: i64,
    counter: u64,
}

impl Default for RngState {
    fn default() -> Self {
        Self {
            seed: 1488,
            counter: 0,
        }
    }
}

impl RngState {
    /// Creates a new RNG state from an explicit seed and counter.
    pub fn new(seed: i64, counter: u64) -> Self {
        Self { seed, counter }
    }

    /// Returns the stable session seed.
    pub fn seed(&self) -> i64 {
        self.seed
    }

    /// Returns how many random draws have been consumed so far.
    pub fn counter(&self) -> u64 {
        self.counter
    }

    /// Increments the draw counter after one deterministic random step.
    pub fn increase_counter(&mut self) {
        self.counter += 1;
    }
}
