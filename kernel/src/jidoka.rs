//! Jidoka Guards for Kernel Invariants
//!
//! Implements Toyota Production System's Jidoka (automation with a human touch)
//! principle: automatically stop when an abnormality is detected.
//!
//! # Philosophy
//!
//! Rather than allowing the system to continue in an invalid state and produce
//! cascading failures, Jidoka guards halt execution immediately when invariants
//! are violated. This makes bugs obvious and prevents data corruption.
//!
//! # Guard Types
//!
//! - **Process Guards**: PID uniqueness, zombie limits, orphan handling
//! - **Memory Guards**: Allocation bounds, no overlaps, consistent totals
//! - **File System Guards**: Path validity, descriptor limits
//! - **VM Guards**: Isolation, resource limits, state consistency

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum allowed processes before Jidoka halt
pub const MAX_PROCESSES: usize = 1000;

/// Maximum allowed zombie processes before Jidoka halt
pub const MAX_ZOMBIES: usize = 100;

/// Maximum memory allocation (256 MB)
pub const MAX_MEMORY_BYTES: usize = 256 * 1024 * 1024;

/// Maximum open file descriptors per process
pub const MAX_FDS_PER_PROCESS: usize = 256;

/// Maximum total open file descriptors
pub const MAX_TOTAL_FDS: usize = 4096;

/// Maximum VMs allowed
pub const MAX_VMS: usize = 16;

/// Result of a Jidoka check
#[derive(Clone, Debug, PartialEq)]
pub enum JidokaStatus {
    /// All invariants satisfied
    Ok,
    /// One or more invariants violated - halt immediately
    Halt(Vec<Violation>),
}

impl JidokaStatus {
    /// Check if status is OK
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if status is halt
    pub fn is_halt(&self) -> bool {
        matches!(self, Self::Halt(_))
    }

    /// Get violations if any
    pub fn violations(&self) -> Option<&[Violation]> {
        match self {
            Self::Ok => None,
            Self::Halt(v) => Some(v),
        }
    }
}

/// An invariant violation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Violation {
    /// Category of the violation
    pub category: ViolationCategory,
    /// Human-readable description
    pub message: String,
    /// Severity level
    pub severity: Severity,
    /// Additional context data
    pub context: Option<String>,
}

impl Violation {
    /// Create a new violation
    pub fn new(
        category: ViolationCategory,
        message: impl Into<String>,
        severity: Severity,
    ) -> Self {
        Self {
            category,
            message: message.into(),
            severity,
            context: None,
        }
    }

    /// Add context to violation
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}] {:?}: {}",
            self.severity, self.category, self.message
        )?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

/// Categories of invariant violations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationCategory {
    /// Process-related violation
    Process,
    /// Memory-related violation
    Memory,
    /// File system violation
    FileSystem,
    /// VM/virtualization violation
    Virtualization,
    /// Scheduler violation
    Scheduler,
    /// IPC violation
    Ipc,
    /// Resource exhaustion
    Resource,
    /// Internal consistency violation
    Consistency,
}

/// Severity levels for violations
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Warning - log but continue
    Warning,
    /// Error - should halt but may recover
    Error,
    /// Critical - must halt immediately
    Critical,
}

/// Invariant check types
#[derive(Clone, Debug, PartialEq)]
pub enum InvariantCheck {
    /// Process count within limits
    ProcessCountLimit(usize),
    /// Memory usage within limits
    MemoryLimit(usize),
    /// File descriptor count within limits
    FdLimit(usize),
    /// No orphan processes (all have valid parent or are init)
    NoOrphanProcesses,
    /// Zombie count within limits
    NoZombieBloat(usize),
    /// VM count within limits
    VmCountLimit(usize),
    /// VM memory within bounds
    VmMemoryBounds,
    /// Deterministic RNG state valid
    DeterministicRng,
    /// PID uniqueness
    UniquePids,
    /// No memory overlaps
    NoMemoryOverlaps,
    /// Consistent memory totals
    ConsistentMemoryTotals,
    /// Valid process states
    ValidProcessStates,
    /// Scheduler queue consistency
    SchedulerConsistency,
    /// Custom invariant
    Custom(String),
}

/// Jidoka guard for kernel state
pub struct JidokaGuard {
    /// Enabled invariant checks
    checks: Vec<InvariantCheck>,
    /// Recorded violations
    violations: Vec<Violation>,
    /// Whether to halt on first violation
    halt_on_first: bool,
}

