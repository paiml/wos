//! APR (Aprender Portable Runtime) Integration for WOS Kernel
//!
//! This module provides kernel-level APR model execution for deterministic
//! replay and state verification. Following the PAIML stack's apr format.
//!
//! # EXTREME TDD
//!
//! All functionality must be test-driven:
//! 1. Write failing test
//! 2. Implement minimum to pass
//! 3. Refactor
//!
//! # Determinism Guarantee
//!
//! APR execution is fully deterministic - given the same seed and inputs,
//! the execution will produce identical outputs across platforms.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use wos_shared::{
    AprError, AprInput, AprModel, AprOutput, AprRuntime as SharedAprRuntime, StateCheckpoint,
};

use crate::KernelState;

/// Kernel APR runtime wrapper
///
/// Provides integration between the kernel and APR model execution.
pub struct KernelAprRuntime {
    /// The underlying APR runtime
    runtime: SharedAprRuntime,
    /// Whether the runtime is currently active
    active: bool,
    /// Number of steps executed
    steps_executed: u64,
}

impl KernelAprRuntime {
    /// Create a new kernel APR runtime from a model
    pub fn new(model: AprModel) -> Result<Self, AprError> {
        Ok(Self {
            runtime: SharedAprRuntime::new(model)?,
            active: false,
            steps_executed: 0,
        })
    }

    /// Load an APR model from JSON
    pub fn from_json(json: &str) -> Result<Self, AprError> {
        let model = AprModel::from_json(json)?;
        Self::new(model)
    }

    /// Start the APR runtime
    pub fn start(&mut self) -> Result<(), AprError> {
        if self.active {
            return Err(AprError::SerializationFailed(
                "Runtime already active".to_string(),
            ));
        }
        self.active = true;
        Ok(())
    }

    /// Stop the APR runtime
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Check if runtime is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if runtime has finished
    pub fn is_finished(&self) -> bool {
        self.runtime.is_finished()
    }

    /// Get the next input to apply
    pub fn next_input(&mut self) -> Result<Option<AprInput>, AprError> {
        if !self.active {
            return Err(AprError::SerializationFailed(
                "Runtime not active".to_string(),
            ));
        }
        Ok(self.runtime.next_input().map(|ti| ti.input.clone()))
    }

    /// Record an output
    pub fn record_output(&mut self, output: AprOutput) {
        self.runtime.record_output(output);
    }

    /// Verify an output against expected
    pub fn verify_output(&self, tick: u64, output: &AprOutput) -> Result<(), AprError> {
        self.runtime.verify_output(tick, output)
    }

    /// Get the current tick
    pub fn current_tick(&self) -> u64 {
        self.runtime.current_tick()
    }

    /// Execute one step and return the output
    pub fn step(&mut self) -> Result<Option<AprOutput>, AprError> {
        if !self.active {
            return Err(AprError::SerializationFailed(
                "Runtime not active".to_string(),
            ));
        }

        if self.is_finished() {
            return Ok(None);
        }

        // Get the next input - clone it to avoid borrow issues
        if let Some(timestamped_input) = self.runtime.next_input() {
            self.steps_executed += 1;
            let input = timestamped_input.input.clone();
            let tick = self.current_tick();

            // Create output based on input type
            let output = match input {
                AprInput::Syscall(ref syscall) => AprOutput {
                    tick,
                    output_type: "syscall".to_string(),
                    data: serde_json::to_value(syscall).unwrap_or_default(),
                },
                AprInput::Command(ref cmd) => AprOutput {
                    tick,
                    output_type: "command".to_string(),
                    data: serde_json::Value::String(cmd.clone()),
                },
                AprInput::KeyPress(key) => AprOutput {
                    tick,
                    output_type: "keypress".to_string(),
                    data: serde_json::Value::String(key.to_string()),
                },
                AprInput::Timer(ms) => AprOutput {
                    tick,
                    output_type: "timer".to_string(),
                    data: serde_json::Value::Number(serde_json::Number::from(ms)),
                },
                AprInput::Signal(ref sig) => AprOutput {
                    tick,
                    output_type: "signal".to_string(),
                    data: serde_json::to_value(sig).unwrap_or_default(),
                },
                AprInput::ExternalEvent(ref event) => AprOutput {
                    tick,
                    output_type: "external_event".to_string(),
                    data: serde_json::Value::String(event.clone()),
                },
                AprInput::Custom { ref name, ref data } => AprOutput {
                    tick,
                    output_type: format!("custom:{}", name),
                    data: data.clone(),
                },
            };

            self.record_output(output.clone());
            Ok(Some(output))
        } else {
            Ok(None)
        }
    }

    /// Get the number of steps executed
    pub fn steps_executed(&self) -> u64 {
        self.steps_executed
    }

    /// Create a checkpoint of current state
    pub fn checkpoint(&self, state: &KernelState) -> StateCheckpoint {
        StateCheckpoint {
            tick: self.current_tick(),
            state_hash: state.compute_hash(),
            description: format!("Checkpoint at step {}", self.steps_executed),
        }
    }

