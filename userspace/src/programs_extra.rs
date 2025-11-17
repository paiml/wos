//! Additional User Programs
//!
//! Extended set of user-level programs:
//! - cat: concatenate and display files (WOS-PROG-001)
//! - grep: search for patterns in text (WOS-PROG-002)
//! - wc: count lines, words, and bytes (WOS-PROG-006)
//! - head: display first lines of a file (WOS-PROG-007)
//! - tail: display last lines of a file (WOS-PROG-008)

use std::path::PathBuf;
use wos_kernel::{KernelState, ProcessId, SystemCall};

// ============================================================================
// Additional User Programs (WOS-PROG-001 through WOS-PROG-008)
// ============================================================================

/// Cat program - concatenate and display files
/// WOS-PROG-001: cat command
#[derive(Clone, Debug, PartialEq)]
pub struct Cat {
    /// Process ID
    pub pid: ProcessId,
    /// File paths to concatenate
    pub files: Vec<PathBuf>,
    /// File contents concatenated
    pub output: String,
    /// Current file index being processed
    pub current_file: usize,
}

impl Cat {
    /// Create a new cat program
    pub fn new(pid: ProcessId, files: Vec<PathBuf>) -> Self {
        Self {
            pid,
            files,
            output: String::new(),
            current_file: 0,
        }
    }

    /// Read files from VFS and concatenate
    pub fn read_files(&mut self, state: &mut KernelState) {
        for file_path in &self.files {
            if let Ok(content) = state.vfs.read_file(file_path) {
                if let Ok(text) = String::from_utf8(content) {
                    self.output.push_str(&text);
                }
            }
        }
    }

    /// Get the output
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Cat main loop - reads and concatenates files
pub fn cat_main_loop(cat: &mut Cat, state: &mut KernelState) -> Option<SystemCall> {
    if cat.output.is_empty() && cat.current_file == 0 {
        // First iteration: read all files
        cat.read_files(state);
        cat.current_file = cat.files.len();

        if !cat.output.is_empty() {
            return Some(SystemCall::Write {
                fd: 1,
                data: cat.output.as_bytes().to_vec(),
            });
        }
    }

    // Done - exit
    Some(SystemCall::Exit(0))
}

/// Grep program - search for patterns in text
/// WOS-PROG-002: grep command
#[derive(Clone, Debug, PartialEq)]
pub struct Grep {
    /// Process ID
    pub pid: ProcessId,
    /// Pattern to search for
    pub pattern: String,
    /// Files to search (empty = stdin)
    pub files: Vec<PathBuf>,
    /// Matching lines
    pub matches: Vec<String>,
    /// Output generated
    pub output: String,
}

impl Grep {
    /// Create a new grep program
    pub fn new(pid: ProcessId, pattern: String, files: Vec<PathBuf>) -> Self {
        Self {
            pid,
            pattern,
            files,
            matches: Vec::new(),
            output: String::new(),
        }
    }

    /// Search files for pattern
    pub fn search_files(&mut self, state: &mut KernelState) {
        for file_path in &self.files {
            if let Ok(content) = state.vfs.read_file(file_path) {
                if let Ok(text) = String::from_utf8(content) {
                    for line in text.lines() {
                        if line.contains(&self.pattern) {
                            self.matches.push(line.to_string());
                        }
                    }
                }
            }
        }
    }

    /// Generate output from matches
    pub fn generate_output(&mut self) {
        if !self.matches.is_empty() {
            self.output = self.matches.join("\n");
            self.output.push('\n');
        }
    }

    /// Get the output
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Grep main loop - searches for pattern and outputs matches
pub fn grep_main_loop(grep: &mut Grep, state: &mut KernelState) -> Option<SystemCall> {
    if grep.matches.is_empty() && grep.output.is_empty() {
        // First iteration: search files
        grep.search_files(state);
        grep.generate_output();

        if !grep.output.is_empty() {
            return Some(SystemCall::Write {
                fd: 1,
                data: grep.output.as_bytes().to_vec(),
            });
        }
    }

    // Done - exit
    Some(SystemCall::Exit(0))
}

/// Wc program - count lines, words, and bytes
/// WOS-PROG-006: wc command
#[derive(Clone, Debug, PartialEq)]
pub struct Wc {
    /// Process ID
    pub pid: ProcessId,
    /// Files to count (empty = stdin)
    pub files: Vec<PathBuf>,
    /// Line count
    pub lines: usize,
    /// Word count
    pub words: usize,
    /// Byte count
    pub bytes: usize,
    /// Output generated
    pub output: String,
}

impl Wc {
    /// Create a new wc program
    pub fn new(pid: ProcessId, files: Vec<PathBuf>) -> Self {
        Self {
            pid,
            files,
            lines: 0,
            words: 0,
            bytes: 0,
            output: String::new(),
        }
    }

