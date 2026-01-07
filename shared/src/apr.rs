//! APR (Aprender Portable Runtime) Model Types
//!
//! This module defines the .apr file format for deterministic replay
//! and reproducibility verification in WOS.
//!
//! # Format Overview
//!
//! APR models capture:
//! - Initial state snapshot
//! - Timestamped inputs
//! - Expected outputs
//! - State checkpoints for verification
//!
//! # Determinism Guarantees
//!
//! Given the same:
//! - APR model
//! - RNG seed
//! - Initial state
//!
//! The execution will produce bit-identical results across platforms.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// APR format version
pub const APR_VERSION: &str = "1.0.0";

/// APR model - complete specification for deterministic replay
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AprModel {
    /// Format version (must match APR_VERSION)
    pub version: String,
    /// Model format identifier
    pub format: AprFormat,
    /// Deterministic RNG seed
    pub seed: u64,
    /// Initial state snapshot
    pub initial_state: serde_json::Value,
    /// Timestamped inputs
    pub inputs: Vec<TimestampedInput>,
    /// Expected outputs for verification
    pub expected_outputs: Vec<AprOutput>,
    /// State checkpoints
    pub checkpoints: Vec<StateCheckpoint>,
    /// Model metadata
    pub metadata: AprMetadata,
}

impl Default for AprModel {
    fn default() -> Self {
        Self {
            version: APR_VERSION.to_string(),
            format: AprFormat::WosKernelState,
            seed: 0,
            initial_state: serde_json::Value::Null,
            inputs: Vec::new(),
            expected_outputs: Vec::new(),
            checkpoints: Vec::new(),
            metadata: AprMetadata::default(),
        }
    }
}