    /// Export the current state as an APR model
    pub fn export_model(&self, initial_state: &KernelState) -> AprModel {
        self.runtime.export_model(initial_state)
    }
}

/// APR execution result
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AprExecutionResult {
    /// Execution completed successfully
    Success {
        /// Number of steps executed
        steps: u64,
        /// Final tick
        final_tick: u64,
    },
    /// Execution failed with error
    Failed {
        /// Step at which failure occurred
        step: u64,
        /// Error description
        error: String,
    },
    /// Execution is in progress
    InProgress {
        /// Current step
        current_step: u64,
        /// Total steps
        total_steps: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use wos_shared::{AprFormat, AprMetadata, TimestampedInput};

    fn create_test_model() -> AprModel {
        AprModel {
            version: "1.0.0".to_string(),
            format: AprFormat::WosKernelState,
            seed: 42,
            initial_state: serde_json::json!({}),
            inputs: vec![
                TimestampedInput {
                    tick: 0,
                    input: AprInput::Command("echo hello".to_string()),
                },
                TimestampedInput {
                    tick: 1,
                    input: AprInput::Command("ps".to_string()),
                },
            ],
            expected_outputs: vec![],
            checkpoints: vec![],
            metadata: AprMetadata::default(),
        }
    }

    #[test]
    fn test_kernel_apr_runtime_creation() {
        let model = create_test_model();
        let runtime = KernelAprRuntime::new(model).unwrap();

        assert!(!runtime.is_active());
        assert!(!runtime.is_finished());
        assert_eq!(runtime.steps_executed(), 0);
    }

    #[test]
    fn test_kernel_apr_runtime_start_stop() {
        let model = create_test_model();
        let mut runtime = KernelAprRuntime::new(model).unwrap();

        assert!(!runtime.is_active());

        runtime.start().unwrap();
        assert!(runtime.is_active());

        runtime.stop();
        assert!(!runtime.is_active());
    }

    #[test]
    fn test_kernel_apr_runtime_double_start_fails() {
        let model = create_test_model();
        let mut runtime = KernelAprRuntime::new(model).unwrap();

        runtime.start().unwrap();
        let result = runtime.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_kernel_apr_runtime_step_when_inactive() {
        let model = create_test_model();
        let mut runtime = KernelAprRuntime::new(model).unwrap();

        let result = runtime.step();
        assert!(result.is_err());
    }

    #[test]
    fn test_kernel_apr_runtime_step_execution() {
        let model = create_test_model();
        let mut runtime = KernelAprRuntime::new(model).unwrap();

        runtime.start().unwrap();

        // First step
        let output1 = runtime.step().unwrap();
        assert!(output1.is_some());
        assert_eq!(runtime.steps_executed(), 1);

        // Second step
        let output2 = runtime.step().unwrap();
        assert!(output2.is_some());
        assert_eq!(runtime.steps_executed(), 2);

        // Third step should return None (finished)
        let output3 = runtime.step().unwrap();
        assert!(output3.is_none());
    }

    #[test]
    fn test_kernel_apr_runtime_from_json() {
        // AprInput uses #[serde(tag = "type", content = "data")] - adjacently tagged format
        // AprMetadata requires tags (Vec<String>) and properties (HashMap)
        let json = r#"{
            "version": "1.0.0",
            "format": "wos-kernel-state",
            "seed": 42,
            "initial_state": {},
            "inputs": [
                {"tick": 0, "input": {"type": "Command", "data": "test"}}
            ],
            "expected_outputs": [],
            "checkpoints": [],
            "metadata": {"tags": [], "properties": {}}
        }"#;

        let runtime = KernelAprRuntime::from_json(json);
        if let Err(ref e) = runtime {
            eprintln!("Error: {:?}", e);
        }
        assert!(runtime.is_ok(), "Failed to parse JSON: {:?}", runtime.err());
    }

    #[test]
    fn test_kernel_apr_runtime_from_invalid_json() {
        let json = "not valid json";
        let runtime = KernelAprRuntime::from_json(json);
        assert!(runtime.is_err());
    }

    #[test]
    fn test_apr_execution_result() {
        let success = AprExecutionResult::Success {
            steps: 10,
            final_tick: 100,
        };

        let failed = AprExecutionResult::Failed {
            step: 5,
            error: "test error".to_string(),
        };

        let in_progress = AprExecutionResult::InProgress {
            current_step: 3,
            total_steps: 10,
        };

        // Verify serialization
        let success_json = serde_json::to_string(&success).unwrap();
        assert!(success_json.contains("Success"));

        let failed_json = serde_json::to_string(&failed).unwrap();
        assert!(failed_json.contains("Failed"));

        let in_progress_json = serde_json::to_string(&in_progress).unwrap();
        assert!(in_progress_json.contains("InProgress"));
    }

    // Additional coverage tests
    mod coverage_tests {
        use super::*;
        use wos_shared::{AprFormat, AprMetadata, TimestampedInput};

        fn create_model_with_all_input_types() -> AprModel {
            AprModel {
                version: "1.0.0".to_string(),
                format: AprFormat::WosKernelState,
                seed: 42,
                initial_state: serde_json::json!({}),
                inputs: vec![
                    TimestampedInput {
                        tick: 0,
                        input: AprInput::Command("test".to_string()),
                    },
                    TimestampedInput {
                        tick: 1,
                        input: AprInput::KeyPress('a'),
                    },
                    TimestampedInput {
                        tick: 2,
                        input: AprInput::Timer(100),
                    },
                    TimestampedInput {
                        tick: 3,
                        input: AprInput::Signal(15), // SIGTERM
                    },
                    TimestampedInput {
                        tick: 4,
                        input: AprInput::ExternalEvent("network".to_string()),
                    },
                    TimestampedInput {
                        tick: 5,
                        input: AprInput::Custom {
                            name: "custom_event".to_string(),
                            data: serde_json::json!({"key": "value"}),
                        },
                    },
                    TimestampedInput {
                        tick: 6,
                        input: AprInput::Syscall(serde_json::json!({"call": "read"})),
                    },
                ],
                expected_outputs: vec![],
                checkpoints: vec![],
                metadata: AprMetadata::default(),
            }
        }

        #[test]
        fn test_step_all_input_types() {
            let model = create_model_with_all_input_types();
            let mut runtime = KernelAprRuntime::new(model).unwrap();

            runtime.start().unwrap();

            // Step through all input types
            let output1 = runtime.step().unwrap().unwrap();
            assert_eq!(output1.output_type, "command");

            let output2 = runtime.step().unwrap().unwrap();
            assert_eq!(output2.output_type, "keypress");

            let output3 = runtime.step().unwrap().unwrap();
            assert_eq!(output3.output_type, "timer");

            let output4 = runtime.step().unwrap().unwrap();
            assert_eq!(output4.output_type, "signal");

            let output5 = runtime.step().unwrap().unwrap();
            assert_eq!(output5.output_type, "external_event");

            let output6 = runtime.step().unwrap().unwrap();
            assert!(output6.output_type.starts_with("custom:"));

            let output7 = runtime.step().unwrap().unwrap();
            assert_eq!(output7.output_type, "syscall");
        }

        #[test]
        fn test_next_input_when_inactive() {
            let model = create_test_model();
            let mut runtime = KernelAprRuntime::new(model).unwrap();

            // Should fail when runtime not active
            let result = runtime.next_input();
            assert!(result.is_err());
        }

        #[test]
        fn test_next_input_when_active() {
            let model = create_test_model();
            let mut runtime = KernelAprRuntime::new(model).unwrap();

            runtime.start().unwrap();
            let input = runtime.next_input().unwrap();
            assert!(input.is_some());
        }

        #[test]
        fn test_verify_output() {
            let model = create_test_model();
            let runtime = KernelAprRuntime::new(model).unwrap();

            let output = AprOutput {
                tick: 0,
                output_type: "test".to_string(),
                data: serde_json::json!({}),
            };

            // verify_output should work (may return Ok or Err based on expected_outputs)
            let _ = runtime.verify_output(0, &output);
        }

        #[test]
        fn test_checkpoint() {
            let model = create_test_model();
            let runtime = KernelAprRuntime::new(model).unwrap();

            let state = crate::KernelState::default();
            let checkpoint = runtime.checkpoint(&state);

            assert_eq!(checkpoint.tick, 0);
            assert!(checkpoint.description.contains("Checkpoint"));
        }

        #[test]
        fn test_export_model() {
            let model = create_test_model();
            let runtime = KernelAprRuntime::new(model).unwrap();

            let state = crate::KernelState::default();
            let exported = runtime.export_model(&state);

            assert_eq!(exported.version, "1.0.0");
        }

        #[test]
        fn test_current_tick() {
            let model = create_test_model();
            let runtime = KernelAprRuntime::new(model).unwrap();

            assert_eq!(runtime.current_tick(), 0);
        }

        #[test]
        fn test_record_output() {
            let model = create_test_model();
            let mut runtime = KernelAprRuntime::new(model).unwrap();

            let output = AprOutput {
                tick: 0,
                output_type: "test".to_string(),
                data: serde_json::json!({}),
            };

            runtime.record_output(output);
            // Recording outputs should not panic
        }

        #[test]
        fn test_apr_execution_result_equality() {
            let result1 = AprExecutionResult::Success {
                steps: 10,
                final_tick: 100,
            };
            let result2 = AprExecutionResult::Success {
                steps: 10,
                final_tick: 100,
            };
            assert_eq!(result1, result2);

            let result3 = AprExecutionResult::Failed {
                step: 5,
                error: "error".to_string(),
            };
            let result4 = AprExecutionResult::Failed {
                step: 5,
                error: "error".to_string(),
            };
            assert_eq!(result3, result4);
        }

        #[test]
        fn test_apr_execution_result_clone() {
            let result = AprExecutionResult::InProgress {
                current_step: 5,
                total_steps: 10,
            };
            let cloned = result.clone();
            assert_eq!(result, cloned);
        }

        #[test]
        fn test_apr_execution_result_debug() {
            let result = AprExecutionResult::Success {
                steps: 10,
                final_tick: 100,
            };
            let debug_str = format!("{:?}", result);
            assert!(debug_str.contains("Success"));
        }
    }
}