    /// Count lines, words, and bytes in files
    pub fn count_files(&mut self, state: &mut KernelState) {
        for file_path in &self.files {
            if let Ok(content) = state.vfs.read_file(file_path) {
                self.bytes += content.len();

                if let Ok(text) = String::from_utf8(content) {
                    self.lines += text.lines().count();
                    self.words += text.split_whitespace().count();
                }
            }
        }
    }

    /// Generate output
    pub fn generate_output(&mut self) {
        self.output = format!("{:8} {:8} {:8}\n", self.lines, self.words, self.bytes);
    }

    /// Get the output
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Wc main loop - counts and outputs statistics
pub fn wc_main_loop(wc: &mut Wc, state: &mut KernelState) -> Option<SystemCall> {
    if wc.output.is_empty() {
        // First iteration: count files
        wc.count_files(state);
        wc.generate_output();

        return Some(SystemCall::Write {
            fd: 1,
            data: wc.output.as_bytes().to_vec(),
        });
    }

    // Done - exit
    Some(SystemCall::Exit(0))
}

/// Head program - display first N lines of a file
/// WOS-PROG-007: head command
#[derive(Clone, Debug, PartialEq)]
pub struct Head {
    /// Process ID
    pub pid: ProcessId,
    /// File to display
    pub file: PathBuf,
    /// Number of lines to show (default 10)
    pub num_lines: usize,
    /// Output generated
    pub output: String,
}

impl Head {
    /// Create a new head program
    pub fn new(pid: ProcessId, file: PathBuf, num_lines: Option<usize>) -> Self {
        Self {
            pid,
            file,
            num_lines: num_lines.unwrap_or(10),
            output: String::new(),
        }
    }

    /// Read and output first N lines
    pub fn read_file(&mut self, state: &mut KernelState) {
        if let Ok(content) = state.vfs.read_file(&self.file) {
            if let Ok(text) = String::from_utf8(content) {
                let lines: Vec<&str> = text.lines().take(self.num_lines).collect();
                if !lines.is_empty() {
                    self.output = lines.join("\n");
                    self.output.push('\n');
                }
            }
        }
    }

    /// Get the output
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Head main loop - displays first lines of file
pub fn head_main_loop(head: &mut Head, state: &mut KernelState) -> Option<SystemCall> {
    if head.output.is_empty() {
        // First iteration: read file
        head.read_file(state);

        if !head.output.is_empty() {
            return Some(SystemCall::Write {
                fd: 1,
                data: head.output.as_bytes().to_vec(),
            });
        }
    }

    // Done - exit
    Some(SystemCall::Exit(0))
}

/// Tail program - display last N lines of a file
/// WOS-PROG-008: tail command
#[derive(Clone, Debug, PartialEq)]
pub struct Tail {
    /// Process ID
    pub pid: ProcessId,
    /// File to display
    pub file: PathBuf,
    /// Number of lines to show (default 10)
    pub num_lines: usize,
    /// Output generated
    pub output: String,
}

impl Tail {
    /// Create a new tail program
    pub fn new(pid: ProcessId, file: PathBuf, num_lines: Option<usize>) -> Self {
        Self {
            pid,
            file,
            num_lines: num_lines.unwrap_or(10),
            output: String::new(),
        }
    }

    /// Read and output last N lines
    pub fn read_file(&mut self, state: &mut KernelState) {
        if let Ok(content) = state.vfs.read_file(&self.file) {
            if let Ok(text) = String::from_utf8(content) {
                let all_lines: Vec<&str> = text.lines().collect();
                let start = if all_lines.len() > self.num_lines {
                    all_lines.len() - self.num_lines
                } else {
                    0
                };

                let lines = &all_lines[start..];
                if !lines.is_empty() {
                    self.output = lines.join("\n");
                    self.output.push('\n');
                }
            }
        }
    }

    /// Get the output
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Tail main loop - displays last lines of file
pub fn tail_main_loop(tail: &mut Tail, state: &mut KernelState) -> Option<SystemCall> {
    if tail.output.is_empty() {
        // First iteration: read file
        tail.read_file(state);

        if !tail.output.is_empty() {
            return Some(SystemCall::Write {
                fd: 1,
                data: tail.output.as_bytes().to_vec(),
            });
        }
    }

    // Done - exit
    Some(SystemCall::Exit(0))
}