impl Default for JidokaGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl JidokaGuard {
    /// Create a new Jidoka guard with default checks
    pub fn new() -> Self {
        Self {
            checks: vec![
                InvariantCheck::ProcessCountLimit(MAX_PROCESSES),
                InvariantCheck::MemoryLimit(MAX_MEMORY_BYTES),
                InvariantCheck::FdLimit(MAX_TOTAL_FDS),
                InvariantCheck::NoOrphanProcesses,
                InvariantCheck::NoZombieBloat(MAX_ZOMBIES),
                InvariantCheck::UniquePids,
                InvariantCheck::ValidProcessStates,
            ],
            violations: Vec::new(),
            halt_on_first: false,
        }
    }

    /// Create a strict guard that halts on first violation
    pub fn strict() -> Self {
        let mut guard = Self::new();
        guard.halt_on_first = true;
        guard
    }

    /// Add a custom check
    pub fn add_check(&mut self, check: InvariantCheck) {
        self.checks.push(check);
    }

    /// Check kernel state for violations
    pub fn check<S: KernelStateView>(&mut self, state: &S) -> JidokaStatus {
        self.violations.clear();

        for check in &self.checks {
            if let Some(violation) = self.verify_invariant(check, state) {
                self.violations.push(violation);
                if self.halt_on_first {
                    break;
                }
            }
        }

        if self.violations.is_empty() {
            JidokaStatus::Ok
        } else {
            JidokaStatus::Halt(self.violations.clone())
        }
    }

