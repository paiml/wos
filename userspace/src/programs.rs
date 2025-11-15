//! Core User Programs
//!
//! User-level programs that run on WOS:
//! - echo: print arguments to stdout
//! - ls: list files in a directory
//! - ps: list running processes
//! - vim: modal text editor

use crate::vim::state::VisualMode;
use crate::vim::{VimCommand, VimMode, VimState};
use std::path::PathBuf;
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

/// Vim program - modal text editor
#[derive(Clone, Debug)]
pub struct Vim {
    /// Process ID
    pub pid: ProcessId,
    /// File path being edited
    pub file_path: Option<PathBuf>,
    /// Vim editor state
    pub vim_state: VimState,
    /// Input buffer (keystrokes received)
    pub input_buffer: Vec<char>,
    /// Screen output buffer (for rendering)
    pub screen_buffer: String,
    /// Has rendered initial screen
    pub initialized: bool,
    /// Exit requested
    pub exit_requested: bool,
}

impl Vim {
    /// Create a new vim instance
    ///
    /// Note: File content loading is handled at the integration layer (wos/src/lib.rs)
    /// via VFS access. The vim_state can be set after construction via direct field access.
    pub fn new(pid: ProcessId, file_path: Option<PathBuf>) -> Self {
        // Start with empty vim state
        // File content is loaded at integration layer and set via vim.vim_state field
        let vim_state = VimState::new();

        Self {
            pid,
            file_path,
            vim_state,
            input_buffer: Vec::new(),
            screen_buffer: String::new(),
            initialized: false,
            exit_requested: false,
        }
    }

    /// Render the vim editor screen
    pub fn render_screen(&mut self) {
        let buffer = self.vim_state.current_buffer();
        let mut output = String::new();

        // Render buffer content (simplified for now)
        for (i, line) in buffer.lines.iter().enumerate() {
            if i == buffer.cursor.line {
                // Show cursor position with a marker
                output.push_str(&format!("{}~\n", line));
            } else {
                output.push_str(&format!("{}\n", line));
            }
        }

        // Render status line
        let file_display = self
            .file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "[No Name]".to_string());

        let modified_flag = if buffer.modified { " [+]" } else { "" };
        let mode_str = format!("{}", self.vim_state.mode);

        output.push_str(&format!(
            "\n--- {} {}{} - {} ---\n",
            file_display,
            buffer.cursor.line + 1,
            modified_flag,
            mode_str
        ));

        // Show message if any
        if !self.vim_state.message.is_empty() {
            output.push_str(&self.vim_state.message);
            output.push('\n');
        }

