//! Shell Process
//!
//! A simple command-line shell for WOS that provides:
//! - Command parsing
//! - Built-in commands (cd, exit, help)
//! - External command execution
//! - Command history
//! - Environment variables

use std::collections::HashMap;
use wos_kernel::{KernelState, ProcessId, SystemCall};

/// Command structure after parsing
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    /// Command name
    pub name: String,
    /// Command arguments
    pub args: Vec<String>,
}

impl Command {
    /// Create a new command
    pub fn new(name: String, args: Vec<String>) -> Self {
        Self { name, args }
    }
}

/// Shell state
#[derive(Clone, Debug, PartialEq)]
pub struct Shell {
    /// Shell process ID
    pub pid: ProcessId,
    /// Current working directory
    pub cwd: String,
    /// Command history
    pub history: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Exit requested
    pub exit_requested: bool,
}

impl Shell {
    /// Create a new shell
    pub fn new(pid: ProcessId) -> Self {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/bin:/usr/bin".to_string());
        env.insert("HOME".to_string(), "/".to_string());
        env.insert("SHELL".to_string(), "/bin/sh".to_string());

        Self {
            pid,
            cwd: "/".to_string(),
            history: Vec::new(),
            env,
            exit_requested: false,
        }
    }

    /// Parse a command line into a Command
    pub fn parse_command(line: &str) -> Option<Command> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let parts: Vec<String> = trimmed.split_whitespace().map(|s| s.to_string()).collect();

        if parts.is_empty() {
            return None;
        }

        let name = parts[0].clone();
        let args = parts[1..].to_vec();

        Some(Command::new(name, args))
    }

    /// Add command to history
    pub fn add_to_history(&mut self, line: String) {
        if !line.trim().is_empty() {
            self.history.push(line);
        }
    }

    /// Get environment variable
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env.get(key)
    }

    /// Set environment variable
    pub fn set_env(&mut self, key: String, value: String) {
        self.env.insert(key, value);
    }

    /// Execute a built-in command
    ///
    /// Returns true if the command was a built-in, false otherwise
    pub fn execute_builtin(&mut self, cmd: &Command) -> bool {
        match cmd.name.as_str() {
            "cd" => {
                self.builtin_cd(cmd);
                true
            }
            "exit" => {
                self.builtin_exit(cmd);
                true
            }
            "help" => {
                self.builtin_help();
                true
            }
            "pwd" => {
                self.builtin_pwd();
                true
            }
            "export" => {
                self.builtin_export(cmd);
                true
            }
            "history" => {
                self.builtin_history();
                true
            }
            _ => false,
        }
    }

    /// Built-in: cd - change directory
    fn builtin_cd(&mut self, cmd: &Command) {
        if cmd.args.is_empty() {
            // cd with no args goes to HOME
            if let Some(home) = self.env.get("HOME") {
                self.cwd = home.clone();
            }
        } else {
            let path = &cmd.args[0];
            let new_path = if path.starts_with('/') {
                // Absolute path
                path.clone()
            } else {
                // Relative path
                format!("{}/{}", self.cwd.trim_end_matches('/'), path)
            };

            // Normalize path (resolve ".." and ".")
            self.cwd = Self::normalize_path(&new_path);
        }
    }

    /// Normalize a path by resolving ".." and "." components
    fn normalize_path(path: &str) -> String {
        let mut parts = Vec::new();

        for part in path.split('/') {
            match part {
                "" | "." => continue, // Skip empty and current directory
                ".." => {
                    // Go up one directory
                    parts.pop();
                }
                _ => parts.push(part),
            }
        }

        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    }

    /// Built-in: exit - exit the shell
    fn builtin_exit(&mut self, _cmd: &Command) {
        self.exit_requested = true;
    }

    /// Built-in: help - show help message
    fn builtin_help(&self) {
        // In a real implementation, this would print to stdout
        // For now, it's a no-op (testing will verify it was called)
    }

    /// Built-in: pwd - print working directory
    fn builtin_pwd(&self) {
        // In a real implementation, this would print to stdout
        // For now, it's a no-op (testing will verify the cwd)
    }

    /// Built-in: export - set environment variable
    fn builtin_export(&mut self, cmd: &Command) {
        for arg in &cmd.args {
            if let Some(pos) = arg.find('=') {
                let key = arg[..pos].to_string();
                let value = arg[pos + 1..].to_string();
                self.env.insert(key, value);
            }
        }
    }

    /// Built-in: history - show command history
    fn builtin_history(&self) {
        // In a real implementation, this would print to stdout
        // For now, it's a no-op (testing will verify the history)
    }

    /// Execute an external command
    ///
    /// Returns the syscall to fork and execute the command
    pub fn execute_external(&self, _cmd: &Command) -> SystemCall {
        // Fork to create a child process for the external command
        SystemCall::Fork
    }

    /// Check if shell should exit
    pub fn should_exit(&self) -> bool {
        self.exit_requested
    }
}