    /// Verify a single invariant
    fn verify_invariant<S: KernelStateView>(
        &self,
        check: &InvariantCheck,
        state: &S,
    ) -> Option<Violation> {
        match check {
            InvariantCheck::ProcessCountLimit(max) => {
                let count = state.process_count();
                if count > *max {
                    Some(Violation::new(
                        ViolationCategory::Process,
                        format!("Process count {} exceeds limit {}", count, max),
                        Severity::Critical,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::MemoryLimit(max) => {
                let used = state.memory_used();
                if used > *max {
                    Some(Violation::new(
                        ViolationCategory::Memory,
                        format!("Memory usage {} exceeds limit {}", used, max),
                        Severity::Critical,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::FdLimit(max) => {
                let count = state.total_fd_count();
                if count > *max {
                    Some(Violation::new(
                        ViolationCategory::FileSystem,
                        format!("FD count {} exceeds limit {}", count, max),
                        Severity::Critical,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::NoOrphanProcesses => {
                if state.has_orphan_processes() {
                    Some(Violation::new(
                        ViolationCategory::Process,
                        "Orphan processes detected (not reparented to init)",
                        Severity::Error,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::NoZombieBloat(max) => {
                let zombies = state.zombie_count();
                if zombies > *max {
                    Some(Violation::new(
                        ViolationCategory::Process,
                        format!("Zombie count {} exceeds limit {}", zombies, max),
                        Severity::Error,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::VmCountLimit(max) => {
                let count = state.vm_count();
                if count > *max {
                    Some(Violation::new(
                        ViolationCategory::Virtualization,
                        format!("VM count {} exceeds limit {}", count, max),
                        Severity::Critical,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::VmMemoryBounds => {
                if !state.vm_memory_within_bounds() {
                    Some(Violation::new(
                        ViolationCategory::Virtualization,
                        "VM memory exceeds allocated bounds",
                        Severity::Critical,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::DeterministicRng => {
                if !state.rng_state_valid() {
                    Some(Violation::new(
                        ViolationCategory::Consistency,
                        "RNG state is invalid or non-deterministic",
                        Severity::Error,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::UniquePids => {
                if !state.all_pids_unique() {
                    Some(Violation::new(
                        ViolationCategory::Process,
                        "Duplicate PIDs detected",
                        Severity::Critical,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::NoMemoryOverlaps => {
                if state.has_memory_overlaps() {
                    Some(Violation::new(
                        ViolationCategory::Memory,
                        "Memory region overlaps detected",
                        Severity::Critical,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::ConsistentMemoryTotals => {
                if !state.memory_totals_consistent() {
                    Some(Violation::new(
                        ViolationCategory::Memory,
                        "Memory totals are inconsistent",
                        Severity::Error,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::ValidProcessStates => {
                if !state.all_process_states_valid() {
                    Some(Violation::new(
                        ViolationCategory::Process,
                        "Invalid process state transition detected",
                        Severity::Error,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::SchedulerConsistency => {
                if !state.scheduler_consistent() {
                    Some(Violation::new(
                        ViolationCategory::Scheduler,
                        "Scheduler queue inconsistency detected",
                        Severity::Error,
                    ))
                } else {
                    None
                }
            }

            InvariantCheck::Custom(name) => {
                // Custom checks return None by default
                // Override with custom implementation
                let _ = name;
                None
            }
        }
    }

    /// Get all violations from last check
    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    /// Clear recorded violations
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }
}

/// Trait for kernel state inspection
///
/// Implement this trait to allow Jidoka guards to inspect kernel state.
pub trait KernelStateView {
    /// Get total process count
    fn process_count(&self) -> usize;

    /// Get memory used in bytes
    fn memory_used(&self) -> usize;

    /// Get total open file descriptor count
    fn total_fd_count(&self) -> usize;

    /// Check if there are orphan processes
    fn has_orphan_processes(&self) -> bool;

    /// Get zombie process count
    fn zombie_count(&self) -> usize;

    /// Get VM count
    fn vm_count(&self) -> usize;

    /// Check if all VM memory is within bounds
    fn vm_memory_within_bounds(&self) -> bool;

    /// Check if RNG state is valid
    fn rng_state_valid(&self) -> bool;

    /// Check if all PIDs are unique
    fn all_pids_unique(&self) -> bool;

    /// Check for memory region overlaps
    fn has_memory_overlaps(&self) -> bool;

    /// Check if memory totals are consistent
    fn memory_totals_consistent(&self) -> bool;

    /// Check if all process states are valid
    fn all_process_states_valid(&self) -> bool;

    /// Check scheduler consistency
    fn scheduler_consistent(&self) -> bool;
}

/// Simple mock state for testing
#[derive(Default)]
pub struct MockKernelState {
    /// Number of processes in the system
    pub process_count: usize,
    /// Total memory used in bytes
    pub memory_used: usize,
    /// Total file descriptors open across all processes
    pub total_fds: usize,
    /// Whether orphan processes exist (children without parents)
    pub has_orphans: bool,
    /// Number of zombie processes
    pub zombie_count: usize,
    /// Number of virtual machines
    pub vm_count: usize,
    /// Whether duplicate PIDs exist (invariant violation)
    pub duplicate_pids: bool,
}

impl KernelStateView for MockKernelState {
    fn process_count(&self) -> usize {
        self.process_count
    }

    fn memory_used(&self) -> usize {
        self.memory_used
    }

    fn total_fd_count(&self) -> usize {
        self.total_fds
    }

    fn has_orphan_processes(&self) -> bool {
        self.has_orphans
    }

    fn zombie_count(&self) -> usize {
        self.zombie_count
    }

    fn vm_count(&self) -> usize {
        self.vm_count
    }

    fn vm_memory_within_bounds(&self) -> bool {
        true
    }

    fn rng_state_valid(&self) -> bool {
        true
    }

    fn all_pids_unique(&self) -> bool {
        !self.duplicate_pids
    }

    fn has_memory_overlaps(&self) -> bool {
        false
    }

    fn memory_totals_consistent(&self) -> bool {
        true
    }

    fn all_process_states_valid(&self) -> bool {
        true
    }

    fn scheduler_consistent(&self) -> bool {
        true
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jidoka_guard_ok() {
        let mut guard = JidokaGuard::new();
        let state = MockKernelState::default();
        let status = guard.check(&state);
        assert!(status.is_ok());
    }

    #[test]
    fn test_jidoka_process_limit() {
        let mut guard = JidokaGuard::new();
        let state = MockKernelState {
            process_count: MAX_PROCESSES + 1,
            ..Default::default()
        };
        let status = guard.check(&state);
        assert!(status.is_halt());

        let violations = status.violations().unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].category, ViolationCategory::Process);
        assert_eq!(violations[0].severity, Severity::Critical);
    }

    #[test]
    fn test_jidoka_memory_limit() {
        let mut guard = JidokaGuard::new();
        let state = MockKernelState {
            memory_used: MAX_MEMORY_BYTES + 1,
            ..Default::default()
        };
        let status = guard.check(&state);
        assert!(status.is_halt());

        let violations = status.violations().unwrap();
        assert!(violations
            .iter()
            .any(|v| v.category == ViolationCategory::Memory));
    }

    #[test]
    fn test_jidoka_zombie_limit() {
        let mut guard = JidokaGuard::new();
        let state = MockKernelState {
            zombie_count: MAX_ZOMBIES + 1,
            ..Default::default()
        };
        let status = guard.check(&state);
        assert!(status.is_halt());
    }

    #[test]
    fn test_jidoka_orphan_detection() {
        let mut guard = JidokaGuard::new();
        let state = MockKernelState {
            has_orphans: true,
            ..Default::default()
        };
        let status = guard.check(&state);
        assert!(status.is_halt());
    }

    #[test]
    fn test_jidoka_duplicate_pids() {
        let mut guard = JidokaGuard::new();
        let state = MockKernelState {
            duplicate_pids: true,
            ..Default::default()
        };
        let status = guard.check(&state);
        assert!(status.is_halt());

        let violations = status.violations().unwrap();
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Duplicate PIDs")));
    }

    #[test]
    fn test_jidoka_strict_mode() {
        let mut guard = JidokaGuard::strict();
        let state = MockKernelState {
            process_count: MAX_PROCESSES + 1,
            memory_used: MAX_MEMORY_BYTES + 1,
            ..Default::default()
        };
        let status = guard.check(&state);

        // Strict mode halts on first violation
        assert!(status.is_halt());
        let violations = status.violations().unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_jidoka_multiple_violations() {
        let mut guard = JidokaGuard::new();
        let state = MockKernelState {
            process_count: MAX_PROCESSES + 1,
            memory_used: MAX_MEMORY_BYTES + 1,
            zombie_count: MAX_ZOMBIES + 1,
            ..Default::default()
        };
        let status = guard.check(&state);
        assert!(status.is_halt());

        // Non-strict mode collects all violations
        let violations = status.violations().unwrap();
        assert!(violations.len() >= 3);
    }

    #[test]
    fn test_violation_display() {
        let violation = Violation::new(
            ViolationCategory::Process,
            "Test violation",
            Severity::Critical,
        )
        .with_context("additional info");

        let display = violation.to_string();
        assert!(display.contains("Process"));
        assert!(display.contains("Test violation"));
        assert!(display.contains("additional info"));
    }

    #[test]
    fn test_custom_check() {
        let mut guard = JidokaGuard::new();
        guard.add_check(InvariantCheck::Custom("custom_check".to_string()));

        let state = MockKernelState::default();
        let status = guard.check(&state);
        assert!(status.is_ok());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    // Additional coverage tests
    mod coverage_tests {
        use super::*;

        #[test]
        fn test_jidoka_status_ok_violations() {
            let status = JidokaStatus::Ok;
            assert!(status.is_ok());
            assert!(!status.is_halt());
            assert!(status.violations().is_none());
        }

        #[test]
        fn test_jidoka_status_halt_violations() {
            let violations = vec![Violation::new(
                ViolationCategory::Process,
                "test",
                Severity::Critical,
            )];
            let status = JidokaStatus::Halt(violations.clone());
            assert!(!status.is_ok());
            assert!(status.is_halt());
            let got = status.violations().unwrap();
            assert_eq!(got.len(), 1);
        }

        #[test]
        fn test_jidoka_fd_limit() {
            let mut guard = JidokaGuard::new();
            let state = MockKernelState {
                total_fds: MAX_TOTAL_FDS + 1,
                ..Default::default()
            };
            let status = guard.check(&state);
            assert!(status.is_halt());

            let violations = status.violations().unwrap();
            assert!(violations
                .iter()
                .any(|v| v.category == ViolationCategory::FileSystem));
        }

        #[test]
        fn test_jidoka_vm_count_limit() {
            let mut guard = JidokaGuard::new();
            guard.add_check(InvariantCheck::VmCountLimit(MAX_VMS));

            let state = MockKernelState {
                vm_count: MAX_VMS + 1,
                ..Default::default()
            };
            let status = guard.check(&state);
            assert!(status.is_halt());

            let violations = status.violations().unwrap();
            assert!(violations
                .iter()
                .any(|v| v.category == ViolationCategory::Virtualization));
        }

        #[test]
        fn test_jidoka_vm_memory_bounds() {
            let mut guard = JidokaGuard::new();
            guard.add_check(InvariantCheck::VmMemoryBounds);

            // Default MockKernelState returns true for vm_memory_within_bounds
            let state = MockKernelState::default();
            let status = guard.check(&state);
            assert!(status.is_ok());
        }

        #[test]
        fn test_jidoka_deterministic_rng() {
            let mut guard = JidokaGuard::new();
            guard.add_check(InvariantCheck::DeterministicRng);

            // Default MockKernelState returns true for rng_state_valid
            let state = MockKernelState::default();
            let status = guard.check(&state);
            assert!(status.is_ok());
        }

        #[test]
        fn test_jidoka_no_memory_overlaps() {
            let mut guard = JidokaGuard::new();
            guard.add_check(InvariantCheck::NoMemoryOverlaps);

            // Default MockKernelState returns false for has_memory_overlaps
            let state = MockKernelState::default();
            let status = guard.check(&state);
            assert!(status.is_ok());
        }

        #[test]
        fn test_jidoka_consistent_memory_totals() {
            let mut guard = JidokaGuard::new();
            guard.add_check(InvariantCheck::ConsistentMemoryTotals);

            // Default MockKernelState returns true for memory_totals_consistent
            let state = MockKernelState::default();
            let status = guard.check(&state);
            assert!(status.is_ok());
        }

        #[test]
        fn test_jidoka_scheduler_consistency() {
            let mut guard = JidokaGuard::new();
            guard.add_check(InvariantCheck::SchedulerConsistency);

            // Default MockKernelState returns true for scheduler_consistent
            let state = MockKernelState::default();
            let status = guard.check(&state);
            assert!(status.is_ok());
        }

        #[test]
        fn test_jidoka_clear_violations() {
            let mut guard = JidokaGuard::new();
            let state = MockKernelState {
                process_count: MAX_PROCESSES + 1,
                ..Default::default()
            };

            let status = guard.check(&state);
            assert!(status.is_halt());
            assert!(!guard.violations().is_empty());

            guard.clear_violations();
            assert!(guard.violations().is_empty());
        }

        #[test]
        fn test_violation_without_context() {
            let violation =
                Violation::new(ViolationCategory::Memory, "Memory error", Severity::Error);

            let display = violation.to_string();
            assert!(display.contains("Memory"));
            assert!(display.contains("Memory error"));
            assert!(!display.contains("(")); // No context parentheses
        }

        #[test]
        fn test_violation_category_serialization() {
            let categories = vec![
                ViolationCategory::Process,
                ViolationCategory::Memory,
                ViolationCategory::FileSystem,
                ViolationCategory::Virtualization,
                ViolationCategory::Scheduler,
                ViolationCategory::Ipc,
                ViolationCategory::Resource,
                ViolationCategory::Consistency,
            ];

            for category in categories {
                let json = serde_json::to_string(&category).unwrap();
                let restored: ViolationCategory = serde_json::from_str(&json).unwrap();
                assert_eq!(category, restored);
            }
        }

        #[test]
        fn test_severity_serialization() {
            let severities = vec![Severity::Warning, Severity::Error, Severity::Critical];

            for severity in severities {
                let json = serde_json::to_string(&severity).unwrap();
                let restored: Severity = serde_json::from_str(&json).unwrap();
                assert_eq!(severity, restored);
            }
        }

        #[test]
        fn test_violation_serialization() {
            let violation = Violation::new(
                ViolationCategory::Process,
                "test violation",
                Severity::Critical,
            )
            .with_context("context info");

            let json = serde_json::to_string(&violation).unwrap();
            let restored: Violation = serde_json::from_str(&json).unwrap();
            assert_eq!(violation, restored);
        }

        #[test]
        fn test_invariant_check_equality() {
            let check1 = InvariantCheck::ProcessCountLimit(100);
            let check2 = InvariantCheck::ProcessCountLimit(100);
            assert_eq!(check1, check2);

            let check3 = InvariantCheck::Custom("test".to_string());
            let check4 = InvariantCheck::Custom("test".to_string());
            assert_eq!(check3, check4);
        }

        #[test]
        fn test_mock_kernel_state_defaults() {
            let state = MockKernelState::default();

            assert_eq!(state.process_count(), 0);
            assert_eq!(state.memory_used(), 0);
            assert_eq!(state.total_fd_count(), 0);
            assert!(!state.has_orphan_processes());
            assert_eq!(state.zombie_count(), 0);
            assert_eq!(state.vm_count(), 0);
            assert!(state.vm_memory_within_bounds());
            assert!(state.rng_state_valid());
            assert!(state.all_pids_unique());
            assert!(!state.has_memory_overlaps());
            assert!(state.memory_totals_consistent());
            assert!(state.all_process_states_valid());
            assert!(state.scheduler_consistent());
        }

        #[test]
        fn test_jidoka_guard_default() {
            let guard = JidokaGuard::default();
            assert!(guard.violations().is_empty());
        }
    }
}