impl AprModel {
    /// Create a new APR model with the given seed
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }

    /// Create with initial state
    pub fn with_initial_state(seed: u64, state: serde_json::Value) -> Self {
        Self {
            seed,
            initial_state: state,
            ..Default::default()
        }
    }

    /// Add an input to the model
    pub fn add_input(&mut self, tick: u64, input: AprInput) {
        self.inputs.push(TimestampedInput { tick, input });
    }

    /// Add an expected output
    pub fn add_expected_output(&mut self, output: AprOutput) {
        self.expected_outputs.push(output);
    }

    /// Add a checkpoint
    pub fn add_checkpoint(&mut self, checkpoint: StateCheckpoint) {
        self.checkpoints.push(checkpoint);
    }

    /// Validate the model structure
    pub fn validate(&self) -> Result<(), AprError> {
        // Check version
        if self.version != APR_VERSION {
            return Err(AprError::VersionMismatch {
                expected: APR_VERSION.to_string(),
                actual: self.version.clone(),
            });
        }

        // Verify inputs are in order
        let mut last_tick = 0;
        for input in &self.inputs {
            if input.tick < last_tick {
                return Err(AprError::InvalidInputOrder);
            }
            last_tick = input.tick;
        }

        // Verify checkpoints are in order
        last_tick = 0;
        for checkpoint in &self.checkpoints {
            if checkpoint.tick < last_tick {
                return Err(AprError::InvalidCheckpointOrder);
            }
            last_tick = checkpoint.tick;
        }

        Ok(())
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, AprError> {
        serde_json::to_string_pretty(self).map_err(|e| AprError::SerializationFailed(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, AprError> {
        let model: Self = serde_json::from_str(json)
            .map_err(|e| AprError::DeserializationFailed(e.to_string()))?;
        model.validate()?;
        Ok(model)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, AprError> {
        serde_json::to_vec(self).map_err(|e| AprError::SerializationFailed(e.to_string()))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AprError> {
        let model: Self = serde_json::from_slice(bytes)
            .map_err(|e| AprError::DeserializationFailed(e.to_string()))?;
        model.validate()?;
        Ok(model)
    }
}

/// APR format identifier
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AprFormat {
    /// WOS kernel state format
    #[default]
    WosKernelState,
    /// WOS process trace format
    WosProcessTrace,
    /// WOS syscall trace format
    WosSyscallTrace,
    /// Generic simulation format
    GenericSimulation,
    /// Custom format
    Custom(String),
}

/// Timestamped input for replay
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimestampedInput {
    /// Tick at which this input occurs
    pub tick: u64,
    /// The input data
    pub input: AprInput,
}

/// Input types for APR replay
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum AprInput {
    /// System call invocation
    Syscall(serde_json::Value),
    /// Keyboard input
    KeyPress(char),
    /// Command string
    Command(String),
    /// Timer tick
    Timer(u64),
    /// Signal delivery
    Signal(i32),
    /// External event
    ExternalEvent(String),
    /// Custom input type
    Custom {
        /// Input type name
        name: String,
        /// Input data
        data: serde_json::Value,
    },
}

/// Expected output for verification
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AprOutput {
    /// Tick at which output is expected
    pub tick: u64,
    /// Output type identifier
    pub output_type: String,
    /// Output data
    pub data: serde_json::Value,
}

impl AprOutput {
    /// Create a new output
    pub fn new(tick: u64, output_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            tick,
            output_type: output_type.into(),
            data,
        }
    }
}

/// State checkpoint for verification
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StateCheckpoint {
    /// Tick at which checkpoint was taken
    pub tick: u64,
    /// Blake3 hash of state
    pub state_hash: [u8; 32],
    /// Human-readable description
    pub description: String,
}

impl StateCheckpoint {
    /// Create a new checkpoint
    pub fn new(tick: u64, state_hash: [u8; 32], description: impl Into<String>) -> Self {
        Self {
            tick,
            state_hash,
            description: description.into(),
        }
    }

    /// Create from state data (computes hash)
    pub fn from_state(tick: u64, state: &[u8], description: impl Into<String>) -> Self {
        // Simple hash for now - can upgrade to blake3 later
        let mut hash = [0u8; 32];
        let state_hash = simple_hash(state);
        hash.copy_from_slice(&state_hash);
        Self {
            tick,
            state_hash: hash,
            description: description.into(),
        }
    }
}

/// Simple hash function (placeholder for blake3)
fn simple_hash(data: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut state: u64 = 0x_dead_beef_cafe_babe;
    for (i, &byte) in data.iter().enumerate() {
        state = state.wrapping_mul(0x_5851_f42d_4c95_7f2d);
        state = state.wrapping_add(byte as u64);
        result[i % 32] ^= (state >> ((i % 8) * 8)) as u8;
    }
    result
}

/// APR model metadata
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AprMetadata {
    /// Model name
    pub name: Option<String>,
    /// Model description
    pub description: Option<String>,
    /// Creation timestamp (ISO 8601)
    pub created_at: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom properties
    pub properties: HashMap<String, String>,
}

impl AprMetadata {
    /// Create with name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }
}

/// APR errors
#[derive(Clone, Debug, PartialEq)]
pub enum AprError {
    /// Version mismatch between expected and actual APR format
    VersionMismatch {
        /// Expected version string
        expected: String,
        /// Actual version string found
        actual: String,
    },
    /// Inputs not in chronological order
    InvalidInputOrder,
    /// Checkpoints not in chronological order
    InvalidCheckpointOrder,
    /// Serialization failed
    SerializationFailed(String),
    /// Deserialization failed
    DeserializationFailed(String),
    /// Output mismatch during replay verification
    OutputMismatch {
        /// Tick at which mismatch occurred
        tick: u64,
        /// Expected output from APR model
        expected: Box<AprOutput>,
        /// Actual output produced during replay
        actual: Box<AprOutput>,
    },
    /// Checkpoint verification failed during replay
    CheckpointMismatch {
        /// Tick at which mismatch occurred
        tick: u64,
        /// Expected state hash from APR model
        expected: [u8; 32],
        /// Actual state hash computed during replay
        actual: [u8; 32],
    },
    /// Model is empty (no inputs)
    EmptyModel,
    /// Replay finished
    ReplayFinished,
}

impl std::fmt::Display for AprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VersionMismatch { expected, actual } => {
                write!(
                    f,
                    "APR version mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            Self::InvalidInputOrder => write!(f, "Inputs are not in chronological order"),
            Self::InvalidCheckpointOrder => {
                write!(f, "Checkpoints are not in chronological order")
            }
            Self::SerializationFailed(e) => write!(f, "Serialization failed: {}", e),
            Self::DeserializationFailed(e) => write!(f, "Deserialization failed: {}", e),
            Self::OutputMismatch { tick, .. } => {
                write!(f, "Output mismatch at tick {}", tick)
            }
            Self::CheckpointMismatch { tick, .. } => {
                write!(f, "Checkpoint mismatch at tick {}", tick)
            }
            Self::EmptyModel => write!(f, "APR model is empty"),
            Self::ReplayFinished => write!(f, "Replay has finished"),
        }
    }
}

impl std::error::Error for AprError {}

/// APR runtime for executing models
pub struct AprRuntime {
    /// The model being executed
    model: AprModel,
    /// Current input index
    input_index: usize,
    /// Current checkpoint index
    checkpoint_index: usize,
    /// Current tick
    current_tick: u64,
    /// Outputs collected during execution
    outputs: Vec<AprOutput>,
}

impl AprRuntime {
    /// Create a new runtime from a model
    pub fn new(model: AprModel) -> Result<Self, AprError> {
        model.validate()?;
        Ok(Self {
            model,
            input_index: 0,
            checkpoint_index: 0,
            current_tick: 0,
            outputs: Vec::new(),
        })
    }

    /// Get the model seed
    pub fn seed(&self) -> u64 {
        self.model.seed
    }

    /// Get the initial state
    pub fn initial_state(&self) -> &serde_json::Value {
        &self.model.initial_state
    }

    /// Check if replay is finished
    pub fn is_finished(&self) -> bool {
        self.input_index >= self.model.inputs.len()
    }

    /// Get next input
    pub fn next_input(&mut self) -> Option<&TimestampedInput> {
        if self.input_index < self.model.inputs.len() {
            let input = &self.model.inputs[self.input_index];
            self.input_index += 1;
            self.current_tick = input.tick;
            Some(input)
        } else {
            None
        }
    }

    /// Record an output
    pub fn record_output(&mut self, output: AprOutput) {
        self.outputs.push(output);
    }

    /// Verify output against expected
    pub fn verify_output(&self, tick: u64, actual: &AprOutput) -> Result<(), AprError> {
        if let Some(expected) = self
            .model
            .expected_outputs
            .iter()
            .find(|o| o.tick == tick && o.output_type == actual.output_type)
        {
            if expected != actual {
                return Err(AprError::OutputMismatch {
                    tick,
                    expected: Box::new(expected.clone()),
                    actual: Box::new(actual.clone()),
                });
            }
        }
        Ok(())
    }

    /// Verify checkpoint
    pub fn verify_checkpoint(&mut self, state_hash: [u8; 32]) -> Result<(), AprError> {
        if self.checkpoint_index < self.model.checkpoints.len() {
            let checkpoint = &self.model.checkpoints[self.checkpoint_index];
            if checkpoint.tick <= self.current_tick {
                if checkpoint.state_hash != state_hash {
                    return Err(AprError::CheckpointMismatch {
                        tick: checkpoint.tick,
                        expected: checkpoint.state_hash,
                        actual: state_hash,
                    });
                }
                self.checkpoint_index += 1;
            }
        }
        Ok(())
    }

    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Get collected outputs
    pub fn outputs(&self) -> &[AprOutput] {
        &self.outputs
    }

    /// Get the model
    pub fn model(&self) -> &AprModel {
        &self.model
    }

    /// Export the current state as a new APR model
    ///
    /// This creates a new model that can be saved and replayed later.
    /// The initial_state is serialized to JSON for portability.
    pub fn export_model<T: serde::Serialize>(&self, initial_state: &T) -> AprModel {
        AprModel {
            version: APR_VERSION.to_string(),
            format: self.model.format.clone(),
            seed: self.model.seed,
            initial_state: serde_json::to_value(initial_state).unwrap_or_default(),
            inputs: self.model.inputs.clone(),
            expected_outputs: self.outputs.clone(),
            checkpoints: self.model.checkpoints.clone(),
            metadata: self.model.metadata.clone(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apr_model_creation() {
        let model = AprModel::new(12345);
        assert_eq!(model.seed, 12345);
        assert_eq!(model.version, APR_VERSION);
        assert!(model.inputs.is_empty());
    }

    #[test]
    fn test_apr_model_validation() {
        let model = AprModel::new(0);
        assert!(model.validate().is_ok());
    }

    #[test]
    fn test_apr_model_version_mismatch() {
        let mut model = AprModel::new(0);
        model.version = "0.0.0".to_string();
        let result = model.validate();
        assert!(matches!(result, Err(AprError::VersionMismatch { .. })));
    }

    #[test]
    fn test_apr_model_input_order() {
        let mut model = AprModel::new(0);
        model.add_input(10, AprInput::Timer(10));
        model.add_input(5, AprInput::Timer(5)); // Out of order
        let result = model.validate();
        assert!(matches!(result, Err(AprError::InvalidInputOrder)));
    }

    #[test]
    fn test_apr_model_serialization() {
        let mut model = AprModel::new(42);
        model.add_input(0, AprInput::Command("ls".to_string()));
        model.add_input(1, AprInput::KeyPress('a'));

        let json = model.to_json().unwrap();
        let restored = AprModel::from_json(&json).unwrap();

        assert_eq!(model, restored);
    }

    #[test]
    fn test_apr_runtime_creation() {
        let model = AprModel::new(100);
        let runtime = AprRuntime::new(model).unwrap();
        assert_eq!(runtime.seed(), 100);
        // Empty model is finished (no inputs to replay)
        assert!(runtime.is_finished());
    }

    #[test]
    fn test_apr_runtime_with_inputs() {
        let mut model = AprModel::new(100);
        model.add_input(0, AprInput::Command("test".to_string()));
        let runtime = AprRuntime::new(model).unwrap();
        assert!(!runtime.is_finished());
    }

    #[test]
    fn test_apr_runtime_empty_model() {
        let model = AprModel::new(0);
        let runtime = AprRuntime::new(model).unwrap();
        assert!(runtime.is_finished());
    }

    #[test]
    fn test_apr_runtime_replay() {
        let mut model = AprModel::new(42);
        model.add_input(0, AprInput::Command("echo hello".to_string()));
        model.add_input(1, AprInput::Command("ls".to_string()));
        model.add_input(2, AprInput::Command("exit".to_string()));

        let mut runtime = AprRuntime::new(model).unwrap();

        let input1 = runtime.next_input().unwrap();
        assert_eq!(input1.tick, 0);
        assert!(matches!(&input1.input, AprInput::Command(c) if c == "echo hello"));

        let input2 = runtime.next_input().unwrap();
        assert_eq!(input2.tick, 1);

        let input3 = runtime.next_input().unwrap();
        assert_eq!(input3.tick, 2);

        assert!(runtime.next_input().is_none());
        assert!(runtime.is_finished());
    }

    #[test]
    fn test_state_checkpoint() {
        let checkpoint = StateCheckpoint::from_state(100, b"test state", "Test checkpoint");
        assert_eq!(checkpoint.tick, 100);
        assert_eq!(checkpoint.description, "Test checkpoint");
        assert_ne!(checkpoint.state_hash, [0u8; 32]);
    }

    #[test]
    fn test_checkpoint_verification() {
        let mut model = AprModel::new(0);
        let checkpoint = StateCheckpoint::from_state(0, b"initial", "Initial state");
        let expected_hash = checkpoint.state_hash;
        model.add_checkpoint(checkpoint);

        let mut runtime = AprRuntime::new(model).unwrap();
        assert!(runtime.verify_checkpoint(expected_hash).is_ok());
    }

    #[test]
    fn test_checkpoint_mismatch() {
        let mut model = AprModel::new(0);
        let checkpoint = StateCheckpoint::from_state(0, b"initial", "Initial state");
        model.add_checkpoint(checkpoint);

        let mut runtime = AprRuntime::new(model).unwrap();
        let wrong_hash = [1u8; 32];
        let result = runtime.verify_checkpoint(wrong_hash);
        assert!(matches!(result, Err(AprError::CheckpointMismatch { .. })));
    }

    #[test]
    fn test_apr_metadata() {
        let metadata = AprMetadata::with_name("Test Model");
        assert_eq!(metadata.name, Some("Test Model".to_string()));
    }

    #[test]
    fn test_apr_error_display() {
        let err = AprError::VersionMismatch {
            expected: "1.0.0".to_string(),
            actual: "0.9.0".to_string(),
        };
        assert!(err.to_string().contains("1.0.0"));
        assert!(err.to_string().contains("0.9.0"));
    }

    // ========================================================================
    // Property-based tests
    // ========================================================================
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        // Strategy for generating valid AprInput
        fn arb_apr_input() -> impl Strategy<Value = AprInput> {
            prop_oneof![
                any::<char>().prop_map(AprInput::KeyPress),
                "[a-z]{1,20}".prop_map(AprInput::Command),
                any::<u64>().prop_map(AprInput::Timer),
                (1i32..32).prop_map(AprInput::Signal),
                "[a-z]{1,10}".prop_map(AprInput::ExternalEvent),
                ("[a-z]{1,10}", "[a-z]{1,10}").prop_map(|(name, data)| AprInput::Custom {
                    name,
                    data: serde_json::Value::String(data),
                }),
            ]
        }

        // Strategy for generating valid TimestampedInput with monotonic ticks
        fn arb_timestamped_inputs(count: usize) -> impl Strategy<Value = Vec<TimestampedInput>> {
            proptest::collection::vec(arb_apr_input(), count).prop_map(|inputs| {
                inputs
                    .into_iter()
                    .enumerate()
                    .map(|(i, input)| TimestampedInput {
                        tick: i as u64,
                        input,
                    })
                    .collect()
            })
        }

        // Strategy for generating valid StateCheckpoint with monotonic ticks
        fn arb_checkpoints(count: usize) -> impl Strategy<Value = Vec<StateCheckpoint>> {
            proptest::collection::vec(any::<[u8; 32]>(), count).prop_map(|hashes| {
                hashes
                    .into_iter()
                    .enumerate()
                    .map(|(i, hash)| StateCheckpoint {
                        tick: i as u64,
                        state_hash: hash,
                        description: format!("checkpoint_{}", i),
                    })
                    .collect()
            })
        }

        // Strategy for generating valid AprModel
        fn arb_apr_model() -> impl Strategy<Value = AprModel> {
            (any::<u64>(), 0usize..5, 0usize..3).prop_flat_map(
                |(seed, input_count, checkpoint_count)| {
                    (
                        Just(seed),
                        arb_timestamped_inputs(input_count),
                        arb_checkpoints(checkpoint_count),
                    )
                        .prop_map(|(seed, inputs, checkpoints)| AprModel {
                            version: APR_VERSION.to_string(),
                            format: AprFormat::WosKernelState,
                            seed,
                            initial_state: serde_json::Value::Null,
                            inputs,
                            expected_outputs: vec![],
                            checkpoints,
                            metadata: AprMetadata::default(),
                        })
                },
            )
        }

        proptest! {
            // AprModel properties
            #[test]
            fn prop_model_serialization_roundtrip(model in arb_apr_model()) {
                let json = model.to_json().unwrap();
                let restored = AprModel::from_json(&json).unwrap();
                prop_assert_eq!(model, restored);
            }

            #[test]
            fn prop_model_bytes_roundtrip(model in arb_apr_model()) {
                let bytes = model.to_bytes().unwrap();
                let restored = AprModel::from_bytes(&bytes).unwrap();
                prop_assert_eq!(model, restored);
            }

            #[test]
            fn prop_model_validation_passes_for_valid(model in arb_apr_model()) {
                prop_assert!(model.validate().is_ok());
            }

            #[test]
            fn prop_model_ordered_inputs_valid(inputs in arb_timestamped_inputs(5)) {
                let mut model = AprModel::new(0);
                model.inputs = inputs;
                prop_assert!(model.validate().is_ok());
            }

            #[test]
            fn prop_model_out_of_order_inputs_invalid(
                tick1 in 10u64..100,
                tick2 in 0u64..10
            ) {
                let mut model = AprModel::new(0);
                model.inputs = vec![
                    TimestampedInput { tick: tick1, input: AprInput::Timer(0) },
                    TimestampedInput { tick: tick2, input: AprInput::Timer(1) },
                ];
                prop_assert!(matches!(model.validate(), Err(AprError::InvalidInputOrder)));
            }

            #[test]
            fn prop_model_out_of_order_checkpoints_invalid(
                tick1 in 10u64..100,
                tick2 in 0u64..10
            ) {
                let mut model = AprModel::new(0);
                model.checkpoints = vec![
                    StateCheckpoint { tick: tick1, state_hash: [0u8; 32], description: "a".into() },
                    StateCheckpoint { tick: tick2, state_hash: [1u8; 32], description: "b".into() },
                ];
                prop_assert!(matches!(model.validate(), Err(AprError::InvalidCheckpointOrder)));
            }

            #[test]
            fn prop_model_version_mismatch_detected(
                version in "[0-9]\\.[0-9]\\.[0-9]".prop_filter("not current version", |v| v != APR_VERSION)
            ) {
                let mut model = AprModel::new(0);
                model.version = version;
                let result = model.validate();
                let is_version_mismatch = matches!(result, Err(AprError::VersionMismatch { .. }));
                prop_assert!(is_version_mismatch);
            }

            // AprRuntime properties
            #[test]
            fn prop_runtime_input_count_matches(model in arb_apr_model()) {
                let expected_count = model.inputs.len();
                let mut runtime = AprRuntime::new(model).unwrap();
                let mut actual_count = 0;
                while runtime.next_input().is_some() {
                    actual_count += 1;
                }
                prop_assert_eq!(expected_count, actual_count);
            }

            #[test]
            fn prop_runtime_finished_after_all_inputs(model in arb_apr_model()) {
                let mut runtime = AprRuntime::new(model).unwrap();
                while runtime.next_input().is_some() {}
                prop_assert!(runtime.is_finished());
            }

            #[test]
            fn prop_runtime_tick_monotonic(model in arb_apr_model()) {
                let mut runtime = AprRuntime::new(model).unwrap();
                let mut last_tick = 0u64;
                while let Some(input) = runtime.next_input() {
                    prop_assert!(input.tick >= last_tick, "Tick must be monotonically non-decreasing");
                    last_tick = input.tick;
                }
            }

            #[test]
            fn prop_runtime_seed_preserved(seed: u64, inputs in arb_timestamped_inputs(3)) {
                let mut model = AprModel::new(seed);
                model.inputs = inputs;
                let runtime = AprRuntime::new(model).unwrap();
                prop_assert_eq!(runtime.seed(), seed);
            }

            #[test]
            fn prop_runtime_deterministic_replay(model in arb_apr_model()) {
                // Two runtimes with same model should produce same sequence
                let model2 = model.clone();
                let mut runtime1 = AprRuntime::new(model).unwrap();
                let mut runtime2 = AprRuntime::new(model2).unwrap();

                loop {
                    let input1 = runtime1.next_input();
                    let input2 = runtime2.next_input();
                    match (input1, input2) {
                        (Some(i1), Some(i2)) => {
                            prop_assert_eq!(i1.tick, i2.tick);
                            prop_assert_eq!(&i1.input, &i2.input);
                        }
                        (None, None) => break,
                        _ => prop_assert!(false, "Mismatched input availability"),
                    }
                }
            }

            // StateCheckpoint properties
            #[test]
            fn prop_checkpoint_hash_deterministic(state: Vec<u8>, tick: u64) {
                let cp1 = StateCheckpoint::from_state(tick, &state, "test");
                let cp2 = StateCheckpoint::from_state(tick, &state, "test");
                prop_assert_eq!(cp1.state_hash, cp2.state_hash);
            }

            #[test]
            fn prop_checkpoint_different_states_different_hashes(
                state1: Vec<u8>,
                state2: Vec<u8>
            ) {
                prop_assume!(!state1.is_empty() && !state2.is_empty());
                prop_assume!(state1 != state2);
                let cp1 = StateCheckpoint::from_state(0, &state1, "a");
                let cp2 = StateCheckpoint::from_state(0, &state2, "b");
                // Note: Hash collisions are possible but extremely rare
                // We expect them to be different for random data
                prop_assert_ne!(cp1.state_hash, cp2.state_hash);
            }

            #[test]
            fn prop_checkpoint_serialization_roundtrip(
                tick: u64,
                hash: [u8; 32],
                desc in "[a-z]{1,20}"
            ) {
                let checkpoint = StateCheckpoint::new(tick, hash, desc);
                let json = serde_json::to_string(&checkpoint).unwrap();
                let restored: StateCheckpoint = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(checkpoint, restored);
            }

            // AprInput properties
            #[test]
            fn prop_input_serialization_roundtrip(input in arb_apr_input()) {
                let json = serde_json::to_string(&input).unwrap();
                let restored: AprInput = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(input, restored);
            }

            #[test]
            fn prop_input_clone_equality(input in arb_apr_input()) {
                let cloned = input.clone();
                prop_assert_eq!(input, cloned);
            }

            // AprOutput properties
            #[test]
            fn prop_output_serialization_roundtrip(
                tick: u64,
                output_type in "[a-z]{1,10}",
                data in "[a-z]{1,10}"
            ) {
                let output = AprOutput::new(tick, output_type, serde_json::Value::String(data));
                let json = serde_json::to_string(&output).unwrap();
                let restored: AprOutput = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(output, restored);
            }

            #[test]
            fn prop_output_clone_equality(
                tick: u64,
                output_type in "[a-z]{1,10}"
            ) {
                let output = AprOutput::new(tick, output_type, serde_json::Value::Null);
                let cloned = output.clone();
                prop_assert_eq!(output, cloned);
            }

            // AprMetadata properties
            #[test]
            fn prop_metadata_serialization_roundtrip(
                name in proptest::option::of("[a-z]{1,10}"),
                desc in proptest::option::of("[a-z]{1,20}"),
                tags in proptest::collection::vec("[a-z]{1,5}", 0..3)
            ) {
                let metadata = AprMetadata {
                    name,
                    description: desc,
                    created_at: None,
                    author: None,
                    tags,
                    properties: HashMap::new(),
                };
                let json = serde_json::to_string(&metadata).unwrap();
                let restored: AprMetadata = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(metadata, restored);
            }

            // AprFormat properties
            #[test]
            fn prop_format_serialization_roundtrip(idx in 0u8..5) {
                let format = match idx {
                    0 => AprFormat::WosKernelState,
                    1 => AprFormat::WosProcessTrace,
                    2 => AprFormat::WosSyscallTrace,
                    3 => AprFormat::GenericSimulation,
                    _ => AprFormat::Custom("test".to_string()),
                };
                let json = serde_json::to_string(&format).unwrap();
                let restored: AprFormat = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(format, restored);
            }

            // AprError properties
            #[test]
            fn prop_error_display_not_empty(
                expected in "[0-9]\\.[0-9]\\.[0-9]",
                actual in "[0-9]\\.[0-9]\\.[0-9]"
            ) {
                let err = AprError::VersionMismatch { expected, actual };
                prop_assert!(!err.to_string().is_empty());
            }

            // Verification properties
            #[test]
            fn prop_output_verification_accepts_matching(
                tick: u64,
                output_type in "[a-z]{1,10}"
            ) {
                let output = AprOutput::new(tick, output_type.clone(), serde_json::Value::Null);
                let mut model = AprModel::new(0);
                model.expected_outputs.push(output.clone());
                let runtime = AprRuntime::new(model).unwrap();
                prop_assert!(runtime.verify_output(tick, &output).is_ok());
            }

            #[test]
            fn prop_output_verification_rejects_mismatched(
                tick: u64,
                output_type in "[a-z]{1,10}"
            ) {
                let expected = AprOutput::new(tick, output_type.clone(), serde_json::Value::String("expected".into()));
                let actual = AprOutput::new(tick, output_type, serde_json::Value::String("actual".into()));
                let mut model = AprModel::new(0);
                model.expected_outputs.push(expected);
                let runtime = AprRuntime::new(model).unwrap();
                let result = runtime.verify_output(tick, &actual);
                let is_mismatch = matches!(result, Err(AprError::OutputMismatch { .. }));
                prop_assert!(is_mismatch);
            }
        }

        // Non-parameterized tests
        #[test]
        fn prop_runtime_empty_model_is_finished() {
            let model = AprModel::new(0);
            let runtime = AprRuntime::new(model).unwrap();
            assert!(runtime.is_finished());
        }

        #[test]
        fn prop_checkpoint_empty_state_hashes() {
            let cp1 = StateCheckpoint::from_state(0, &[], "empty1");
            let cp2 = StateCheckpoint::from_state(0, &[], "empty2");
            assert_eq!(cp1.state_hash, cp2.state_hash);
        }

        #[test]
        fn prop_model_default_is_valid() {
            let model = AprModel::default();
            assert!(model.validate().is_ok());
        }
    }

    // Additional coverage tests
    mod coverage_tests {
        use super::*;

        #[test]
        fn test_apr_output_new() {
            let output = AprOutput::new(42, "test_type", serde_json::json!({"key": "value"}));
            assert_eq!(output.tick, 42);
            assert_eq!(output.output_type, "test_type");
        }

        #[test]
        fn test_state_checkpoint_new() {
            let hash = [0xABu8; 32];
            let checkpoint = StateCheckpoint::new(100, hash, "Test description");
            assert_eq!(checkpoint.tick, 100);
            assert_eq!(checkpoint.state_hash, hash);
            assert_eq!(checkpoint.description, "Test description");
        }

        #[test]
        fn test_apr_runtime_initial_state() {
            let state = serde_json::json!({"initial": true});
            let model = AprModel::with_initial_state(42, state.clone());
            let runtime = AprRuntime::new(model).unwrap();
            assert_eq!(runtime.initial_state(), &state);
        }

        #[test]
        fn test_apr_runtime_outputs() {
            let mut model = AprModel::new(0);
            model.add_input(0, AprInput::Command("test".to_string()));
            let mut runtime = AprRuntime::new(model).unwrap();

            let output = AprOutput::new(0, "test", serde_json::Value::Null);
            runtime.record_output(output.clone());

            assert_eq!(runtime.outputs().len(), 1);
            assert_eq!(runtime.outputs()[0], output);
        }

        #[test]
        fn test_apr_runtime_model() {
            let model = AprModel::new(12345);
            let runtime = AprRuntime::new(model.clone()).unwrap();
            assert_eq!(runtime.model().seed, model.seed);
        }

        #[test]
        fn test_apr_runtime_export_model() {
            let mut model = AprModel::new(42);
            model.add_input(0, AprInput::Timer(100));
            let mut runtime = AprRuntime::new(model).unwrap();

            // Record an output
            let output = AprOutput::new(0, "timer", serde_json::json!(100));
            runtime.record_output(output);

            // Export model
            let initial_state = serde_json::json!({"exported": true});
            let exported = runtime.export_model(&initial_state);

            assert_eq!(exported.seed, 42);
            assert_eq!(exported.expected_outputs.len(), 1);
        }

        #[test]
        fn test_apr_error_display_all_variants() {
            let version_err = AprError::VersionMismatch {
                expected: "1.0.0".to_string(),
                actual: "2.0.0".to_string(),
            };
            assert!(version_err.to_string().contains("version mismatch"));

            let input_err = AprError::InvalidInputOrder;
            assert!(input_err.to_string().contains("chronological"));

            let checkpoint_err = AprError::InvalidCheckpointOrder;
            assert!(checkpoint_err.to_string().contains("chronological"));

            let ser_err = AprError::SerializationFailed("test error".to_string());
            assert!(ser_err.to_string().contains("Serialization"));

            let deser_err = AprError::DeserializationFailed("parse error".to_string());
            assert!(deser_err.to_string().contains("Deserialization"));

            let output_err = AprError::OutputMismatch {
                tick: 5,
                expected: Box::new(AprOutput::new(5, "a", serde_json::Value::Null)),
                actual: Box::new(AprOutput::new(5, "b", serde_json::Value::Null)),
            };
            assert!(output_err.to_string().contains("tick 5"));

            let checkpoint_mismatch = AprError::CheckpointMismatch {
                tick: 10,
                expected: [1u8; 32],
                actual: [2u8; 32],
            };
            assert!(checkpoint_mismatch.to_string().contains("tick 10"));

            let empty_err = AprError::EmptyModel;
            assert!(empty_err.to_string().contains("empty"));

            let finished_err = AprError::ReplayFinished;
            assert!(finished_err.to_string().contains("finished"));
        }

        #[test]
        fn test_apr_format_all_variants() {
            let formats = vec![
                AprFormat::WosKernelState,
                AprFormat::WosProcessTrace,
                AprFormat::WosSyscallTrace,
                AprFormat::GenericSimulation,
                AprFormat::Custom("test".to_string()),
            ];

            for format in formats {
                let json = serde_json::to_string(&format).unwrap();
                let restored: AprFormat = serde_json::from_str(&json).unwrap();
                assert_eq!(format, restored);
            }
        }

        #[test]
        fn test_apr_model_add_expected_output() {
            let mut model = AprModel::new(0);
            let output = AprOutput::new(0, "test", serde_json::Value::Null);
            model.add_expected_output(output.clone());
            assert_eq!(model.expected_outputs.len(), 1);
            assert_eq!(model.expected_outputs[0], output);
        }

        #[test]
        fn test_apr_model_add_checkpoint() {
            let mut model = AprModel::new(0);
            let checkpoint = StateCheckpoint::new(0, [0u8; 32], "test");
            model.add_checkpoint(checkpoint.clone());
            assert_eq!(model.checkpoints.len(), 1);
            assert_eq!(model.checkpoints[0], checkpoint);
        }

        #[test]
        fn test_apr_model_from_bytes() {
            let model = AprModel::new(42);
            let bytes = model.to_bytes().unwrap();
            let restored = AprModel::from_bytes(&bytes).unwrap();
            assert_eq!(model, restored);
        }

        #[test]
        fn test_apr_model_from_bytes_invalid() {
            let result = AprModel::from_bytes(b"invalid json");
            assert!(result.is_err());
        }

        #[test]
        fn test_apr_model_from_json_invalid() {
            let result = AprModel::from_json("not valid json");
            assert!(result.is_err());
        }

        #[test]
        fn test_apr_runtime_verify_output_no_expected() {
            let model = AprModel::new(0);
            let runtime = AprRuntime::new(model).unwrap();

            let output = AprOutput::new(0, "test", serde_json::Value::Null);
            // No expected outputs - should pass
            let result = runtime.verify_output(0, &output);
            assert!(result.is_ok());
        }

        #[test]
        fn test_apr_runtime_verify_checkpoint_no_checkpoints() {
            let model = AprModel::new(0);
            let mut runtime = AprRuntime::new(model).unwrap();

            // No checkpoints - should pass
            let result = runtime.verify_checkpoint([0u8; 32]);
            assert!(result.is_ok());
        }

        #[test]
        fn test_apr_metadata_default() {
            let metadata = AprMetadata::default();
            assert!(metadata.name.is_none());
            assert!(metadata.description.is_none());
            assert!(metadata.created_at.is_none());
            assert!(metadata.author.is_none());
            assert!(metadata.tags.is_empty());
            assert!(metadata.properties.is_empty());
        }

        #[test]
        fn test_timestamped_input_serialization() {
            let input = TimestampedInput {
                tick: 42,
                input: AprInput::Timer(100),
            };
            let json = serde_json::to_string(&input).unwrap();
            let restored: TimestampedInput = serde_json::from_str(&json).unwrap();
            assert_eq!(input, restored);
        }

        #[test]
        fn test_apr_input_all_variants_serialization() {
            let inputs = vec![
                AprInput::Syscall(serde_json::json!({"call": "read"})),
                AprInput::KeyPress('x'),
                AprInput::Command("test".to_string()),
                AprInput::Timer(1000),
                AprInput::Signal(15),
                AprInput::ExternalEvent("network".to_string()),
                AprInput::Custom {
                    name: "custom".to_string(),
                    data: serde_json::json!({"key": "value"}),
                },
            ];

            for input in inputs {
                let json = serde_json::to_string(&input).unwrap();
                let restored: AprInput = serde_json::from_str(&json).unwrap();
                assert_eq!(input, restored);
            }
        }

        #[test]
        fn test_apr_checkpoint_order_validation() {
            let mut model = AprModel::new(0);
            model.add_checkpoint(StateCheckpoint::new(10, [0u8; 32], "first"));
            model.add_checkpoint(StateCheckpoint::new(5, [1u8; 32], "second")); // Out of order
            let result = model.validate();
            assert!(matches!(result, Err(AprError::InvalidCheckpointOrder)));
        }

        #[test]
        fn test_apr_runtime_verify_checkpoint_skip_future() {
            let mut model = AprModel::new(0);
            model.add_input(0, AprInput::Timer(100));
            // Checkpoint at tick 10 (future)
            model.add_checkpoint(StateCheckpoint::new(10, [0xABu8; 32], "future"));

            let mut runtime = AprRuntime::new(model).unwrap();
            runtime.next_input(); // current_tick = 0

            // Verify with any hash - should pass since checkpoint is in the future
            let result = runtime.verify_checkpoint([0u8; 32]);
            assert!(result.is_ok());
        }

        #[test]
        fn test_apr_model_clone_equality() {
            let mut model = AprModel::new(42);
            model.add_input(0, AprInput::Command("test".to_string()));

            let cloned = model.clone();
            assert_eq!(model, cloned);
        }

        #[test]
        fn test_simple_hash_deterministic() {
            // Test the simple_hash function indirectly via StateCheckpoint
            let cp1 = StateCheckpoint::from_state(0, b"test data", "a");
            let cp2 = StateCheckpoint::from_state(0, b"test data", "b");
            assert_eq!(cp1.state_hash, cp2.state_hash);
        }
    }
}