        self.screen_buffer = output;
    }

    /// Process input character
    pub fn process_input(&mut self, ch: char) -> Result<(), String> {
        match &self.vim_state.mode {
            VimMode::Normal => {
                // Handle normal mode keys
                match ch {
                    'i' => {
                        self.vim_state.set_mode(VimMode::Insert);
                        Ok(())
                    }
                    ':' => {
                        self.vim_state.set_mode(VimMode::Command {
                            buffer: String::new(),
                        });
                        Ok(())
                    }
                    'h' | 'j' | 'k' | 'l' | 'x' | 'u' | 'r' => {
                        // Parse and execute vim command
                        match crate::vim::parser::parse_normal_key(ch) {
                            Ok(cmd) => {
                                let buffer = self.vim_state.current_buffer_mut();
                                cmd.execute(buffer).map_err(|e| e.to_string())?;
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    'v' => {
                        // Enter visual character mode
                        let buffer = self.vim_state.current_buffer_mut();
                        VimCommand::EnterVisualChar
                            .execute(buffer)
                            .map_err(|e| e.to_string())?;
                        self.vim_state
                            .set_mode(VimMode::Visual(VisualMode::Character));
                        Ok(())
                    }
                    'V' => {
                        // Enter visual line mode
                        let buffer = self.vim_state.current_buffer_mut();
                        VimCommand::EnterVisualLine
                            .execute(buffer)
                            .map_err(|e| e.to_string())?;
                        self.vim_state.set_mode(VimMode::Visual(VisualMode::Line));
                        Ok(())
                    }
                    '\x16' => {
                        // Ctrl+v - Enter visual block mode
                        let buffer = self.vim_state.current_buffer_mut();
                        VimCommand::EnterVisualBlock
                            .execute(buffer)
                            .map_err(|e| e.to_string())?;
                        self.vim_state.set_mode(VimMode::Visual(VisualMode::Block));
                        Ok(())
                    }
                    _ => Ok(()), // Ignore unknown keys
                }
            }
            VimMode::Insert => {
                // Handle insert mode keys
                if ch == '\x1b' {
                    // ESC key - return to normal mode
                    self.vim_state.set_mode(VimMode::Normal);
                    Ok(())
                } else {
                    // Insert character
                    let cmd = if ch == '\n' {
                        VimCommand::InsertNewline
                    } else if ch == '\x08' || ch == '\x7f' {
                        // Backspace
                        VimCommand::Backspace
                    } else {
                        VimCommand::InsertChar(ch)
                    };

                    let buffer = self.vim_state.current_buffer_mut();
                    cmd.execute(buffer).map_err(|e| e.to_string())?;
                    Ok(())
                }
            }
            VimMode::Visual(_) => {
                // Handle visual mode keys
                match ch {
                    '\x1b' => {
                        // ESC - Clear visual anchor and mode, return to normal mode
                        let buffer = self.vim_state.current_buffer_mut();
                        buffer.visual_anchor = None;
                        buffer.visual_mode = None;
                        self.vim_state.set_mode(VimMode::Normal);
                        Ok(())
                    }
                    'h' | 'j' | 'k' | 'l' => {
                        // Navigation in visual mode - move cursor but keep anchor
                        match crate::vim::parser::parse_normal_key(ch) {
                            Ok(cmd) => {
                                let buffer = self.vim_state.current_buffer_mut();
                                cmd.execute(buffer).map_err(|e| e.to_string())?;
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    'd' | 'x' => {
                        // Delete visual selection
                        let buffer = self.vim_state.current_buffer_mut();
                        VimCommand::VisualDelete
                            .execute(buffer)
                            .map_err(|e| e.to_string())?;
                        self.vim_state.set_mode(VimMode::Normal);
                        Ok(())
                    }
                    'y' => {
                        // Yank visual selection (placeholder for now)
                        let buffer = self.vim_state.current_buffer_mut();
                        VimCommand::VisualYank
                            .execute(buffer)
                            .map_err(|e| e.to_string())?;
                        self.vim_state.set_mode(VimMode::Normal);
                        Ok(())
                    }
                    _ => Ok(()), // Ignore unknown keys
                }
            }
            VimMode::Command { buffer: cmd_buffer } => {
                if ch == '\n' {
                    // Execute command
                    let cmd = cmd_buffer.clone();
                    match crate::vim::ex_commands::execute_ex_command(&mut self.vim_state, &cmd) {
                        Ok(msg) => {
                            self.vim_state.set_message(msg);
                            self.vim_state.set_mode(VimMode::Normal);

                            // Check if quit was requested
                            if cmd.trim() == "q"
                                || cmd.trim() == "q!"
                                || cmd.trim() == "wq"
                                || cmd.trim() == "x"
                            {
                                self.exit_requested = true;
                            }
                            Ok(())
                        }
                        Err(e) => {
                            self.vim_state.set_message(format!("Error: {}", e));
                            self.vim_state.set_mode(VimMode::Normal);
                            Ok(())
                        }
                    }
                } else if ch == '\x1b' {
                    // ESC - cancel command
                    self.vim_state.set_mode(VimMode::Normal);
                    Ok(())
                } else {
                    // Add to command buffer
                    let mut new_buffer = cmd_buffer.clone();
                    new_buffer.push(ch);
                    self.vim_state
                        .set_mode(VimMode::Command { buffer: new_buffer });
                    Ok(())
                }
            }
        }
    }

    /// Get the screen output
    pub fn get_screen(&self) -> &str {
        &self.screen_buffer
    }
}

/// Vim main loop - handles input and renders screen
pub fn vim_main_loop(vim: &mut Vim, _state: &KernelState) -> Option<SystemCall> {
    // First call: render initial screen
    if !vim.initialized {
        vim.render_screen();
        vim.initialized = true;
        return Some(SystemCall::Write {
            fd: 1,
            data: vim.screen_buffer.as_bytes().to_vec(),
        });
    }

    // Check if exit requested
    if vim.exit_requested {
        return Some(SystemCall::Exit(0));
    }

    // Interactive input handling is managed at the integration layer (wos/src/lib.rs)
    // This MVP implementation renders the initial state and exits
    // Full interactive mode would require stdin blocking and keystroke processing
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

    // Vim tests
    #[test]
    fn test_vim_new_empty() {
        let vim = Vim::new(2, None);
        assert_eq!(vim.pid, 2);
        assert_eq!(vim.file_path, None);
        assert!(!vim.initialized);
        assert!(!vim.exit_requested);
        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);
    }

    #[test]
    fn test_vim_new_with_file() {
        let path = PathBuf::from("/test.txt");
        let vim = Vim::new(2, Some(path.clone()));
        assert_eq!(vim.pid, 2);
        assert_eq!(vim.file_path, Some(path));
    }

    #[test]
    fn test_vim_render_screen_empty() {
        let mut vim = Vim::new(2, None);
        vim.render_screen();

        let screen = vim.get_screen();
        assert!(screen.contains("[No Name]"));
        assert!(screen.contains("NORMAL"));
    }

    #[test]
    fn test_vim_render_screen_with_content() {
        let mut vim = Vim::new(2, None);
        vim.vim_state = crate::vim::VimState::new_with_text("Hello\nWorld");
        vim.render_screen();

        let screen = vim.get_screen();
        assert!(screen.contains("Hello"));
        assert!(screen.contains("World"));
    }

    #[test]
    fn test_vim_process_input_mode_switch() {
        let mut vim = Vim::new(2, None);
        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);

        // Switch to insert mode
        vim.process_input('i').unwrap();
        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Insert);

        // Switch back to normal mode
        vim.process_input('\x1b').unwrap();
        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);
    }

    #[test]
    fn test_vim_process_input_command_mode() {
        let mut vim = Vim::new(2, None);

        // Enter command mode
        vim.process_input(':').unwrap();
        assert!(matches!(
            vim.vim_state.mode,
            crate::vim::VimMode::Command { .. }
        ));

        // Type a command
        vim.process_input('w').unwrap();
        if let crate::vim::VimMode::Command { buffer } = &vim.vim_state.mode {
            assert_eq!(buffer, "w");
        } else {
            panic!("Expected Command mode");
        }
    }

    #[test]
    fn test_vim_process_input_insert_text() {
        let mut vim = Vim::new(2, None);

        // Switch to insert mode
        vim.process_input('i').unwrap();

        // Type some text
        vim.process_input('H').unwrap();
        vim.process_input('i').unwrap();

        let buffer = vim.vim_state.current_buffer();
        assert_eq!(buffer.text(), "Hi");
    }

    #[test]
    fn test_vim_process_input_navigation() {
        let mut vim = Vim::new(2, None);
        vim.vim_state = crate::vim::VimState::new_with_text("Line1\nLine2\nLine3");

        // Move down
        vim.process_input('j').unwrap();
        assert_eq!(vim.vim_state.current_buffer().cursor.line, 1);

        // Move right
        vim.process_input('l').unwrap();
        assert_eq!(vim.vim_state.current_buffer().cursor.col, 1);

        // Move left
        vim.process_input('h').unwrap();
        assert_eq!(vim.vim_state.current_buffer().cursor.col, 0);

        // Move up
        vim.process_input('k').unwrap();
        assert_eq!(vim.vim_state.current_buffer().cursor.line, 0);
    }

    #[test]
    fn test_vim_process_input_quit_command() {
        let mut vim = Vim::new(2, None);

        // Enter quit command
        vim.process_input(':').unwrap();
        vim.process_input('q').unwrap();
        vim.process_input('\n').unwrap();

        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);
        assert!(vim.exit_requested);
    }

    #[test]
    fn test_vim_main_loop_initial_render() {
        let mut vim = Vim::new(2, None);
        let state = KernelState::new();

        let syscall = vim_main_loop(&mut vim, &state);

        // Should write initial screen
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 1, .. })));
        assert!(vim.initialized);
    }

    #[test]
    fn test_vim_main_loop_exit() {
        let mut vim = Vim::new(2, None);
        vim.initialized = true;
        vim.exit_requested = true;
        let state = KernelState::new();

        let syscall = vim_main_loop(&mut vim, &state);
        assert_eq!(syscall, Some(SystemCall::Exit(0)));
    }

    #[test]
    fn test_vim_integration_full_session() {
        let mut vim = Vim::new(2, None);
        let state = KernelState::new();

        // Initial render
        let syscall1 = vim_main_loop(&mut vim, &state);
        assert!(matches!(syscall1, Some(SystemCall::Write { .. })));

        // Simulate user session
        vim.process_input('i').unwrap(); // Enter insert mode
        vim.process_input('H').unwrap();
        vim.process_input('e').unwrap();
        vim.process_input('l').unwrap();
        vim.process_input('l').unwrap();
        vim.process_input('o').unwrap();
        vim.process_input('\x1b').unwrap(); // Exit insert mode

        assert_eq!(vim.vim_state.current_buffer().text(), "Hello");

        // Quit
        vim.process_input(':').unwrap();
        vim.process_input('q').unwrap();
        vim.process_input('!').unwrap();
        vim.process_input('\n').unwrap();

        assert!(vim.exit_requested);
    }

    // Property-based tests (PMAT Protocol: 100 cases per property)
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn proptest_echo_never_panics(args in prop::collection::vec(any::<String>(), 0..20)) {
                let mut echo = Echo::new(1, args);
                echo.generate_output();
                let _output = echo.get_output();
            }

            #[test]
            fn proptest_echo_output_deterministic(args in prop::collection::vec(any::<String>(), 0..20)) {
                let mut echo1 = Echo::new(1, args.clone());
                let mut echo2 = Echo::new(1, args);

                echo1.generate_output();
                echo2.generate_output();

                prop_assert_eq!(echo1.get_output(), echo2.get_output());
            }

            #[test]
            fn proptest_echo_main_loop_never_panics(args in prop::collection::vec(any::<String>(), 0..20)) {
                let mut echo = Echo::new(1, args);
                let state = KernelState::new();

                let _syscall1 = echo_main_loop(&mut echo, &state);
                let _syscall2 = echo_main_loop(&mut echo, &state);
            }
        }
    }

    #[test]
    fn test_ps_process_state_blocked() {
        let mut state = KernelState::new();
        let mut proc1 = Process::new(1, None);
        proc1.state = wos_kernel::ProcessState::Blocked;
        state.add_process(proc1);

        let mut ps = Ps::new(2);
        ps.list_processes(&state);

        assert_eq!(ps.processes.len(), 1);
        assert_eq!(ps.processes[0].state, "S");
    }

    #[test]
    fn test_ps_process_state_terminated() {
        let mut state = KernelState::new();
        let mut proc1 = Process::new(1, None);
        proc1.state = wos_kernel::ProcessState::Terminated(0);
        state.add_process(proc1);

        let mut ps = Ps::new(2);
        ps.list_processes(&state);

        assert_eq!(ps.processes.len(), 1);
        assert_eq!(ps.processes[0].state, "Z");
    }

    #[test]
    fn test_vim_visual_char_mode_escape() {
        let mut vim = Vim::new(2, None);
        vim.vim_state = crate::vim::VimState::new_with_text("Hello\nWorld");

        // Enter visual character mode
        vim.process_input('v').unwrap();
        assert!(matches!(
            vim.vim_state.mode,
            crate::vim::VimMode::Visual(VisualMode::Character)
        ));

        // Press ESC to exit
        vim.process_input('\x1b').unwrap();
        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);
        assert_eq!(vim.vim_state.current_buffer().visual_anchor, None);
    }

    #[test]
    fn test_vim_visual_line_mode_escape() {
        let mut vim = Vim::new(2, None);
        vim.vim_state = crate::vim::VimState::new_with_text("Hello\nWorld");

        // Enter visual line mode
        vim.process_input('V').unwrap();
        assert!(matches!(
            vim.vim_state.mode,
            crate::vim::VimMode::Visual(VisualMode::Line)
        ));

        // Press ESC to exit
        vim.process_input('\x1b').unwrap();
        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);
    }

    #[test]
    fn test_vim_visual_block_mode_escape() {
        let mut vim = Vim::new(2, None);
        vim.vim_state = crate::vim::VimState::new_with_text("Hello\nWorld");

        // Enter visual block mode (Ctrl+v)
        vim.process_input('\x16').unwrap();
        assert!(matches!(
            vim.vim_state.mode,
            crate::vim::VimMode::Visual(VisualMode::Block)
        ));

        // Press ESC to exit
        vim.process_input('\x1b').unwrap();
        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);
    }

    #[test]
    fn test_vim_visual_mode_navigation() {
        let mut vim = Vim::new(2, None);
        vim.vim_state = crate::vim::VimState::new_with_text("Hello\nWorld\nTest");

        // Enter visual mode and navigate
        vim.process_input('v').unwrap();
        vim.process_input('l').unwrap(); // Move right
        vim.process_input('j').unwrap(); // Move down
        vim.process_input('h').unwrap(); // Move left
        vim.process_input('k').unwrap(); // Move up

        assert!(matches!(
            vim.vim_state.mode,
            crate::vim::VimMode::Visual(VisualMode::Character)
        ));
    }

    #[test]
    fn test_vim_visual_mode_unknown_key() {
        let mut vim = Vim::new(2, None);
        vim.vim_state = crate::vim::VimState::new_with_text("Hello");

        // Enter visual mode
        vim.process_input('v').unwrap();

        // Press unknown key (should be ignored)
        vim.process_input('z').unwrap();

        // Should still be in visual mode
        assert!(matches!(
            vim.vim_state.mode,
            crate::vim::VimMode::Visual(VisualMode::Character)
        ));
    }

    #[test]
    fn test_vim_command_mode_escape() {
        let mut vim = Vim::new(2, None);

        // Enter command mode
        vim.process_input(':').unwrap();
        vim.process_input('w').unwrap();

        // Press ESC to cancel
        vim.process_input('\x1b').unwrap();

        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);
        assert!(!vim.exit_requested);
    }

    #[test]
    fn test_vim_command_mode_quit_variations() {
        // Test :wq
        let mut vim = Vim::new(2, None);
        vim.process_input(':').unwrap();
        vim.process_input('w').unwrap();
        vim.process_input('q').unwrap();
        vim.process_input('\n').unwrap();
        assert!(vim.exit_requested);

        // Test :x
        let mut vim = Vim::new(2, None);
        vim.process_input(':').unwrap();
        vim.process_input('x').unwrap();
        vim.process_input('\n').unwrap();
        assert!(vim.exit_requested);

        // Test :q!
        let mut vim = Vim::new(2, None);
        vim.process_input(':').unwrap();
        vim.process_input('q').unwrap();
        vim.process_input('!').unwrap();
        vim.process_input('\n').unwrap();
        assert!(vim.exit_requested);
    }

    #[test]
    fn test_vim_normal_mode_unknown_key() {
        let mut vim = Vim::new(2, None);
        vim.vim_state = crate::vim::VimState::new_with_text("Hello");

        // Press unknown key in normal mode (should be ignored)
        vim.process_input('z').unwrap();
        vim.process_input('q').unwrap();
        vim.process_input('m').unwrap();

        // Should still be in normal mode
        assert_eq!(vim.vim_state.mode, crate::vim::VimMode::Normal);
    }

    #[test]
    fn test_vim_insert_mode_backspace_variations() {
        let mut vim = Vim::new(2, None);

        // Test \x08 (backspace)
        vim.process_input('i').unwrap();
        vim.process_input('H').unwrap();
        vim.process_input('i').unwrap();
        vim.process_input('\x08').unwrap(); // Backspace
        assert_eq!(vim.vim_state.current_buffer().text(), "H");

        // Test \x7f (DEL)
        vim.process_input('\x7f').unwrap(); // DEL
        assert_eq!(vim.vim_state.current_buffer().text(), "");
    }

    #[test]
    fn test_vim_render_screen_with_modified_flag() {
        let mut vim = Vim::new(2, Some(PathBuf::from("/test.txt")));
        vim.vim_state = crate::vim::VimState::new_with_text("Hello");
        vim.vim_state.current_buffer_mut().modified = true;

        vim.render_screen();

        let screen = vim.get_screen();
        assert!(screen.contains("[+]"));
        assert!(screen.contains("/test.txt"));
    }

    #[test]
    fn test_vim_render_screen_with_message() {
        let mut vim = Vim::new(2, None);
        vim.vim_state.set_message("Test message".to_string());

        vim.render_screen();

        let screen = vim.get_screen();
        assert!(screen.contains("Test message"));
    }
}
