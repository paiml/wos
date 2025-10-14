//! Core User Programs
//!
//! User-level programs that run on WOS:
//! - echo: print arguments to stdout
//! - ls: list files in a directory
//! - ps: list running processes

use wos_kernel::{KernelState, ProcessId, SystemCall};

/// Echo program - prints arguments to stdout
#[derive(Clone, Debug, PartialEq)]
pub struct Echo {
    /// Process ID
    pub pid: ProcessId,
    /// Arguments to echo
    pub args: Vec<String>,
    /// Output generated
    pub output: String,
}

impl Echo {
    /// Create a new echo program
    pub fn new(pid: ProcessId, args: Vec<String>) -> Self {
        Self {
            pid,
            args,
            output: String::new(),
        }
    }

    /// Generate output from arguments
    pub fn generate_output(&mut self) {
        self.output = self.args.join(" ");
        if !self.output.is_empty() {
            self.output.push('\n');
        }
    }

    /// Get the output to write
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Echo main loop - generates output and writes to stdout
pub fn echo_main_loop(echo: &mut Echo, _state: &KernelState) -> Option<SystemCall> {
    if echo.output.is_empty() {
        // First iteration: generate output
        echo.generate_output();
        if !echo.output.is_empty() {
            // Write to stdout (fd=1)
            return Some(SystemCall::Write {
                fd: 1,
                data: echo.output.as_bytes().to_vec(),
            });
        }
    }

    // Done - exit
    Some(SystemCall::Exit(0))
}

/// Ls program - lists files in the VFS
#[derive(Clone, Debug, PartialEq)]
pub struct Ls {
    /// Process ID
    pub pid: ProcessId,
    /// Files found
    pub files: Vec<String>,
    /// Output generated
    pub output: String,
}

impl Ls {
    /// Create a new ls program
    pub fn new(pid: ProcessId) -> Self {
        Self {
            pid,
            files: Vec::new(),
            output: String::new(),
        }
    }

    /// List files from VFS
    pub fn list_files(&mut self, state: &KernelState) {
        // List all files from VFS
        self.files = state
            .vfs
            .list_files()
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        self.files.sort();
    }

    /// Generate output from file list
    pub fn generate_output(&mut self) {
        if self.files.is_empty() {
            self.output = String::new();
        } else {
            self.output = self.files.join("\n");
            self.output.push('\n');
        }
    }

    /// Get the output to write
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Ls main loop - lists files and writes to stdout
pub fn ls_main_loop(ls: &mut Ls, state: &KernelState) -> Option<SystemCall> {
    if ls.files.is_empty() && ls.output.is_empty() {
        // First iteration: list files
        ls.list_files(state);
        ls.generate_output();
        if !ls.output.is_empty() {
            // Write to stdout (fd=1)
            return Some(SystemCall::Write {
                fd: 1,
                data: ls.output.as_bytes().to_vec(),
            });
        }
    }

    // Done - exit
    Some(SystemCall::Exit(0))
}

/// Ps program - lists running processes
#[derive(Clone, Debug, PartialEq)]
pub struct Ps {
    /// Process ID
    pub pid: ProcessId,
    /// Process information
    pub processes: Vec<ProcessInfo>,
    /// Output generated
    pub output: String,
}

/// Process information for ps output
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: ProcessId,
    /// Parent process ID
    pub ppid: Option<ProcessId>,
    /// Process state
    pub state: String,
}

impl Ps {
    /// Create a new ps program
    pub fn new(pid: ProcessId) -> Self {
        Self {
            pid,
            processes: Vec::new(),
            output: String::new(),
        }
    }

    /// List processes from kernel state
    pub fn list_processes(&mut self, state: &KernelState) {
        self.processes.clear();

        for (&pid, process) in state.processes.iter() {
            let state_str = match &process.state {
                wos_kernel::ProcessState::Ready => "R",
                wos_kernel::ProcessState::Running => "R",
                wos_kernel::ProcessState::Blocked => "S",
                wos_kernel::ProcessState::Terminated(_) => "Z",
            };

            self.processes.push(ProcessInfo {
                pid,
                ppid: process.parent_pid,
                state: state_str.to_string(),
            });
        }

        // Sort by PID for consistent output
        self.processes.sort_by_key(|p| p.pid);
    }

    /// Generate output from process list
    pub fn generate_output(&mut self) {
        if self.processes.is_empty() {
            self.output = "PID  PPID STATE\n".to_string();
        } else {
            let mut lines = vec!["PID  PPID STATE".to_string()];
            for proc in &self.processes {
                let ppid_str = proc
                    .ppid
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "-".to_string());
                lines.push(format!("{:<4} {:<4} {}", proc.pid, ppid_str, proc.state));
            }
            self.output = lines.join("\n");
            self.output.push('\n');
        }
    }