/// Shell main loop logic
///
/// Processes a command line and returns the next syscall to execute
pub fn shell_main_loop(shell: &mut Shell, line: &str, _state: &KernelState) -> Option<SystemCall> {
    // Add to history
    shell.add_to_history(line.to_string());

    // Parse command
    let cmd = Shell::parse_command(line)?;

    // Try built-in commands first
    if shell.execute_builtin(&cmd) {
        // Built-in executed
        if shell.should_exit() {
            // Return Exit syscall
            return Some(SystemCall::Exit(0));
        }
        return None; // Built-in executed, no syscall needed
    }

    // External command - fork to execute
    Some(shell.execute_external(&cmd))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_parse_command() {
        let cmd = Shell::parse_command("ls -la /home").unwrap();
        assert_eq!(cmd.name, "ls");
        assert_eq!(cmd.args, vec!["-la", "/home"]);
    }

    #[test]
    fn test_shell_parse_command_no_args() {
        let cmd = Shell::parse_command("pwd").unwrap();
        assert_eq!(cmd.name, "pwd");
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn test_shell_parse_command_empty() {
        let cmd = Shell::parse_command("   ");
        assert!(cmd.is_none());
    }

    #[test]
    fn test_shell_parse_command_whitespace() {
        let cmd = Shell::parse_command("  ls   -la   ").unwrap();
        assert_eq!(cmd.name, "ls");
        assert_eq!(cmd.args, vec!["-la"]);
    }

    #[test]
    fn test_shell_builtin_cd() {
        let mut shell = Shell::new(2);
        assert_eq!(shell.cwd, "/");

        let cmd = Command::new("cd".to_string(), vec!["/home".to_string()]);
        shell.execute_builtin(&cmd);
        assert_eq!(shell.cwd, "/home");
    }

    #[test]
    fn test_shell_builtin_cd_relative() {
        let mut shell = Shell::new(2);
        shell.cwd = "/home".to_string();

        let cmd = Command::new("cd".to_string(), vec!["user".to_string()]);
        shell.execute_builtin(&cmd);
        assert_eq!(shell.cwd, "/home/user");
    }

    #[test]
    fn test_shell_builtin_cd_home() {
        let mut shell = Shell::new(2);
        shell.cwd = "/tmp".to_string();

        let cmd = Command::new("cd".to_string(), vec![]);
        shell.execute_builtin(&cmd);
        assert_eq!(shell.cwd, "/");
    }

    #[test]
    fn test_shell_builtin_exit() {
        let mut shell = Shell::new(2);
        assert!(!shell.should_exit());

        let cmd = Command::new("exit".to_string(), vec![]);
        shell.execute_builtin(&cmd);
        assert!(shell.should_exit());
    }

    #[test]
    fn test_shell_builtin_help() {
        let mut shell = Shell::new(2);
        let cmd = Command::new("help".to_string(), vec![]);
        let result = shell.execute_builtin(&cmd);
        assert!(result);
    }

    #[test]
    fn test_shell_builtin_pwd() {
        let mut shell = Shell::new(2);
        shell.cwd = "/home/user".to_string();
        let cmd = Command::new("pwd".to_string(), vec![]);
        let result = shell.execute_builtin(&cmd);
        assert!(result);
        // pwd is a builtin, so it should return true
        assert_eq!(shell.cwd, "/home/user");
    }

    #[test]
    fn test_shell_builtin_history() {
        let mut shell = Shell::new(2);
        shell.add_to_history("ls -la".to_string());
        shell.add_to_history("pwd".to_string());

        let cmd = Command::new("history".to_string(), vec![]);
        let result = shell.execute_builtin(&cmd);
        assert!(result);
        // history is a builtin, so it should return true
        // The history should still contain the commands
        assert_eq!(shell.history.len(), 2);
    }

    #[test]
    fn test_shell_exec_external() {
        let shell = Shell::new(2);
        let cmd = Command::new("ls".to_string(), vec![]);
        let syscall = shell.execute_external(&cmd);
        assert_eq!(syscall, SystemCall::Fork);
    }

    #[test]
    fn test_shell_history() {
        let mut shell = Shell::new(2);
        assert_eq!(shell.history.len(), 0);

        shell.add_to_history("ls -la".to_string());
        shell.add_to_history("pwd".to_string());

        assert_eq!(shell.history.len(), 2);
        assert_eq!(shell.history[0], "ls -la");
        assert_eq!(shell.history[1], "pwd");
    }

    #[test]
    fn test_shell_history_ignores_empty() {
        let mut shell = Shell::new(2);
        shell.add_to_history("".to_string());
        shell.add_to_history("   ".to_string());

        assert_eq!(shell.history.len(), 0);
    }

    #[test]
    fn test_shell_env_vars() {
        let mut shell = Shell::new(2);

        // Check default env vars
        assert_eq!(shell.get_env("PATH"), Some(&"/bin:/usr/bin".to_string()));
        assert_eq!(shell.get_env("HOME"), Some(&"/".to_string()));

        // Set new env var
        shell.set_env("USER".to_string(), "root".to_string());
        assert_eq!(shell.get_env("USER"), Some(&"root".to_string()));
    }

    #[test]
    fn test_shell_builtin_export() {
        let mut shell = Shell::new(2);

        let cmd = Command::new(
            "export".to_string(),
            vec!["FOO=bar".to_string(), "BAZ=qux".to_string()],
        );
        shell.execute_builtin(&cmd);

        assert_eq!(shell.get_env("FOO"), Some(&"bar".to_string()));
        assert_eq!(shell.get_env("BAZ"), Some(&"qux".to_string()));
    }

    #[test]
    fn test_shell_main_loop_builtin() {
        let mut shell = Shell::new(2);
        let state = KernelState::new();

        let syscall = shell_main_loop(&mut shell, "cd /home", &state);
        assert_eq!(syscall, None); // Built-in doesn't need syscall
        assert_eq!(shell.cwd, "/home");
        assert_eq!(shell.history.len(), 1);
    }

    #[test]
    fn test_shell_main_loop_exit() {
        let mut shell = Shell::new(2);
        let state = KernelState::new();

        let syscall = shell_main_loop(&mut shell, "exit", &state);
        assert_eq!(syscall, Some(SystemCall::Exit(0)));
        assert!(shell.should_exit());
    }

    #[test]
    fn test_shell_main_loop_external() {
        let mut shell = Shell::new(2);
        let state = KernelState::new();

        let syscall = shell_main_loop(&mut shell, "ls -la", &state);
        assert_eq!(syscall, Some(SystemCall::Fork));
        assert_eq!(shell.history.len(), 1);
    }

    #[test]
    fn test_shell_main_loop_empty() {
        let mut shell = Shell::new(2);
        let state = KernelState::new();

        let syscall = shell_main_loop(&mut shell, "   ", &state);
        assert_eq!(syscall, None);
        assert_eq!(shell.history.len(), 0); // Empty lines not added to history
    }

    // WOS-400: Tests for path normalization
    #[test]
    fn test_normalize_path_parent_directory() {
        assert_eq!(Shell::normalize_path("/home/user/.."), "/home");
        assert_eq!(Shell::normalize_path("/home/user/../.."), "/");
        assert_eq!(Shell::normalize_path("/a/b/c/../.."), "/a");
    }

    #[test]
    fn test_normalize_path_current_directory() {
        assert_eq!(Shell::normalize_path("/home/./user"), "/home/user");
        assert_eq!(Shell::normalize_path("/./home/./user/."), "/home/user");
    }

    #[test]
    fn test_normalize_path_complex() {
        assert_eq!(
            Shell::normalize_path("/home/user/../other/./file"),
            "/home/other/file"
        );
        assert_eq!(Shell::normalize_path("/a/./b/../c"), "/a/c");
    }

    #[test]
    fn test_normalize_path_root() {
        assert_eq!(Shell::normalize_path("/"), "/");
        assert_eq!(Shell::normalize_path("/.."), "/");
        assert_eq!(Shell::normalize_path("/../.."), "/");
    }

    #[test]
    fn test_normalize_path_empty_components() {
        assert_eq!(Shell::normalize_path("//home///user//"), "/home/user");
    }

    #[test]
    fn test_shell_cd_with_parent_directory() {
        let mut shell = Shell::new(2);
        shell.cwd = "/home/user".to_string();

        let cmd = Command::new("cd".to_string(), vec!["..".to_string()]);
        shell.execute_builtin(&cmd);
        assert_eq!(shell.cwd, "/home");
    }

    #[test]
    fn test_shell_cd_with_current_directory() {
        let mut shell = Shell::new(2);
        shell.cwd = "/home".to_string();

        let cmd = Command::new("cd".to_string(), vec![".".to_string()]);
        shell.execute_builtin(&cmd);
        assert_eq!(shell.cwd, "/home");
    }
}
