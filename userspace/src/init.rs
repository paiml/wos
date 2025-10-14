//! Init Process (PID 1)
//!
//! The init process is the first process started by the kernel and has special
//! responsibilities:
//! - Always has PID 1
//! - Launches the shell on startup
//! - Reaps orphaned processes (processes whose parents have died)
//! - Never exits (runs forever)

use wos_kernel::{KernelState, ProcessId, SystemCall};

/// Init process state
#[derive(Clone, Debug, PartialEq)]
pub struct InitProcess {
    /// Process ID (should always be 1)
    pub pid: ProcessId,
    /// Shell process ID (if launched)
    pub shell_pid: Option<ProcessId>,
    /// Child processes that init is responsible for
    pub children: Vec<ProcessId>,
}

impl InitProcess {
    /// Create a new init process
    ///
    /// # Panics
    /// Panics if pid is not 1
    pub fn new(pid: ProcessId) -> Self {
        assert_eq!(pid, 1, "Init process must have PID 1");
        Self {
            pid,
            shell_pid: None,
            children: Vec::new(),
        }
    }

    /// Launch the shell process
    ///
    /// Returns the syscall to fork and create the shell
    pub fn launch_shell(&mut self) -> SystemCall {
        SystemCall::Fork
    }

    /// Handle a forked shell process
    ///
    /// Should be called after Fork syscall succeeds
    pub fn set_shell_pid(&mut self, pid: ProcessId) {
        self.shell_pid = Some(pid);
        self.children.push(pid);
    }

    /// Add a child process (orphan that was reparented to init)
    pub fn add_child(&mut self, pid: ProcessId) {
        if !self.children.contains(&pid) {
            self.children.push(pid);
        }
    }

    /// Remove a child process (after it has been reaped)
    pub fn remove_child(&mut self, pid: ProcessId) {
        self.children.retain(|&p| p != pid);
    }

    /// Check if init has any children to reap
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get next child to wait for
    pub fn next_child(&self) -> Option<ProcessId> {
        self.children.first().copied()
    }

    /// Reap a child process
    ///
    /// Returns the syscall to wait for the child
    pub fn reap_child(&self, pid: ProcessId) -> SystemCall {
        SystemCall::WaitPid(pid)
    }
}

/// Init process main loop logic
///
/// This represents one iteration of the init process's main loop.
/// Returns the next syscall to execute, or None if init should yield.
pub fn init_main_loop(init: &InitProcess, state: &KernelState) -> Option<SystemCall> {
    // If shell hasn't been launched yet, launch it
    if init.shell_pid.is_none() {
        return Some(SystemCall::Fork);
    }

    // Check for terminated children to reap
    for &child_pid in &init.children {
        if let Some(child) = state.get_process(child_pid) {
            if child.is_terminated() {
                return Some(SystemCall::WaitPid(child_pid));
            }
        }
    }

    // No work to do - yield
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use wos_kernel::{Process, ProcessState};

    #[test]
    fn test_init_is_pid_1() {
        let init = InitProcess::new(1);
        assert_eq!(init.pid, 1);
    }

    #[test]
    #[should_panic(expected = "Init process must have PID 1")]
    fn test_init_must_be_pid_1() {
        InitProcess::new(2);
    }

    #[test]
    fn test_init_launches_shell() {
        let mut init = InitProcess::new(1);
        assert_eq!(init.shell_pid, None);

        let syscall = init.launch_shell();
        assert_eq!(syscall, SystemCall::Fork);
    }

    #[test]
    fn test_init_sets_shell_pid() {
        let mut init = InitProcess::new(1);
        init.set_shell_pid(2);

        assert_eq!(init.shell_pid, Some(2));
        assert!(init.children.contains(&2));
    }

    #[test]
    fn test_init_adds_children() {
        let mut init = InitProcess::new(1);
        init.add_child(3);
        init.add_child(4);

        assert_eq!(init.children.len(), 2);
        assert!(init.children.contains(&3));
        assert!(init.children.contains(&4));
    }

    #[test]
    fn test_init_removes_children() {
        let mut init = InitProcess::new(1);
        init.add_child(3);
        init.add_child(4);

        init.remove_child(3);

        assert_eq!(init.children.len(), 1);
        assert!(!init.children.contains(&3));
        assert!(init.children.contains(&4));
    }

    #[test]
    fn test_init_has_children() {
        let mut init = InitProcess::new(1);
        assert!(!init.has_children());

        init.add_child(2);
        assert!(init.has_children());
    }

    #[test]
    fn test_init_reaps_orphans() {
        let mut init = InitProcess::new(1);

        // Simulate orphaned process
        init.add_child(5);

        let syscall = init.reap_child(5);
        assert_eq!(syscall, SystemCall::WaitPid(5));

        // After reaping
        init.remove_child(5);
        assert!(!init.children.contains(&5));
    }

    #[test]
    fn test_init_main_loop_launches_shell() {
        let init = InitProcess::new(1);
        let state = KernelState::new();

        let syscall = init_main_loop(&init, &state);
        assert_eq!(syscall, Some(SystemCall::Fork));
    }

    #[test]
    fn test_init_main_loop_reaps_terminated_child() {
        let mut state = KernelState::new();

        // Create init process
        let mut init = InitProcess::new(1);
        init.set_shell_pid(2);

        // Create terminated child process
        let mut child = Process::new(2, Some(1));
        child.state = ProcessState::Terminated(0);
        state.add_process(child);

        let syscall = init_main_loop(&init, &state);
        assert_eq!(syscall, Some(SystemCall::WaitPid(2)));
    }

    #[test]
    fn test_init_main_loop_yields_when_idle() {
        let mut init = InitProcess::new(1);
        init.set_shell_pid(2);

        let mut state = KernelState::new();
        // Create running child
        let child = Process::new(2, Some(1));
        state.add_process(child);

        let syscall = init_main_loop(&init, &state);
        assert_eq!(syscall, None);
    }

    #[test]
    fn test_init_never_exits() {
        // Init should never generate an Exit syscall
        let init = InitProcess::new(1);
        let state = KernelState::new();

        // Multiple iterations should never produce Exit
        for _ in 0..100 {
            let syscall = init_main_loop(&init, &state);
            if let Some(call) = syscall {
                assert!(!matches!(call, SystemCall::Exit(_)));
            }
        }
    }
}