    /// Get the output to write
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Ps main loop - lists processes and writes to stdout
pub fn ps_main_loop(ps: &mut Ps, state: &KernelState) -> Option<SystemCall> {
    if ps.processes.is_empty() && ps.output.is_empty() {
        // First iteration: list processes
        ps.list_processes(state);
        ps.generate_output();
        // Write to stdout (fd=1)
        return Some(SystemCall::Write {
            fd: 1,
            data: ps.output.as_bytes().to_vec(),
        });
    }

    // Done - exit
    Some(SystemCall::Exit(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wos_kernel::Process;

    #[test]
    fn test_echo_no_args() {
        let mut echo = Echo::new(2, vec![]);
        echo.generate_output();
        assert_eq!(echo.get_output(), "");
    }

    #[test]
    fn test_echo_single_arg() {
        let mut echo = Echo::new(2, vec!["hello".to_string()]);
        echo.generate_output();
        assert_eq!(echo.get_output(), "hello\n");
    }

    #[test]
    fn test_echo_multiple_args() {
        let mut echo = Echo::new(2, vec!["hello".to_string(), "world".to_string()]);
        echo.generate_output();
        assert_eq!(echo.get_output(), "hello world\n");
    }

    #[test]
    fn test_echo_main_loop_no_args() {
        let mut echo = Echo::new(2, vec![]);
        let state = KernelState::new();

        let syscall = echo_main_loop(&mut echo, &state);
        assert_eq!(syscall, Some(SystemCall::Exit(0)));
    }

    #[test]
    fn test_echo_main_loop_with_args() {
        let mut echo = Echo::new(2, vec!["test".to_string()]);
        let state = KernelState::new();

        // First call should write to stdout
        let syscall = echo_main_loop(&mut echo, &state);
        assert_eq!(
            syscall,
            Some(SystemCall::Write {
                fd: 1,
                data: b"test\n".to_vec()
            })
        );

        // Second call should exit
        let syscall = echo_main_loop(&mut echo, &state);
        assert_eq!(syscall, Some(SystemCall::Exit(0)));
    }

    #[test]
    fn test_ls_no_files() {
        let mut ls = Ls::new(2);
        let state = KernelState::new();

        ls.list_files(&state);
        ls.generate_output();
        assert_eq!(ls.get_output(), "");
    }

    #[test]
    fn test_ls_lists_files() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(std::path::PathBuf::from("/test.txt"), vec![])
            .unwrap();
        state
            .vfs
            .create_file(std::path::PathBuf::from("/another.txt"), vec![])
            .unwrap();

        let mut ls = Ls::new(2);
        ls.list_files(&state);

        assert_eq!(ls.files.len(), 2);
        assert!(ls.files.contains(&"/test.txt".to_string()));
        assert!(ls.files.contains(&"/another.txt".to_string()));
    }

    #[test]
    fn test_ls_generate_output() {
        let mut ls = Ls::new(2);
        ls.files = vec!["/file1.txt".to_string(), "/file2.txt".to_string()];
        ls.generate_output();

        assert_eq!(ls.get_output(), "/file1.txt\n/file2.txt\n");
    }

    #[test]
    fn test_ls_main_loop() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(std::path::PathBuf::from("/test.txt"), vec![])
            .unwrap();

        let mut ls = Ls::new(2);

        // First call should write to stdout
        let syscall = ls_main_loop(&mut ls, &state);
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 1, .. })));

        // Second call should exit
        let syscall = ls_main_loop(&mut ls, &state);
        assert_eq!(syscall, Some(SystemCall::Exit(0)));
    }

    #[test]
    fn test_ps_no_processes() {
        let mut ps = Ps::new(2);
        let state = KernelState::new();

        ps.list_processes(&state);
        ps.generate_output();

        // Should still show header
        assert!(ps.get_output().starts_with("PID  PPID STATE"));
    }

    #[test]
    fn test_ps_lists_processes() {
        let mut state = KernelState::new();
        let proc1 = Process::new(1, None);
        let proc2 = Process::new(2, Some(1));
        state.add_process(proc1);
        state.add_process(proc2);

        let mut ps = Ps::new(3);
        ps.list_processes(&state);

        assert_eq!(ps.processes.len(), 2);
        assert_eq!(ps.processes[0].pid, 1);
        assert_eq!(ps.processes[0].ppid, None);
        assert_eq!(ps.processes[1].pid, 2);
        assert_eq!(ps.processes[1].ppid, Some(1));
    }

    #[test]
    fn test_ps_generate_output() {
        let mut ps = Ps::new(2);
        ps.processes = vec![
            ProcessInfo {
                pid: 1,
                ppid: None,
                state: "R".to_string(),
            },
            ProcessInfo {
                pid: 2,
                ppid: Some(1),
                state: "R".to_string(),
            },
        ];
        ps.generate_output();

        let output = ps.get_output();
        assert!(output.contains("PID  PPID STATE"));
        assert!(output.contains("1    -    R"));
        assert!(output.contains("2    1    R"));
    }

    #[test]
    fn test_ps_main_loop() {
        let mut state = KernelState::new();
        let proc = Process::new(1, None);
        state.add_process(proc);

        let mut ps = Ps::new(2);

        // First call should write to stdout
        let syscall = ps_main_loop(&mut ps, &state);
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 1, .. })));

        // Second call should exit
        let syscall = ps_main_loop(&mut ps, &state);
        assert_eq!(syscall, Some(SystemCall::Exit(0)));
    }

    #[test]
    fn test_echo_integration() {
        let mut echo = Echo::new(2, vec!["hello".to_string(), "world".to_string()]);
        let state = KernelState::new();

        // Run the full lifecycle
        let syscall1 = echo_main_loop(&mut echo, &state);
        assert_eq!(
            syscall1,
            Some(SystemCall::Write {
                fd: 1,
                data: b"hello world\n".to_vec()
            })
        );

        let syscall2 = echo_main_loop(&mut echo, &state);
        assert_eq!(syscall2, Some(SystemCall::Exit(0)));
    }
}
