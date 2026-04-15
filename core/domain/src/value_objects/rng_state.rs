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
    pub fn new(seed: i64, counter: u64) -> Self {
        Self { seed, counter }
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }

    pub fn counter(&self) -> u64 {
        self.counter
    }

    pub fn increase_counter(&mut self) {
        self.counter += 1;
    }
}
