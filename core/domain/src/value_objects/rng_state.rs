#[derive(Debug, Clone, Copy)]
pub struct RngState {
    seed: u64,
    counter: u64,
}

impl Default for RngState {
    fn default() -> Self {
        Self {
            seed: 1337,
            counter: 0,
        }
    }
}

impl RngState {
    pub fn new(seed: u64, counter: u64) -> Self {
        Self { seed, counter }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn counter(&self) -> u64 {
        self.counter
    }
}
