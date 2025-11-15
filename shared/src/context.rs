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

    // ========================================================================
    // RED TESTS - Coverage improvement (shared/context.rs 72.22% → 100%)
    // ========================================================================

    #[test]
    fn test_next_random_generates_values() {
        // Lines 49-52: RNG generation with existing rng
        let mut ctx = ExecutionContext::new(42);
        let val1 = ctx.next_random();
        let val2 = ctx.next_random();

        // Values should be different (deterministic but advancing)
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_next_random_deterministic() {
        // Lines 49-52: Same seed produces same sequence
        let mut ctx1 = ExecutionContext::new(42);
        let mut ctx2 = ExecutionContext::new(42);

        assert_eq!(ctx1.next_random(), ctx2.next_random());
        assert_eq!(ctx1.next_random(), ctx2.next_random());
        assert_eq!(ctx1.next_random(), ctx2.next_random());
    }

    #[test]
    fn test_next_random_reinitializes_rng() {
        // Lines 55-56: RNG re-initialization path
        let mut ctx = ExecutionContext::new(42);

        // Clear the RNG to force re-initialization
        ctx.rng = None;

        // This should re-initialize and return a value
        let val = ctx.next_random();

        // Should be the first value from seed 42
        let mut expected_ctx = ExecutionContext::new(42);
        assert_eq!(val, expected_ctx.next_random());

        // RNG should now be Some again
        assert!(ctx.rng.is_some());
    }

    #[test]
    fn test_next_random_after_deserialization() {
        // Lines 55-56: Simulate deserialization scenario where rng is None
        let ctx = ExecutionContext {
            rng: None, // Simulates deserializ ation (rng is skipped)
            rng_seed: 123,
            simulated_time: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
        };

        let mut ctx_mut = ctx;
        let val = ctx_mut.next_random();

        // Should successfully generate a value
        assert!(val > 0 || val == 0); // Any u64 is valid

        // RNG should be initialized now
        assert!(ctx_mut.rng.is_some());
    }
}
