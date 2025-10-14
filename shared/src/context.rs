//! Execution Context
//!
//! Deterministic execution context with RNG, clock, and I/O tracking.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Execution context with deterministic RNG and simulated time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Deterministic RNG state
    #[serde(skip)]
    rng: Option<ChaCha8Rng>,
    /// RNG seed (for serialization)
    rng_seed: u64,
    /// Simulated time in microseconds
    pub simulated_time: u64,
    /// Standard output buffer
    pub stdout: Vec<u8>,
    /// Standard error buffer
    pub stderr: Vec<u8>,
}

impl ExecutionContext {
    /// Create new execution context with seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Some(ChaCha8Rng::seed_from_u64(seed)),
            rng_seed: seed,
            simulated_time: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    /// Advance simulated time by delta microseconds
    pub fn advance_time(&mut self, delta_us: u64) {
        self.simulated_time += delta_us;
    }

    /// Get RNG seed
    pub fn seed(&self) -> u64 {
        self.rng_seed
    }

    /// Get random u64 (for future use)
    #[allow(dead_code)]
    pub fn next_random(&mut self) -> u64 {
        use rand::RngCore;
        if let Some(ref mut rng) = self.rng {
            rng.next_u64()
        } else {
            // Re-initialize if needed
            self.rng = Some(ChaCha8Rng::seed_from_u64(self.rng_seed));
            self.rng.as_mut().unwrap().next_u64()
        }
    }
}

impl PartialEq for ExecutionContext {
    fn eq(&self, other: &Self) -> bool {
        self.rng_seed == other.rng_seed
            && self.simulated_time == other.simulated_time
            && self.stdout == other.stdout
            && self.stderr == other.stderr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ExecutionContext::new(42);
        assert_eq!(ctx.seed(), 42);
        assert_eq!(ctx.simulated_time, 0);
    }

    #[test]
    fn test_context_deterministic() {
        let ctx1 = ExecutionContext::new(42);
        let ctx2 = ExecutionContext::new(42);
        assert_eq!(ctx1, ctx2);
    }

    #[test]
    fn test_advance_time() {
        let mut ctx = ExecutionContext::new(42);
        ctx.advance_time(1000);
        assert_eq!(ctx.simulated_time, 1000);
        ctx.advance_time(500);
        assert_eq!(ctx.simulated_time, 1500);
    }
}
