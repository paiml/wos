//! WOS - WASM Operating System
//!
//! Main entry point integrating kernel and userspace for WASM execution.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod quality;

pub use quality::{BuildStatus, QualityMetrics};
use wasm_bindgen::prelude::*;
use wos_kernel::{dispatch_syscall, KernelState, SystemCall};

/// Get WOS version
#[wasm_bindgen]
pub fn wos_version() -> String {
    format!(
        "WOS v{} (kernel: {}, userspace: {})",
        env!("CARGO_PKG_VERSION"),
        wos_kernel::kernel_version(),
        wos_userspace::userspace_version()
    )
}

/// WASM-bindgen wrapper for WOS kernel
#[wasm_bindgen]
pub struct WosWasm {
    state: KernelState,
}

impl Default for WosWasm {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WosWasm {
    /// Create a new WOS instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            state: KernelState::with_init(),
        }
    }

    /// Execute a syscall and return the output as JSON
    ///
    /// Takes a syscall as JSON string, executes it, and returns the output as JSON
    #[wasm_bindgen(js_name = executeSyscall)]
    pub fn execute_syscall(
        &mut self,
        syscall_json: &str,
        calling_pid: u32,
    ) -> Result<String, String> {
        // Parse syscall from JSON
        let syscall: SystemCall = serde_json::from_str(syscall_json)
            .map_err(|e| format!("Failed to parse syscall: {}", e))?;

        // Execute syscall
        let result = dispatch_syscall(self.state.clone(), syscall, calling_pid);

        match result {
            Ok((new_state, output)) => {
                self.state = new_state;
                serde_json::to_string(&output)
                    .map_err(|e| format!("Failed to serialize output: {}", e))
            }
            Err(e) => Err(format!("Syscall error: {:?}", e)),
        }
    }

    /// Execute a command string (shell-like interface)
    ///
    /// Parses a command and executes it, returning the output
    #[wasm_bindgen(js_name = executeCommand)]
    pub fn execute_command(&mut self, command: &str) -> String {
        let command = command.trim();
        if command.is_empty() {
            return String::new();
        }

        // Parse command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd_name = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        // Built-in commands
        match cmd_name {
            "help" => self.cmd_help(),
            "ps" => self.cmd_ps(args),
            "ls" => self.cmd_ls(args),
            "cat" => self.cmd_cat(args),
            "pwd" => self.cmd_pwd(),
            "touch" => self.cmd_touch(args),
            "mkdir" => self.cmd_mkdir(args),
            "rm" => self.cmd_rm(args),
            "echo" => self.cmd_echo(args),
            "grep" => self.cmd_grep(args),
            "wc" => self.cmd_wc(args),
            "version" => wos_version(),
            "state" => self.cmd_state(),
            "reset" => {
                self.reset();
                "System reset complete".to_string()
            }
            _ => format!(
                "Unknown command: {}\nType 'help' for available commands",
                cmd_name
            ),
        }
    }

    // Helper methods for command execution
    fn cmd_help(&self) -> String {
        let mut output = String::from("Available commands:\n");
        output.push_str("  help      - Show this help message\n");
        output.push_str("  ps        - List processes\n");
        output.push_str("  ls        - List files\n");
        output.push_str("  cat       - Display file contents\n");
        output.push_str("  pwd       - Print working directory\n");
        output.push_str("  touch     - Create file\n");
        output.push_str("  mkdir     - Create directory\n");
        output.push_str("  rm        - Remove file\n");
        output.push_str("  echo      - Echo arguments\n");
        output.push_str("  grep      - Search file contents\n");
        output.push_str("  wc        - Count words/lines/bytes\n");
        output.push_str("  version   - Show system version\n");
        output.push_str("  state     - Show kernel state\n");
        output.push_str("  reset     - Reset system to initial state\n");
        output
    }

    fn cmd_ps(&self, _args: Vec<String>) -> String {
        let mut output = String::from("PID\tSTATE\t\t\tPARENT\n");
        output.push_str("---\t-----\t\t\t------\n");

        for (pid, process) in &self.state.processes {
            let parent = process
                .parent_pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string());

            let state_str = format!("{:?}", process.state);

            output.push_str(&format!("{}\t{:16}\t{}\n", pid, state_str, parent));
        }

        if self.state.processes.is_empty() {
            output.push_str("No processes running\n");
        }

        output
    }

    fn cmd_ls(&self, _args: Vec<String>) -> String {
        // List all files from VFS
        let files = self.state.vfs.list_files();

        if files.is_empty() {
            String::new()
        } else {
            files
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }
    }

    fn cmd_echo(&self, args: Vec<String>) -> String {
        args.join(" ")
    }

    fn cmd_state(&self) -> String {
        let proc_count = self.state.processes.len();

        // Count total memory pages across all processes
        let mut total_mem_pages = 0;
        for process in self.state.processes.values() {
            total_mem_pages += process.memory_pages.len();
        }

        let mut output = String::from("Kernel State:\n");
        output.push_str(&format!("  Processes: {}\n", proc_count));
        output.push_str(&format!("  Total Memory Pages: {}\n", total_mem_pages));
        output.push_str(&format!("  Next PID: {}\n", self.state.next_pid));
        output.push_str(&format!("  Current PID: {:?}\n", self.state.current_pid));

        output
    }

    fn cmd_cat(&self, args: Vec<String>) -> String {
        if args.is_empty() {
            return "cat: missing file operand\n".to_string();
        }

        let path = std::path::PathBuf::from(&args[0]);
        match self.state.vfs.read_file(&path) {
            Ok(contents) => String::from_utf8_lossy(&contents).to_string(),
            Err(_) => format!("cat: {}: No such file or directory\n", args[0]),
        }
    }

    fn cmd_pwd(&self) -> String {
        // For now, always return /
        // Future: track current working directory per process
        "/\n".to_string()
    }

    fn cmd_touch(&mut self, args: Vec<String>) -> String {
        if args.is_empty() {
            return "touch: missing file operand\n".to_string();
        }

        let path = std::path::PathBuf::from(&args[0]);
        match self.state.vfs.create_file(path, vec![]) {
            Ok(()) => String::new(),
            Err(_) => format!("touch: cannot create file '{}'\n", args[0]),
        }
    }

    fn cmd_mkdir(&mut self, args: Vec<String>) -> String {
        if args.is_empty() {
            return "mkdir: missing operand\n".to_string();
        }

        // VFS doesn't have explicit directory support yet
        // For now, just create a marker file
        let path = std::path::PathBuf::from(&format!("{}/.directory", args[0]));
        match self.state.vfs.create_file(path, vec![]) {
            Ok(()) => String::new(),
            Err(_) => format!("mkdir: cannot create directory '{}'\n", args[0]),
        }
    }

    fn cmd_rm(&mut self, args: Vec<String>) -> String {
        if args.is_empty() {
            return "rm: missing operand\n".to_string();
        }

        let path = std::path::PathBuf::from(&args[0]);
        match self.state.vfs.delete_file(&path) {
            Ok(()) => String::new(),
            Err(_) => format!(
                "rm: cannot remove '{}': No such file or directory\n",
                args[0]
            ),
        }
    }

    fn cmd_grep(&self, args: Vec<String>) -> String {
        if args.len() < 2 {
            return "grep: missing pattern or file\n".to_string();
        }

        let pattern = &args[0];
        let path = std::path::PathBuf::from(&args[1]);

        match self.state.vfs.read_file(&path) {
            Ok(contents) => {
                let text = String::from_utf8_lossy(&contents);
                let mut output = String::new();
                for line in text.lines() {
                    if line.contains(pattern) {
                        output.push_str(line);
                        output.push('\n');
                    }
                }
                output
            }
            Err(_) => format!("grep: {}: No such file or directory\n", args[1]),
        }
    }

    fn cmd_wc(&self, args: Vec<String>) -> String {
        if args.is_empty() {
            return "wc: missing file operand\n".to_string();
        }

        let path = std::path::PathBuf::from(&args[0]);
        match self.state.vfs.read_file(&path) {
            Ok(contents) => {
                let text = String::from_utf8_lossy(&contents);
                let lines = text.lines().count();
                let words = text.split_whitespace().count();
                let bytes = contents.len();
                format!("  {}  {}  {} {}\n", lines, words, bytes, args[0])
            }
            Err(_) => format!("wc: {}: No such file or directory\n", args[0]),
        }
    }

    /// Get current kernel state as JSON
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&self) -> Result<String, String> {
        serde_json::to_string(&self.state).map_err(|e| format!("Failed to serialize state: {}", e))
    }

    /// Set kernel state from JSON
    #[wasm_bindgen(js_name = setState)]
    pub fn set_state(&mut self, state_json: &str) -> Result<(), String> {
        let state: KernelState = serde_json::from_str(state_json)
            .map_err(|e| format!("Failed to parse state: {}", e))?;
        self.state = state;
        Ok(())
    }

    /// Get number of processes
    #[wasm_bindgen(js_name = processCount)]
    pub fn process_count(&self) -> usize {
        self.state.processes.len()
    }

    /// Reset to initial state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.state = KernelState::with_init();
    }

    /// Get quality metrics as JSON
    #[wasm_bindgen(js_name = getQualityMetrics)]
    pub fn get_quality_metrics(&self) -> Result<String, String> {
        let metrics = QualityMetrics::new();
        metrics.to_json()
    }

    /// Export quality report as HTML
    #[wasm_bindgen(js_name = exportQualityHtml)]
    pub fn export_quality_html(&self) -> String {
        let metrics = QualityMetrics::new();
        metrics.to_html()
    }

    /// Export quality report as Markdown
    #[wasm_bindgen(js_name = exportQualityMarkdown)]
    pub fn export_quality_markdown(&self) -> String {
        let metrics = QualityMetrics::new();
        metrics.to_markdown()
    }

    /// Export quality report as SARIF
    #[wasm_bindgen(js_name = exportQualitySarif)]
    pub fn export_quality_sarif(&self) -> String {
        let metrics = QualityMetrics::new();
        metrics.to_sarif()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wos_kernel::Process;

    #[test]
    fn test_wos_version() {
        let version = wos_version();
        assert!(version.starts_with("WOS v"));
    }

    #[test]
    fn test_wos_wasm_new() {
        let wos = WosWasm::new();
        // Should start with init and shell processes
        assert_eq!(wos.state.processes.len(), 2);
    }

    #[test]
    fn test_wos_wasm_process_count() {
        let mut wos = WosWasm::new();
        // Should start with init and shell processes
        assert_eq!(wos.process_count(), 2);

        // Add another process to state
        let mut state = wos.state.clone();
        let proc = Process::new(3, Some(2));
        state.add_process(proc);
        wos.state = state;

        assert_eq!(wos.process_count(), 3);
    }

    #[test]
    fn test_wos_wasm_reset() {
        let mut wos = WosWasm::new();

        // Add another process
        let mut state = wos.state.clone();
        let proc = Process::new(3, Some(2));
        state.add_process(proc);
        wos.state = state;

        assert_eq!(wos.process_count(), 3);

        // Reset - should return to init and shell
        wos.reset();
        assert_eq!(wos.process_count(), 2);
    }

    #[test]
    fn test_wos_wasm_get_state() {
        let wos = WosWasm::new();
        let state_json = wos.get_state().unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&state_json).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_wos_wasm_set_state() {
        let mut wos = WosWasm::new();

        // Add a process to state
        let mut state = KernelState::new();
        let proc = Process::new(1, None);
        state.add_process(proc);

        // Serialize state
        let state_json = serde_json::to_string(&state).unwrap();

        // Set state
        wos.set_state(&state_json).unwrap();
        assert_eq!(wos.process_count(), 1);
    }

    #[test]
    fn test_wos_wasm_state_roundtrip() {
        let mut wos = WosWasm::new();

        // Add another process (starts with 2: init and shell)
        let mut state = wos.state.clone();
        let proc = Process::new(3, Some(2));
        state.add_process(proc);
        wos.state = state;

        // Get state
        let state_json = wos.get_state().unwrap();

        // Create new instance and set state
        let mut wos2 = WosWasm::new();
        wos2.set_state(&state_json).unwrap();

        assert_eq!(wos2.process_count(), 3);
    }

    #[test]
    fn test_wos_wasm_execute_syscall_getpid() {
        let mut wos = WosWasm::new();

        // Add a process first
        let mut state = wos.state.clone();
        let proc = Process::new(1, None);
        state.add_process(proc);
        wos.state = state;

        // Execute GetPid syscall
        let syscall = SystemCall::GetPid;
        let syscall_json = serde_json::to_string(&syscall).unwrap();
        let result = wos.execute_syscall(&syscall_json, 1);

        // Should succeed
        assert!(result.is_ok());
        // Should return valid JSON
        let json_str = result.unwrap();
        assert!(!json_str.is_empty());
    }

    #[test]
    fn test_wos_wasm_execute_syscall_succeeds() {
        // Test that syscall execution works end-to-end
        let mut wos = WosWasm::new();

        // Add a process
        let proc = Process::new(1, None);
        wos.state.add_process(proc);

        // Execute GetPid syscall (simple, no state mutation)
        let syscall = SystemCall::GetPid;
        let syscall_json = serde_json::to_string(&syscall).unwrap();
        let result = wos.execute_syscall(&syscall_json, 1);

        // Should succeed and return non-empty JSON
        assert!(result.is_ok());
        let output_json = result.unwrap();
        assert!(!output_json.is_empty());
        // Verify it's valid JSON
        let _parsed: serde_json::Value = serde_json::from_str(&output_json).unwrap();
    }

    #[test]
    fn test_wos_wasm_execute_syscall_invalid_json() {
        let mut wos = WosWasm::new();

        let syscall_json = "invalid json";
        let result = wos.execute_syscall(syscall_json, 1);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse syscall"));
    }

    #[test]
    fn test_wos_wasm_set_state_invalid_json() {
        let mut wos = WosWasm::new();

        let result = wos.set_state("invalid json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse state"));
    }

    #[test]
    fn test_wos_wasm_execute_command_echo() {
        let mut wos = WosWasm::new();
        let result = wos.execute_command("echo hello world");

        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_wos_wasm_execute_command_help() {
        let mut wos = WosWasm::new();
        let result = wos.execute_command("help");

        assert!(result.contains("Available commands"));
        assert!(result.contains("help"));
        assert!(result.contains("ps"));
        assert!(result.contains("ls"));
        assert!(result.contains("echo"));
    }

    #[test]
    fn test_wos_wasm_execute_command_version() {
        let mut wos = WosWasm::new();
        let result = wos.execute_command("version");

        assert!(result.starts_with("WOS v"));
    }

    #[test]
    fn test_wos_wasm_execute_command_ps_with_init() {
        let mut wos = WosWasm::new();
        let result = wos.execute_command("ps");

        assert!(result.contains("PID"));
        // Should show init and shell processes
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(!result.contains("No processes running"));
    }

    #[test]
    fn test_wos_wasm_execute_command_ps_with_processes() {
        let mut wos = WosWasm::new();

        // Add a process
        let proc = Process::new(1, None);
        wos.state.add_process(proc);

        let result = wos.execute_command("ps");

        assert!(result.contains("PID"));
        assert!(result.contains("1"));
        assert!(!result.contains("No processes running"));
    }

    #[test]
    fn test_wos_wasm_execute_command_ls_empty() {
        let mut wos = WosWasm::new();
        let result = wos.execute_command("ls");

        assert_eq!(result, "");
    }

    #[test]
    fn test_wos_wasm_execute_command_ls_with_files() {
        use std::path::PathBuf;
        let mut wos = WosWasm::new();

        // Add files to VFS
        wos.state
            .vfs
            .create_file(PathBuf::from("/test.txt"), vec![])
            .unwrap();
        wos.state
            .vfs
            .create_file(PathBuf::from("/another.txt"), vec![])
            .unwrap();

        let result = wos.execute_command("ls");

        assert!(result.contains("/test.txt"));
        assert!(result.contains("/another.txt"));
    }

    #[test]
    fn test_wos_wasm_execute_command_state() {
        let mut wos = WosWasm::new();

        // Add another process (starts with 2: init and shell)
        let proc = Process::new(3, Some(2));
        wos.state.add_process(proc);

        let result = wos.execute_command("state");

        assert!(result.contains("Kernel State"));
        assert!(result.contains("Processes: 3"));
        assert!(result.contains("Next PID"));
    }

    #[test]
    fn test_wos_wasm_execute_command_reset() {
        let mut wos = WosWasm::new();

        // Add another process (starts with 2: init and shell)
        let proc = Process::new(3, Some(2));
        wos.state.add_process(proc);

        assert_eq!(wos.process_count(), 3);

        let result = wos.execute_command("reset");

        assert!(result.contains("reset complete"));
        assert_eq!(wos.process_count(), 2);
    }

    #[test]
    fn test_wos_wasm_execute_command_unknown() {
        let mut wos = WosWasm::new();
        let result = wos.execute_command("unknown_command");

        assert!(result.contains("Unknown command"));
        assert!(result.contains("unknown_command"));
    }

    #[test]
    fn test_wos_wasm_execute_command_empty() {
        let mut wos = WosWasm::new();
        let result = wos.execute_command("");

        assert_eq!(result, "");
    }

    #[test]
    fn test_wos_wasm_execute_command_whitespace() {
        let mut wos = WosWasm::new();
        let result = wos.execute_command("   ");

        assert_eq!(result, "");
    }

    #[test]
    fn test_wos_wasm_get_quality_metrics() {
        let wos = WosWasm::new();
        let metrics_json = wos.get_quality_metrics().unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&metrics_json).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("tdg_grade").is_some());
        assert!(parsed.get("test_count").is_some());
    }

    #[test]
    fn test_wos_wasm_export_quality_html() {
        let wos = WosWasm::new();
        let html = wos.export_quality_html();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("WOS Quality Report"));
        assert!(html.contains("TDG Grade"));
    }

    #[test]
    fn test_wos_wasm_export_quality_markdown() {
        let wos = WosWasm::new();
        let md = wos.export_quality_markdown();

        assert!(md.contains("# WOS Quality Report"));
        assert!(md.contains("TDG Grade"));
        assert!(md.contains("Test Coverage"));
    }

    #[test]
    fn test_wos_wasm_export_quality_sarif() {
        let wos = WosWasm::new();
        let sarif = wos.export_quality_sarif();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        assert!(parsed.is_object());

        // Check SARIF structure
        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["runs"].is_array());
    }
}
