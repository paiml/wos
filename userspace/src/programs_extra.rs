//! Additional User Programs
//!
//! Extended set of user-level programs:
//! - cat: concatenate and display files (WOS-PROG-001)
//! - grep: search for patterns in text (WOS-PROG-002)
//! - wc: count lines, words, and bytes (WOS-PROG-006)
//! - head: display first lines of a file (WOS-PROG-007)
//! - tail: display last lines of a file (WOS-PROG-008)
//! - cp: copy files and directories (WOS-PROG-017)
//! - mv: move or rename files (WOS-PROG-018)
//! - mkdir: create directories (WOS-PROG-020)
//! - rm: remove files and directories (WOS-PROG-019)
//! - chmod: change file permissions (WOS-PROG-014)
//! - find: search directory trees for files (WOS-PROG-005)
//! - sort: sort lines of text (WOS-PROG-009)
//! - uniq: remove duplicate lines (WOS-PROG-010)

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

/// Cp program - copy files and directories
/// WOS-PROG-017: cp command
#[derive(Clone, Debug, PartialEq)]
pub struct Cp {
    /// Process ID
    pub pid: ProcessId,
    /// Source path
    pub src: PathBuf,
    /// Destination path
    pub dst: PathBuf,
    /// Recursive copy (-r flag)
    pub recursive: bool,
    /// Preserve permissions/timestamps (-p flag)
    pub preserve: bool,
    /// Interactive mode (-i flag)
    pub interactive: bool,
    /// Operation completed
    pub completed: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl Cp {
    /// Create a new cp program
    pub fn new(
        pid: ProcessId,
        src: PathBuf,
        dst: PathBuf,
        recursive: bool,
        preserve: bool,
        interactive: bool,
    ) -> Self {
        Self {
            pid,
            src,
            dst,
            recursive,
            preserve,
            interactive,
            completed: false,
            error: None,
        }
    }

    /// Copy a file from src to dst
    pub fn copy_file(&mut self, state: &mut KernelState) {
        // Read source file
        let content = match state.vfs.read_file(&self.src) {
            Ok(data) => data,
            Err(e) => {
                self.error = Some(format!("cannot read '{}': {:?}", self.src.display(), e));
                return;
            }
        };

        // Write to destination
        if let Err(e) = state.vfs.create_file(self.dst.clone(), content) {
            self.error = Some(format!("cannot create '{}': {:?}", self.dst.display(), e));
            return;
        }

        // Preserve permissions if -p flag
        if self.preserve {
            if let Ok(perms) = state.vfs.get_permissions(&self.src) {
                let _ = state.vfs.set_permissions(&self.dst, perms);
            }
        }

        self.completed = true;
    }
}

/// Cp main loop - copies files
pub fn cp_main_loop(cp: &mut Cp, state: &mut KernelState) -> Option<SystemCall> {
    if !cp.completed && cp.error.is_none() {
        // First iteration: copy file
        cp.copy_file(state);

        if let Some(ref error) = cp.error {
            return Some(SystemCall::Write {
                fd: 2, // stderr
                data: format!("cp: {}\n", error).as_bytes().to_vec(),
            });
        }
    }

    // Done - exit with appropriate code
    let exit_code = if cp.error.is_some() { 1 } else { 0 };
    Some(SystemCall::Exit(exit_code))
}

/// Mv program - move or rename files
/// WOS-PROG-018: mv command
#[derive(Clone, Debug, PartialEq)]
pub struct Mv {
    /// Process ID
    pub pid: ProcessId,
    /// Source path
    pub src: PathBuf,
    /// Destination path
    pub dst: PathBuf,
    /// Interactive mode (-i flag)
    pub interactive: bool,
    /// Operation completed
    pub completed: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl Mv {
    /// Create a new mv program
    pub fn new(pid: ProcessId, src: PathBuf, dst: PathBuf, interactive: bool) -> Self {
        Self {
            pid,
            src,
            dst,
            interactive,
            completed: false,
            error: None,
        }
    }

    /// Move a file from src to dst
    pub fn move_file(&mut self, state: &mut KernelState) {
        // Read source file
        let content = match state.vfs.read_file(&self.src) {
            Ok(data) => data,
            Err(e) => {
                self.error = Some(format!("cannot read '{}': {:?}", self.src.display(), e));
                return;
            }
        };

        // Get permissions to preserve them
        let perms = match state.vfs.get_permissions(&self.src) {
            Ok(p) => Some(p),
            Err(_) => None,
        };

        // Write to destination
        if let Err(e) = state.vfs.create_file(self.dst.clone(), content) {
            self.error = Some(format!("cannot create '{}': {:?}", self.dst.display(), e));
            return;
        }

        // Restore permissions
        if let Some(perms) = perms {
            let _ = state.vfs.set_permissions(&self.dst, perms);
        }

        // Delete source file
        if let Err(e) = state.vfs.delete_file(&self.src) {
            self.error = Some(format!("cannot remove '{}': {:?}", self.src.display(), e));
            return;
        }

        self.completed = true;
    }
}

/// Mv main loop - moves/renames files
pub fn mv_main_loop(mv: &mut Mv, state: &mut KernelState) -> Option<SystemCall> {
    if !mv.completed && mv.error.is_none() {
        // First iteration: move file
        mv.move_file(state);

        if let Some(ref error) = mv.error {
            return Some(SystemCall::Write {
                fd: 2, // stderr
                data: format!("mv: {}\n", error).as_bytes().to_vec(),
            });
        }
    }

    // Done - exit with appropriate code
    let exit_code = if mv.error.is_some() { 1 } else { 0 };
    Some(SystemCall::Exit(exit_code))
}

/// Mkdir program - create directories
/// WOS-PROG-020: mkdir command
#[derive(Clone, Debug, PartialEq)]
pub struct Mkdir {
    /// Process ID
    pub pid: ProcessId,
    /// Directory path to create
    pub path: PathBuf,
    /// Create parent directories (-p flag)
    pub parents: bool,
    /// Set permissions (-m flag)
    pub mode: Option<u32>,
    /// Operation completed
    pub completed: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl Mkdir {
    /// Create a new mkdir program
    pub fn new(pid: ProcessId, path: PathBuf, parents: bool, mode: Option<u32>) -> Self {
        Self {
            pid,
            path,
            parents,
            mode,
            completed: false,
            error: None,
        }
    }

    /// Create the directory
    pub fn create_dir(&mut self, state: &mut KernelState) {
        if self.parents {
            // Create parent directories recursively
            self.create_parents(state);
            // Return early if there was an error
            if self.error.is_some() {
                return;
            }
        } else {
            // Create single directory
            if let Err(e) = state.vfs.create_directory(self.path.clone()) {
                self.error = Some(format!(
                    "cannot create directory '{}': {:?}",
                    self.path.display(),
                    e
                ));
                return;
            }
        }

        // Set permissions if -m flag provided
        if let Some(mode) = self.mode {
            if let Err(e) = state.vfs.chmod(&self.path, mode) {
                self.error = Some(format!("cannot set permissions: {:?}", e));
                return;
            }
        }

        self.completed = true;
    }

    /// Create parent directories recursively for -p flag
    fn create_parents(&mut self, state: &mut KernelState) {
        let path_str = match self.path.to_str() {
            Some(s) => s,
            None => {
                self.error = Some("invalid path".to_string());
                return;
            }
        };

        // Split path into components
        let components: Vec<&str> = path_str
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        // Create each directory in sequence
        let mut current_path = PathBuf::from("/");
        for component in components {
            current_path.push(component);

            // Skip if already exists
            if state.vfs.exists(&current_path) {
                if !state.vfs.is_directory(&current_path) {
                    self.error = Some(format!(
                        "'{}' exists but is not a directory",
                        current_path.display()
                    ));
                    return;
                }
                continue;
            }

            // Create directory
            if let Err(e) = state.vfs.create_directory(current_path.clone()) {
                self.error = Some(format!(
                    "cannot create directory '{}': {:?}",
                    current_path.display(),
                    e
                ));
                return;
            }
        }
    }
}

/// Mkdir main loop - creates directories
pub fn mkdir_main_loop(mkdir: &mut Mkdir, state: &mut KernelState) -> Option<SystemCall> {
    if !mkdir.completed && mkdir.error.is_none() {
        // First iteration: create directory
        mkdir.create_dir(state);

        if let Some(ref error) = mkdir.error {
            return Some(SystemCall::Write {
                fd: 2, // stderr
                data: format!("mkdir: {}\n", error).as_bytes().to_vec(),
            });
        }
    }

    // Done - exit with appropriate code
    let exit_code = if mkdir.error.is_some() { 1 } else { 0 };
    Some(SystemCall::Exit(exit_code))
}

/// Rm program - remove files and directories
/// WOS-PROG-019: rm command
#[derive(Clone, Debug, PartialEq)]
pub struct Rm {
    /// Process ID
    pub pid: ProcessId,
    /// Paths to remove
    pub paths: Vec<PathBuf>,
    /// Recursive removal (-r flag)
    pub recursive: bool,
    /// Force removal, ignore errors (-f flag)
    pub force: bool,
    /// Interactive confirmation (-i flag)
    pub interactive: bool,
    /// Operation completed
    pub completed: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl Rm {
    /// Create a new rm program
    pub fn new(
        pid: ProcessId,
        paths: Vec<PathBuf>,
        recursive: bool,
        force: bool,
        interactive: bool,
    ) -> Self {
        Self {
            pid,
            paths,
            recursive,
            force,
            interactive,
            completed: false,
            error: None,
        }
    }

    /// Remove files and directories
    pub fn remove(&mut self, state: &mut KernelState) {
        let paths = self.paths.clone();
        for path in &paths {
            if !state.vfs.exists(path) {
                if !self.force {
                    self.error = Some(format!(
                        "cannot remove '{}': No such file or directory",
                        path.display()
                    ));
                    return;
                }
                continue;
            }

            if state.vfs.is_directory(path) {
                if !self.recursive {
                    self.error = Some(format!(
                        "cannot remove '{}': Is a directory",
                        path.display()
                    ));
                    return;
                }

                // Remove directory recursively
                if let Err(e) = self.remove_recursive(state, path) {
                    if !self.force {
                        self.error = Some(e);
                        return;
                    }
                }
            } else {
                // Remove file
                if let Err(e) = state.vfs.delete_file(path) {
                    if !self.force {
                        self.error = Some(format!("cannot remove '{}': {:?}", path.display(), e));
                        return;
                    }
                }
            }
        }

        self.completed = true;
    }

    /// Remove directory recursively
    fn remove_recursive(&mut self, state: &mut KernelState, path: &PathBuf) -> Result<(), String> {
        // List directory contents
        let entries = state
            .vfs
            .list_directory(path)
            .map_err(|e| format!("cannot list directory '{}': {:?}", path.display(), e))?;

        // Remove all entries first
        for entry in &entries {
            let entry_path = path.join(&entry.name);

            if entry.is_directory() {
                // Recursively remove subdirectory
                self.remove_recursive(state, &entry_path)?;
            } else {
                // Remove file
                state
                    .vfs
                    .delete_file(&entry_path)
                    .map_err(|e| format!("cannot remove '{}': {:?}", entry_path.display(), e))?;
            }
        }

        // Now remove the empty directory
        state
            .vfs
            .remove_directory(path)
            .map_err(|e| format!("cannot remove directory '{}': {:?}", path.display(), e))?;

        Ok(())
    }
}

/// Rm main loop - removes files and directories
pub fn rm_main_loop(rm: &mut Rm, state: &mut KernelState) -> Option<SystemCall> {
    if !rm.completed && rm.error.is_none() {
        // First iteration: remove files/directories
        rm.remove(state);

        if let Some(ref error) = rm.error {
            return Some(SystemCall::Write {
                fd: 2, // stderr
                data: format!("rm: {}\n", error).as_bytes().to_vec(),
            });
        }
    }

    // Done - exit with appropriate code
    let exit_code = if rm.error.is_some() { 1 } else { 0 };
    Some(SystemCall::Exit(exit_code))
}

/// Chmod program - change file permissions
/// WOS-PROG-014: chmod command
#[derive(Clone, Debug, PartialEq)]
pub struct Chmod {
    /// Process ID
    pub pid: ProcessId,
    /// Permission mode (numeric, e.g., 0o755)
    pub mode: u32,
    /// Files to change permissions
    pub paths: Vec<PathBuf>,
    /// Recursive (-R flag)
    pub recursive: bool,
    /// Operation completed
    pub completed: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl Chmod {
    /// Create a new chmod program with numeric mode
    pub fn new(pid: ProcessId, mode: u32, paths: Vec<PathBuf>, recursive: bool) -> Self {
        Self {
            pid,
            mode,
            paths,
            recursive,
            completed: false,
            error: None,
        }
    }

    /// Change permissions for files
    pub fn chmod(&mut self, state: &mut KernelState) {
        let paths = self.paths.clone();
        for path in &paths {
            if !state.vfs.exists(path) {
                self.error = Some(format!(
                    "cannot access '{}': No such file or directory",
                    path.display()
                ));
                return;
            }

            if self.recursive && state.vfs.is_directory(path) {
                // Recursively chmod directory
                if let Err(e) = self.chmod_recursive(state, path) {
                    self.error = Some(e);
                    return;
                }
            } else {
                // Change permissions for single file/directory
                if let Err(e) = state.vfs.chmod(path, self.mode) {
                    self.error = Some(format!("cannot chmod '{}': {:?}", path.display(), e));
                    return;
                }
            }
        }

        self.completed = true;
    }

    /// Recursively change permissions for directory and contents
    fn chmod_recursive(&mut self, state: &mut KernelState, path: &PathBuf) -> Result<(), String> {
        // Change permissions for the directory itself
        state
            .vfs
            .chmod(path, self.mode)
            .map_err(|e| format!("cannot chmod '{}': {:?}", path.display(), e))?;

        // List directory contents
        let entries = state
            .vfs
            .list_directory(path)
            .map_err(|e| format!("cannot list directory '{}': {:?}", path.display(), e))?;

        // Recursively chmod all entries
        for entry in &entries {
            let entry_path = path.join(&entry.name);

            if entry.is_directory() {
                // Recursively chmod subdirectory
                self.chmod_recursive(state, &entry_path)?;
            } else {
                // Change permissions for file
                state
                    .vfs
                    .chmod(&entry_path, self.mode)
                    .map_err(|e| format!("cannot chmod '{}': {:?}", entry_path.display(), e))?;
            }
        }

        Ok(())
    }
}

/// Chmod main loop - changes file permissions
pub fn chmod_main_loop(chmod: &mut Chmod, state: &mut KernelState) -> Option<SystemCall> {
    if !chmod.completed && chmod.error.is_none() {
        // First iteration: change permissions
        chmod.chmod(state);

        if let Some(ref error) = chmod.error {
            return Some(SystemCall::Write {
                fd: 2, // stderr
                data: format!("chmod: {}\n", error).as_bytes().to_vec(),
            });
        }
    }

    // Done - exit with appropriate code
    let exit_code = if chmod.error.is_some() { 1 } else { 0 };
    Some(SystemCall::Exit(exit_code))
}

/// Find program - search directory trees for files
/// WOS-PROG-005: find command
#[derive(Clone, Debug, PartialEq)]
pub struct Find {
    /// Process ID
    pub pid: ProcessId,
    /// Starting path
    pub path: PathBuf,
    /// Name pattern to match (simple string matching)
    pub name_pattern: Option<String>,
    /// Type filter: 'f' for file, 'd' for directory
    pub type_filter: Option<char>,
    /// Matching paths found
    pub matches: Vec<PathBuf>,
    /// Output generated
    pub output: String,
    /// Operation completed
    pub completed: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl Find {
    /// Create a new find program
    pub fn new(
        pid: ProcessId,
        path: PathBuf,
        name_pattern: Option<String>,
        type_filter: Option<char>,
    ) -> Self {
        Self {
            pid,
            path,
            name_pattern,
            type_filter,
            matches: Vec::new(),
            output: String::new(),
            completed: false,
            error: None,
        }
    }

    /// Search directory tree for matching files
    pub fn search(&mut self, state: &mut KernelState) {
        if !state.vfs.exists(&self.path) {
            self.error = Some(format!(
                "'{}': No such file or directory",
                self.path.display()
            ));
            return;
        }

        // Perform recursive search
        if let Err(e) = self.search_recursive(state, &self.path.clone()) {
            self.error = Some(e);
            return;
        }

        // Generate output from matches
        if !self.matches.is_empty() {
            self.output = self
                .matches
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n");
            self.output.push('\n');
        }

        self.completed = true;
    }

    /// Recursively search directory
    fn search_recursive(&mut self, state: &mut KernelState, path: &PathBuf) -> Result<(), String> {
        // Check if current path matches criteria
        if self.matches_criteria(state, path) {
            self.matches.push(path.clone());
        }

        // If path is a directory, recurse into it
        if state.vfs.is_directory(path) {
            let entries = state
                .vfs
                .list_directory(path)
                .map_err(|e| format!("cannot list '{}': {:?}", path.display(), e))?;

            for entry in &entries {
                let entry_path = path.join(&entry.name);
                self.search_recursive(state, &entry_path)?;
            }
        }

        Ok(())
    }

    /// Check if path matches search criteria
    fn matches_criteria(&self, state: &KernelState, path: &PathBuf) -> bool {
        // Type filter
        if let Some(type_filter) = self.type_filter {
            let is_dir = state.vfs.is_directory(path);
            match type_filter {
                'd' => {
                    if !is_dir {
                        return false;
                    }
                }
                'f' => {
                    if is_dir {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        // Name pattern filter (simple substring matching)
        if let Some(ref pattern) = self.name_pattern {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if !name_str.contains(pattern) {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Find main loop - searches for files and outputs matches
pub fn find_main_loop(find: &mut Find, state: &mut KernelState) -> Option<SystemCall> {
    if !find.completed && find.error.is_none() {
        // First iteration: search
        find.search(state);

        if let Some(ref error) = find.error {
            return Some(SystemCall::Write {
                fd: 2, // stderr
                data: format!("find: {}\n", error).as_bytes().to_vec(),
            });
        }

        if !find.output.is_empty() {
            return Some(SystemCall::Write {
                fd: 1,
                data: find.output.as_bytes().to_vec(),
            });
        }
    }

    // Done - exit with appropriate code
    let exit_code = if find.error.is_some() { 1 } else { 0 };
    Some(SystemCall::Exit(exit_code))
}

/// Sort program - sort lines of text
/// WOS-PROG-009: sort command
#[derive(Clone, Debug, PartialEq)]
pub struct Sort {
    /// Process ID
    pub pid: ProcessId,
    /// File path to sort (empty = stdin)
    pub file: Option<PathBuf>,
    /// Numeric sort (-n flag)
    pub numeric: bool,
    /// Reverse order (-r flag)
    pub reverse: bool,
    /// Remove duplicates (-u flag)
    pub unique: bool,
    /// Sorted output
    pub output: String,
    /// Operation completed
    pub completed: bool,
}

impl Sort {
    /// Create a new sort program
    pub fn new(
        pid: ProcessId,
        file: Option<PathBuf>,
        numeric: bool,
        reverse: bool,
        unique: bool,
    ) -> Self {
        Self {
            pid,
            file,
            numeric,
            reverse,
            unique,
            completed: false,
            output: String::new(),
        }
    }

    /// Sort lines from file
    pub fn sort_lines(&mut self, state: &mut KernelState) -> Result<(), String> {
        // Read file content
        let content = if let Some(ref file) = self.file {
            state
                .vfs
                .read_file(file)
                .map_err(|e| format!("cannot read '{}': {:?}", file.display(), e))?
        } else {
            return Err("stdin not yet supported".to_string());
        };

        let text = String::from_utf8(content)
            .map_err(|_| "invalid UTF-8 in input".to_string())?;

        // Split into lines
        let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();

        // Sort lines
        if self.numeric {
            // Numeric sort: try to parse as numbers
            lines.sort_by(|a, b| {
                let a_num: Result<i64, _> = a.trim().parse();
                let b_num: Result<i64, _> = b.trim().parse();

                match (a_num, b_num) {
                    (Ok(a_val), Ok(b_val)) => a_val.cmp(&b_val),
                    (Ok(_), Err(_)) => std::cmp::Ordering::Less,
                    (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
                    (Err(_), Err(_)) => a.cmp(b),
                }
            });
        } else {
            // Alphabetic sort
            lines.sort();
        }

        // Reverse if requested
        if self.reverse {
            lines.reverse();
        }

        // Remove duplicates if requested
        if self.unique {
            lines.dedup();
        }

        // Generate output
        if !lines.is_empty() {
            self.output = lines.join("\n");
            self.output.push('\n');
        }

        self.completed = true;
        Ok(())
    }
}

/// Sort main loop - sorts lines
pub fn sort_main_loop(sort: &mut Sort, state: &mut KernelState) -> Option<SystemCall> {
    if !sort.completed {
        if let Err(e) = sort.sort_lines(state) {
            return Some(SystemCall::Write {
                fd: 2, // stderr
                data: format!("sort: {}\n", e).as_bytes().to_vec(),
            });
        }

        if !sort.output.is_empty() {
            return Some(SystemCall::Write {
                fd: 1,
                data: sort.output.as_bytes().to_vec(),
            });
        }
    }

    Some(SystemCall::Exit(0))
}

/// Uniq program - remove duplicate lines
/// WOS-PROG-010: uniq command
#[derive(Clone, Debug, PartialEq)]
pub struct Uniq {
    /// Process ID
    pub pid: ProcessId,
    /// File path (empty = stdin)
    pub file: Option<PathBuf>,
    /// Count occurrences (-c flag)
    pub count: bool,
    /// Show only duplicates (-d flag)
    pub only_duplicates: bool,
    /// Show only unique (-u flag)
    pub only_unique: bool,
    /// Output
    pub output: String,
    /// Operation completed
    pub completed: bool,
}

impl Uniq {
    /// Create a new uniq program
    pub fn new(
        pid: ProcessId,
        file: Option<PathBuf>,
        count: bool,
        only_duplicates: bool,
        only_unique: bool,
    ) -> Self {
        Self {
            pid,
            file,
            count,
            only_duplicates,
            only_unique,
            completed: false,
            output: String::new(),
        }
    }

    /// Remove duplicate lines
    pub fn process(&mut self, state: &mut KernelState) -> Result<(), String> {
        // Read file content
        let content = if let Some(ref file) = self.file {
            state
                .vfs
                .read_file(file)
                .map_err(|e| format!("cannot read '{}': {:?}", file.display(), e))?
        } else {
            return Err("stdin not yet supported".to_string());
        };

        let text = String::from_utf8(content)
            .map_err(|_| "invalid UTF-8 in input".to_string())?;

        // Process lines
        let lines: Vec<&str> = text.lines().collect();
        let mut result = Vec::new();

        let mut i = 0;
        while i < lines.len() {
            let current = lines[i];
            let mut count_val = 1;

            // Count consecutive duplicates
            while i + count_val < lines.len() && lines[i + count_val] == current {
                count_val += 1;
            }

            // Apply filters
            let is_duplicate = count_val > 1;
            let should_include = if self.only_duplicates {
                is_duplicate
            } else if self.only_unique {
                !is_duplicate
            } else {
                true
            };

            if should_include {
                if self.count {
                    result.push(format!("{:7} {}", count_val, current));
                } else {
                    result.push(current.to_string());
                }
            }

            i += count_val;
        }

        // Generate output
        if !result.is_empty() {
            self.output = result.join("\n");
            self.output.push('\n');
        }

        self.completed = true;
        Ok(())
    }
}

/// Uniq main loop - removes duplicates
pub fn uniq_main_loop(uniq: &mut Uniq, state: &mut KernelState) -> Option<SystemCall> {
    if !uniq.completed {
        if let Err(e) = uniq.process(state) {
            return Some(SystemCall::Write {
                fd: 2, // stderr
                data: format!("uniq: {}\n", e).as_bytes().to_vec(),
            });
        }

        if !uniq.output.is_empty() {
            return Some(SystemCall::Write {
                fd: 1,
                data: uniq.output.as_bytes().to_vec(),
            });
        }
    }

    Some(SystemCall::Exit(0))
}

// ============================================================================
// Cut - Extract Columns (WOS-PROG-011)
// ============================================================================

/// Cut command for extracting fields, characters, or bytes from lines
#[derive(Clone, Debug, PartialEq)]
pub struct Cut {
    /// Process ID
    pub pid: ProcessId,
    /// Input file (None for stdin)
    pub file: Option<PathBuf>,
    /// Field numbers to extract (1-indexed)
    pub fields: Option<Vec<usize>>,
    /// Character positions to extract (1-indexed)
    pub characters: Option<Vec<usize>>,
    /// Byte positions to extract (1-indexed)
    pub bytes: Option<Vec<usize>>,
    /// Field delimiter (default: tab)
    pub delimiter: char,
    /// Output buffer
    pub output: String,
    /// Completed flag
    pub completed: bool,
    /// Error message
    pub error: Option<String>,
}

impl Cut {
    /// Create a new Cut instance
    pub fn new(pid: ProcessId, file: Option<PathBuf>) -> Self {
        Self {
            pid,
            file,
            fields: None,
            characters: None,
            bytes: None,
            delimiter: '\t',
            output: String::new(),
            completed: false,
            error: None,
        }
    }

    /// Set field extraction mode
    pub fn with_fields(mut self, fields: Vec<usize>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Set character extraction mode
    pub fn with_characters(mut self, characters: Vec<usize>) -> Self {
        self.characters = Some(characters);
        self
    }

    /// Set byte extraction mode
    pub fn with_bytes(mut self, bytes: Vec<usize>) -> Self {
        self.bytes = Some(bytes);
        self
    }

    /// Set field delimiter
    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Process the input
    pub fn process(&mut self, state: &mut KernelState) -> Result<(), String> {
        if self.completed {
            return Ok(());
        }

        // Read input
        let content = if let Some(ref file_path) = self.file {
            state
                .vfs
                .read_file(file_path)
                .map_err(|e| format!("cannot read '{}': {:?}", file_path.display(), e))?
        } else {
            return Err("stdin not implemented".to_string());
        };

        let text = String::from_utf8(content)
            .map_err(|e| format!("invalid UTF-8: {}", e))?;

        let mut result = Vec::new();

        for line in text.lines() {
            if let Some(ref field_nums) = self.fields {
                // Field extraction mode
                let fields: Vec<&str> = line.split(self.delimiter).collect();
                let mut selected = Vec::new();
                for &field_num in field_nums {
                    if field_num > 0 && field_num <= fields.len() {
                        selected.push(fields[field_num - 1]);
                    }
                }
                result.push(selected.join(&self.delimiter.to_string()));
            } else if let Some(ref char_nums) = self.characters {
                // Character extraction mode
                let chars: Vec<char> = line.chars().collect();
                let mut selected = Vec::new();
                for &char_num in char_nums {
                    if char_num > 0 && char_num <= chars.len() {
                        selected.push(chars[char_num - 1]);
                    }
                }
                result.push(selected.into_iter().collect::<String>());
            } else if let Some(ref byte_nums) = self.bytes {
                // Byte extraction mode
                let bytes = line.as_bytes();
                let mut selected = Vec::new();
                for &byte_num in byte_nums {
                    if byte_num > 0 && byte_num <= bytes.len() {
                        selected.push(bytes[byte_num - 1]);
                    }
                }
                result.push(String::from_utf8_lossy(&selected).to_string());
            } else {
                return Err("must specify -f, -c, or -b".to_string());
            }
        }

        self.output = result.join("\n");
        if !self.output.is_empty() && !text.is_empty() {
            self.output.push('\n');
        }
        self.completed = true;
        Ok(())
    }
}

/// Main loop for cut command
pub fn cut_main_loop(cut: &mut Cut, state: &mut KernelState) -> Option<SystemCall> {
    if !cut.completed && cut.error.is_none() {
        if let Err(e) = cut.process(state) {
            cut.error = Some(e.clone());
            return Some(SystemCall::Write {
                fd: 2,
                data: format!("cut: {}\n", e).as_bytes().to_vec(),
            });
        }
    }

    if cut.completed && !cut.output.is_empty() {
        return Some(SystemCall::Write {
            fd: 1,
            data: cut.output.as_bytes().to_vec(),
        });
    }

    Some(SystemCall::Exit(0))
}

// ============================================================================
// Diff - Compare Files (WOS-PROG-012)
// ============================================================================

/// Diff command for comparing two files
#[derive(Clone, Debug, PartialEq)]
pub struct Diff {
    /// Process ID
    pub pid: ProcessId,
    /// First file path
    pub file1: PathBuf,
    /// Second file path
    pub file2: PathBuf,
    /// Unified diff format
    pub unified: bool,
    /// Context lines (for unified diff)
    pub context: usize,
    /// Brief mode (report only if files differ)
    pub brief: bool,
    /// Ignore whitespace
    pub ignore_whitespace: bool,
    /// Output buffer
    pub output: String,
    /// Completed flag
    pub completed: bool,
    /// Error message
    pub error: Option<String>,
}

impl Diff {
    /// Create a new Diff instance
    pub fn new(pid: ProcessId, file1: PathBuf, file2: PathBuf) -> Self {
        Self {
            pid,
            file1,
            file2,
            unified: false,
            context: 3,
            brief: false,
            ignore_whitespace: false,
            output: String::new(),
            completed: false,
            error: None,
        }
    }

    /// Set unified diff format
    pub fn with_unified(mut self, context: usize) -> Self {
        self.unified = true;
        self.context = context;
        self
    }

    /// Set brief mode
    pub fn with_brief(mut self) -> Self {
        self.brief = true;
        self
    }

    /// Set ignore whitespace mode
    pub fn with_ignore_whitespace(mut self) -> Self {
        self.ignore_whitespace = true;
        self
    }

    /// Process the diff
    pub fn process(&mut self, state: &mut KernelState) -> Result<(), String> {
        if self.completed {
            return Ok(());
        }

        // Read both files
        let content1 = state
            .vfs
            .read_file(&self.file1)
            .map_err(|e| format!("cannot read '{}': {:?}", self.file1.display(), e))?;

        let content2 = state
            .vfs
            .read_file(&self.file2)
            .map_err(|e| format!("cannot read '{}': {:?}", self.file2.display(), e))?;

        let text1 = String::from_utf8(content1)
            .map_err(|e| format!("invalid UTF-8 in {}: {}", self.file1.display(), e))?;

        let text2 = String::from_utf8(content2)
            .map_err(|e| format!("invalid UTF-8 in {}: {}", self.file2.display(), e))?;

        let lines1: Vec<&str> = text1.lines().collect();
        let lines2: Vec<&str> = text2.lines().collect();

        // Check if files are identical
        let identical = if self.ignore_whitespace {
            lines1
                .iter()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                == lines2
                    .iter()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
        } else {
            lines1 == lines2
        };

        if identical {
            self.completed = true;
            return Ok(());
        }

        if self.brief {
            self.output = format!(
                "Files {} and {} differ\n",
                self.file1.display(),
                self.file2.display()
            );
            self.completed = true;
            return Ok(());
        }

        // Compute diff using Myers algorithm (simplified version)
        let diff_result = self.compute_diff(&lines1, &lines2);

        if self.unified {
            self.output = self.format_unified(&diff_result, &lines1, &lines2);
        } else {
            self.output = self.format_normal(&diff_result, &lines1, &lines2);
        }

        self.completed = true;
        Ok(())
    }

    /// Compute diff using a simple LCS-based algorithm
    fn compute_diff<'a>(&self, lines1: &[&'a str], lines2: &[&'a str]) -> Vec<DiffOp> {
        let mut ops = Vec::new();
        let mut i = 0;
        let mut j = 0;

        while i < lines1.len() || j < lines2.len() {
            if i < lines1.len() && j < lines2.len() {
                let line1 = if self.ignore_whitespace {
                    lines1[i].trim()
                } else {
                    lines1[i]
                };
                let line2 = if self.ignore_whitespace {
                    lines2[j].trim()
                } else {
                    lines2[j]
                };

                if line1 == line2 {
                    ops.push(DiffOp::Equal(i, j));
                    i += 1;
                    j += 1;
                } else {
                    // Look ahead to find matching lines
                    let mut found_match = false;
                    for k in 1..=5 {
                        if i + k < lines1.len() && j < lines2.len() {
                            let future1 = if self.ignore_whitespace {
                                lines1[i + k].trim()
                            } else {
                                lines1[i + k]
                            };
                            if future1 == line2 {
                                // Lines were deleted from file1
                                for _ in 0..k {
                                    ops.push(DiffOp::Delete(i));
                                    i += 1;
                                }
                                found_match = true;
                                break;
                            }
                        }
                        if j + k < lines2.len() && i < lines1.len() {
                            let future2 = if self.ignore_whitespace {
                                lines2[j + k].trim()
                            } else {
                                lines2[j + k]
                            };
                            if line1 == future2 {
                                // Lines were added to file2
                                for _ in 0..k {
                                    ops.push(DiffOp::Insert(j));
                                    j += 1;
                                }
                                found_match = true;
                                break;
                            }
                        }
                    }

                    if !found_match {
                        // Assume it's a change
                        ops.push(DiffOp::Change(i, j));
                        i += 1;
                        j += 1;
                    }
                }
            } else if i < lines1.len() {
                ops.push(DiffOp::Delete(i));
                i += 1;
            } else {
                ops.push(DiffOp::Insert(j));
                j += 1;
            }
        }

        ops
    }

    /// Format diff in normal format
    fn format_normal(&self, ops: &[DiffOp], lines1: &[&str], lines2: &[&str]) -> String {
        let mut result = String::new();
        let mut i = 0;

        while i < ops.len() {
            match ops[i] {
                DiffOp::Equal(_, _) => {
                    i += 1;
                }
                DiffOp::Delete(line_num) => {
                    let mut del_count = 1;
                    while i + del_count < ops.len() {
                        if let DiffOp::Delete(_) = ops[i + del_count] {
                            del_count += 1;
                        } else {
                            break;
                        }
                    }
                    result.push_str(&format!("{}d{}\n", line_num + 1, line_num + del_count));
                    for k in 0..del_count {
                        result.push_str(&format!("< {}\n", lines1[line_num + k]));
                    }
                    i += del_count;
                }
                DiffOp::Insert(line_num) => {
                    let mut ins_count = 1;
                    while i + ins_count < ops.len() {
                        if let DiffOp::Insert(_) = ops[i + ins_count] {
                            ins_count += 1;
                        } else {
                            break;
                        }
                    }
                    result.push_str(&format!("{}a{}\n", line_num, line_num + ins_count));
                    for k in 0..ins_count {
                        result.push_str(&format!("> {}\n", lines2[line_num + k]));
                    }
                    i += ins_count;
                }
                DiffOp::Change(line1, line2) => {
                    result.push_str(&format!("{}c{}\n", line1 + 1, line2 + 1));
                    result.push_str(&format!("< {}\n", lines1[line1]));
                    result.push_str("---\n");
                    result.push_str(&format!("> {}\n", lines2[line2]));
                    i += 1;
                }
            }
        }

        result
    }

    /// Format diff in unified format
    fn format_unified(&self, ops: &[DiffOp], lines1: &[&str], lines2: &[&str]) -> String {
        let mut result = String::new();

        result.push_str(&format!("--- {}\n", self.file1.display()));
        result.push_str(&format!("+++ {}\n", self.file2.display()));

        // For simplicity, show all changes in one hunk
        let mut hunk = String::new();
        let mut hunk_line1 = 1;
        let mut hunk_line2 = 1;

        for op in ops {
            match op {
                DiffOp::Equal(i, _) => {
                    hunk.push_str(&format!(" {}\n", lines1[*i]));
                }
                DiffOp::Delete(i) => {
                    hunk.push_str(&format!("-{}\n", lines1[*i]));
                }
                DiffOp::Insert(j) => {
                    hunk.push_str(&format!("+{}\n", lines2[*j]));
                }
                DiffOp::Change(i, j) => {
                    hunk.push_str(&format!("-{}\n", lines1[*i]));
                    hunk.push_str(&format!("+{}\n", lines2[*j]));
                }
            }
        }

        if !hunk.is_empty() {
            result.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk_line1,
                lines1.len(),
                hunk_line2,
                lines2.len()
            ));
            result.push_str(&hunk);
        }

        result
    }
}

/// Diff operation types
#[derive(Clone, Debug, PartialEq)]
enum DiffOp {
    Equal(usize, usize),
    Delete(usize),
    Insert(usize),
    Change(usize, usize),
}

/// Main loop for diff command
pub fn diff_main_loop(diff: &mut Diff, state: &mut KernelState) -> Option<SystemCall> {
    if !diff.completed && diff.error.is_none() {
        if let Err(e) = diff.process(state) {
            diff.error = Some(e.clone());
            return Some(SystemCall::Write {
                fd: 2,
                data: format!("diff: {}\n", e).as_bytes().to_vec(),
            });
        }
    }

    if diff.completed && !diff.output.is_empty() {
        return Some(SystemCall::Write {
            fd: 1,
            data: diff.output.as_bytes().to_vec(),
        });
    }

    let exit_code = if diff.error.is_some() {
        2
    } else if !diff.output.is_empty() {
        1 // Files differ
    } else {
        0 // Files are identical
    };

    Some(SystemCall::Exit(exit_code))
}

// ============================================================================
// Sed - Stream Editor (WOS-PROG-003)
// ============================================================================

/// Sed command for stream editing
#[derive(Clone, Debug, PartialEq)]
pub struct Sed {
    /// Process ID
    pub pid: ProcessId,
    /// Input file (None for stdin)
    pub file: Option<PathBuf>,
    /// Sed script (commands to execute)
    pub script: String,
    /// In-place editing
    pub in_place: bool,
    /// Suppress automatic printing (-n flag)
    pub quiet: bool,
    /// Output buffer
    pub output: String,
    /// Completed flag
    pub completed: bool,
    /// Error message
    pub error: Option<String>,
}

impl Sed {
    /// Create a new Sed instance
    pub fn new(pid: ProcessId, script: String, file: Option<PathBuf>) -> Self {
        Self {
            pid,
            file,
            script,
            in_place: false,
            quiet: false,
            output: String::new(),
            completed: false,
            error: None,
        }
    }

    /// Set quiet mode (suppress automatic printing)
    pub fn with_quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Set in-place editing mode
    pub fn with_in_place(mut self) -> Self {
        self.in_place = true;
        self
    }

    /// Process the input
    pub fn process(&mut self, state: &mut KernelState) -> Result<(), String> {
        if self.completed {
            return Ok(());
        }

        // Read input
        let content = if let Some(ref file_path) = self.file {
            state
                .vfs
                .read_file(file_path)
                .map_err(|e| format!("cannot read '{}': {:?}", file_path.display(), e))?
        } else {
            return Err("stdin not implemented".to_string());
        };

        let text = String::from_utf8(content)
            .map_err(|e| format!("invalid UTF-8: {}", e))?;

        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();

        // Parse sed command
        let commands = self.parse_script(&self.script)?;

        // Execute sed commands
        let mut result_lines = Vec::new();
        let mut line_num = 1;

        for line in &lines {
            let mut current_line = line.clone();
            let mut should_print = !self.quiet;
            let mut should_delete = false;

            for cmd in &commands {
                if !self.matches_address(line_num, lines.len(), &cmd.address) {
                    continue;
                }

                match &cmd.command {
                    SedCommand::Substitute {
                        pattern,
                        replacement,
                        global,
                    } => {
                        current_line = self.substitute(&current_line, pattern, replacement, *global);
                    }
                    SedCommand::Delete => {
                        should_delete = true;
                        should_print = false;
                        break;
                    }
                    SedCommand::Print => {
                        result_lines.push(current_line.clone());
                    }
                }
            }

            if !should_delete && should_print {
                result_lines.push(current_line);
            }

            line_num += 1;
        }

        self.output = result_lines.join("\n");
        if !self.output.is_empty() && !text.is_empty() {
            self.output.push('\n');
        }

        // Handle in-place editing
        if self.in_place {
            if let Some(ref file_path) = self.file {
                // Delete old file and create new one with modified content
                state
                    .vfs
                    .delete_file(file_path)
                    .map_err(|e| format!("cannot delete '{}': {:?}", file_path.display(), e))?;
                state
                    .vfs
                    .create_file(file_path.clone(), self.output.as_bytes().to_vec())
                    .map_err(|e| format!("cannot write '{}': {:?}", file_path.display(), e))?;
            }
        }

        self.completed = true;
        Ok(())
    }

    /// Parse sed script into commands
    fn parse_script(&self, script: &str) -> Result<Vec<SedCmd>, String> {
        let mut commands = Vec::new();

        // Simple parser for basic sed commands
        // Format: [address]command
        // Examples: s/foo/bar/, 2d, 1,3p, /pattern/d

        let parts: Vec<&str> = script.split(';').collect();
        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // Parse address (if any) and command
            let (address, command_str) = self.parse_address_and_command(part)?;
            let command = self.parse_command(command_str)?;

            commands.push(SedCmd { address, command });
        }

        Ok(commands)
    }

    /// Parse address and command from a sed script line
    fn parse_address_and_command<'a>(&self, line: &'a str) -> Result<(SedAddress, &'a str), String> {
        // Check for line number addresses (e.g., "2d", "1,3p")
        if let Some(first_char) = line.chars().next() {
            if first_char.is_ascii_digit() {
                // Parse line number or range
                if let Some(comma_pos) = line.find(',') {
                    // Range: "1,3p"
                    let start_str = &line[..comma_pos];
                    let start: usize = start_str
                        .parse()
                        .map_err(|_| format!("invalid line number: {}", start_str))?;

                    let rest = &line[comma_pos + 1..];
                    let mut i = 0;
                    while i < rest.len() && rest.chars().nth(i).unwrap().is_ascii_digit() {
                        i += 1;
                    }

                    let end: usize = rest[..i]
                        .parse()
                        .map_err(|_| format!("invalid line number: {}", &rest[..i]))?;

                    return Ok((SedAddress::Range(start, end), &rest[i..]));
                } else {
                    // Single line number: "2d"
                    let mut i = 0;
                    while i < line.len() && line.chars().nth(i).unwrap().is_ascii_digit() {
                        i += 1;
                    }

                    let line_num: usize = line[..i]
                        .parse()
                        .map_err(|_| format!("invalid line number: {}", &line[..i]))?;

                    return Ok((SedAddress::Line(line_num), &line[i..]));
                }
            }
        }

        // No address, applies to all lines
        Ok((SedAddress::All, line))
    }

    /// Parse a sed command
    fn parse_command(&self, cmd_str: &str) -> Result<SedCommand, String> {
        let cmd_str = cmd_str.trim();

        if cmd_str.starts_with('s') {
            // Substitute command: s/pattern/replacement/flags
            self.parse_substitute(cmd_str)
        } else if cmd_str == "d" {
            Ok(SedCommand::Delete)
        } else if cmd_str == "p" {
            Ok(SedCommand::Print)
        } else {
            Err(format!("unknown command: {}", cmd_str))
        }
    }

    /// Parse substitute command
    fn parse_substitute(&self, cmd_str: &str) -> Result<SedCommand, String> {
        if !cmd_str.starts_with('s') {
            return Err("substitute command must start with 's'".to_string());
        }

        let rest = &cmd_str[1..];
        if rest.is_empty() {
            return Err("invalid substitute command".to_string());
        }

        let delimiter = rest.chars().next().unwrap();
        let parts: Vec<&str> = rest[1..].split(delimiter).collect();

        if parts.len() < 2 {
            return Err("invalid substitute command format".to_string());
        }

        let pattern = parts[0].to_string();
        let replacement = parts[1].to_string();
        let flags = if parts.len() > 2 { parts[2] } else { "" };
        let global = flags.contains('g');

        Ok(SedCommand::Substitute {
            pattern,
            replacement,
            global,
        })
    }

    /// Check if an address matches the current line
    fn matches_address(&self, line_num: usize, total_lines: usize, address: &SedAddress) -> bool {
        match address {
            SedAddress::All => true,
            SedAddress::Line(n) => line_num == *n,
            SedAddress::Range(start, end) => line_num >= *start && line_num <= *end,
            SedAddress::Last => line_num == total_lines,
        }
    }

    /// Perform substitution on a line
    fn substitute(&self, line: &str, pattern: &str, replacement: &str, global: bool) -> String {
        if global {
            line.replace(pattern, replacement)
        } else {
            // Replace only first occurrence
            if let Some(pos) = line.find(pattern) {
                let mut result = String::new();
                result.push_str(&line[..pos]);
                result.push_str(replacement);
                result.push_str(&line[pos + pattern.len()..]);
                result
            } else {
                line.to_string()
            }
        }
    }
}

/// Sed command types
#[derive(Clone, Debug, PartialEq)]
struct SedCmd {
    address: SedAddress,
    command: SedCommand,
}

/// Sed address types
#[derive(Clone, Debug, PartialEq)]
enum SedAddress {
    All,
    Line(usize),
    Range(usize, usize),
    Last,
}

/// Sed command types
#[derive(Clone, Debug, PartialEq)]
enum SedCommand {
    Substitute {
        pattern: String,
        replacement: String,
        global: bool,
    },
    Delete,
    Print,
}

/// Main loop for sed command
pub fn sed_main_loop(sed: &mut Sed, state: &mut KernelState) -> Option<SystemCall> {
    if !sed.completed && sed.error.is_none() {
        if let Err(e) = sed.process(state) {
            sed.error = Some(e.clone());
            return Some(SystemCall::Write {
                fd: 2,
                data: format!("sed: {}\n", e).as_bytes().to_vec(),
            });
        }
    }

    if sed.completed && !sed.output.is_empty() && !sed.in_place {
        return Some(SystemCall::Write {
            fd: 1,
            data: sed.output.as_bytes().to_vec(),
        });
    }

    Some(SystemCall::Exit(0))
}

// ============================================================================
// Awk - Pattern Processing Language (WOS-PROG-004)
// ============================================================================

/// Awk command for pattern processing
#[derive(Clone, Debug, PartialEq)]
pub struct Awk {
    /// Process ID
    pub pid: ProcessId,
    /// Input file (None for stdin)
    pub file: Option<PathBuf>,
    /// Awk program script
    pub program: String,
    /// Field separator (default: whitespace)
    pub field_separator: String,
    /// Output buffer
    pub output: String,
    /// Completed flag
    pub completed: bool,
    /// Error message
    pub error: Option<String>,
}

impl Awk {
    /// Create a new Awk instance
    pub fn new(pid: ProcessId, program: String, file: Option<PathBuf>) -> Self {
        Self {
            pid,
            file,
            program,
            field_separator: " \t".to_string(),
            output: String::new(),
            completed: false,
            error: None,
        }
    }

    /// Set field separator
    pub fn with_field_separator(mut self, separator: String) -> Self {
        self.field_separator = separator;
        self
    }

    /// Process the input
    pub fn process(&mut self, state: &mut KernelState) -> Result<(), String> {
        if self.completed {
            return Ok(());
        }

        // Read input
        let content = if let Some(ref file_path) = self.file {
            state
                .vfs
                .read_file(file_path)
                .map_err(|e| format!("cannot read '{}': {:?}", file_path.display(), e))?
        } else {
            return Err("stdin not implemented".to_string());
        };

        let text = String::from_utf8(content)
            .map_err(|e| format!("invalid UTF-8: {}", e))?;

        let lines: Vec<&str> = text.lines().collect();

        // Parse awk program
        let actions = self.parse_program(&self.program)?;

        // Execute BEGIN actions
        let mut result_lines = Vec::new();
        for action in &actions {
            if let AwkPattern::Begin = action.pattern {
                for statement in &action.statements {
                    self.execute_statement(statement, &[], 0, &lines, &mut result_lines)?;
                }
            }
        }

        // Process each line
        for (line_num, line) in lines.iter().enumerate() {
            let fields = self.split_fields(line);
            let nr = line_num + 1;

            for action in &actions {
                if self.matches_pattern(&action.pattern, line, nr, &lines) {
                    for statement in &action.statements {
                        self.execute_statement(statement, &fields, nr, &lines, &mut result_lines)?;
                    }
                }
            }
        }

        // Execute END actions
        for action in &actions {
            if let AwkPattern::End = action.pattern {
                for statement in &action.statements {
                    self.execute_statement(statement, &[], lines.len(), &lines, &mut result_lines)?;
                }
            }
        }

        self.output = result_lines.join("\n");
        if !self.output.is_empty() {
            self.output.push('\n');
        }

        self.completed = true;
        Ok(())
    }

    /// Split line into fields
    fn split_fields(&self, line: &str) -> Vec<String> {
        if self.field_separator == " \t" {
            // Default whitespace separator
            line.split_whitespace().map(|s| s.to_string()).collect()
        } else {
            line.split(&self.field_separator).map(|s| s.to_string()).collect()
        }
    }

    /// Parse awk program
    fn parse_program(&self, program: &str) -> Result<Vec<AwkAction>, String> {
        let mut actions = Vec::new();

        // Simple parser for basic awk programs
        // Formats supported:
        // - BEGIN { action }
        // - END { action }
        // - { action } (all lines)
        // - /pattern/ { action }
        // - NR==N { action }

        let program = program.trim();

        // Handle BEGIN block
        if program.starts_with("BEGIN") {
            let rest = &program[5..].trim();
            if let Some(action_str) = self.extract_braces(rest) {
                let statements = self.parse_action(action_str)?;
                actions.push(AwkAction {
                    pattern: AwkPattern::Begin,
                    statements,
                });
                return Ok(actions);
            }
        }

        // Handle END block
        if program.starts_with("END") {
            let rest = &program[3..].trim();
            if let Some(action_str) = self.extract_braces(rest) {
                let statements = self.parse_action(action_str)?;
                actions.push(AwkAction {
                    pattern: AwkPattern::End,
                    statements,
                });
                return Ok(actions);
            }
        }

        // Handle pattern { action } format
        if let Some(brace_start) = program.find('{') {
            let pattern_str = program[..brace_start].trim();
            let action_str = &program[brace_start..];

            if let Some(action_content) = self.extract_braces(action_str) {
                let pattern = if pattern_str.is_empty() {
                    AwkPattern::All
                } else if pattern_str.starts_with('/') && pattern_str.ends_with('/') {
                    // Pattern matching: /regex/
                    let pattern_text = pattern_str[1..pattern_str.len()-1].to_string();
                    AwkPattern::Regex(pattern_text)
                } else if pattern_str.contains("==") {
                    // Condition: NR==5
                    AwkPattern::Condition(pattern_str.to_string())
                } else {
                    AwkPattern::All
                };

                let statements = self.parse_action(action_content)?;
                actions.push(AwkAction { pattern, statements });
            }
        } else {
            // No braces, assume it's a simple action for all lines
            let statements = self.parse_action(program)?;
            actions.push(AwkAction {
                pattern: AwkPattern::All,
                statements,
            });
        }

        Ok(actions)
    }

    /// Extract content between braces
    fn extract_braces<'a>(&self, s: &'a str) -> Option<&'a str> {
        let s = s.trim();
        if !s.starts_with('{') {
            return None;
        }
        let end = s.rfind('}')?;
        Some(s[1..end].trim())
    }

    /// Parse action into statements
    fn parse_action(&self, action: &str) -> Result<Vec<AwkStatement>, String> {
        let mut statements = Vec::new();

        // Split by semicolons for multiple statements
        for stmt in action.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }

            if stmt.starts_with("print") {
                let rest = stmt[5..].trim();
                if rest.is_empty() || rest == "$0" {
                    statements.push(AwkStatement::Print(AwkExpr::Field(0)));
                } else if rest.starts_with('$') {
                    let field_num_str = &rest[1..];
                    let field_num: usize = field_num_str
                        .parse()
                        .map_err(|_| format!("invalid field number: {}", field_num_str))?;
                    statements.push(AwkStatement::Print(AwkExpr::Field(field_num)));
                } else if rest == "NF" {
                    statements.push(AwkStatement::Print(AwkExpr::NF));
                } else if rest == "NR" {
                    statements.push(AwkStatement::Print(AwkExpr::NR));
                } else {
                    statements.push(AwkStatement::Print(AwkExpr::String(rest.to_string())));
                }
            } else {
                // Default: treat as print statement
                statements.push(AwkStatement::Print(AwkExpr::Field(0)));
            }
        }

        if statements.is_empty() {
            statements.push(AwkStatement::Print(AwkExpr::Field(0)));
        }

        Ok(statements)
    }

    /// Check if pattern matches
    fn matches_pattern(&self, pattern: &AwkPattern, line: &str, nr: usize, _lines: &[&str]) -> bool {
        match pattern {
            AwkPattern::All => true,
            AwkPattern::Begin | AwkPattern::End => false,
            AwkPattern::Regex(regex) => line.contains(regex.as_str()),
            AwkPattern::Condition(cond) => {
                // Simple condition parsing: NR==5, NF>3, etc.
                if cond.starts_with("NR==") {
                    let num_str = &cond[4..];
                    if let Ok(num) = num_str.parse::<usize>() {
                        return nr == num;
                    }
                }
                false
            }
        }
    }

    /// Execute a statement
    fn execute_statement(
        &self,
        statement: &AwkStatement,
        fields: &[String],
        nr: usize,
        _lines: &[&str],
        output: &mut Vec<String>,
    ) -> Result<(), String> {
        match statement {
            AwkStatement::Print(expr) => {
                let value = self.evaluate_expr(expr, fields, nr)?;
                output.push(value);
            }
        }
        Ok(())
    }

    /// Evaluate an expression
    fn evaluate_expr(&self, expr: &AwkExpr, fields: &[String], nr: usize) -> Result<String, String> {
        match expr {
            AwkExpr::Field(0) => {
                // $0 is the entire line
                Ok(fields.join(" "))
            }
            AwkExpr::Field(n) => {
                if *n > 0 && *n <= fields.len() {
                    Ok(fields[*n - 1].clone())
                } else {
                    Ok(String::new())
                }
            }
            AwkExpr::NR => Ok(nr.to_string()),
            AwkExpr::NF => Ok(fields.len().to_string()),
            AwkExpr::String(s) => Ok(s.clone()),
        }
    }
}

/// Awk pattern types
#[derive(Clone, Debug, PartialEq)]
enum AwkPattern {
    All,
    Begin,
    End,
    Regex(String),
    Condition(String),
}

/// Awk action (pattern-statement pair)
#[derive(Clone, Debug, PartialEq)]
struct AwkAction {
    pattern: AwkPattern,
    statements: Vec<AwkStatement>,
}

/// Awk statements
#[derive(Clone, Debug, PartialEq)]
enum AwkStatement {
    Print(AwkExpr),
}

/// Awk expressions
#[derive(Clone, Debug, PartialEq)]
enum AwkExpr {
    Field(usize),
    NR,
    NF,
    String(String),
}

/// Main loop for awk command
pub fn awk_main_loop(awk: &mut Awk, state: &mut KernelState) -> Option<SystemCall> {
    if !awk.completed && awk.error.is_none() {
        if let Err(e) = awk.process(state) {
            awk.error = Some(e.clone());
            return Some(SystemCall::Write {
                fd: 2,
                data: format!("awk: {}\n", e).as_bytes().to_vec(),
            });
        }
    }

    if awk.completed && !awk.output.is_empty() {
        return Some(SystemCall::Write {
            fd: 1,
            data: awk.output.as_bytes().to_vec(),
        });
    }

    Some(SystemCall::Exit(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wos_kernel::KernelState;

    // ============================================================================
    // Cp Tests (WOS-PROG-017)
    // ============================================================================

    #[test]
    fn test_cp_basic_copy() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/src.txt"), b"hello".to_vec())
            .unwrap();

        let mut cp = Cp::new(
            1, // ProcessId
            PathBuf::from("/src.txt"),
            PathBuf::from("/dst.txt"),
            false,
            false,
            false,
        );

        let syscall = cp_main_loop(&mut cp, &mut state);
        assert!(cp.completed);
        assert!(cp.error.is_none());
        assert!(matches!(syscall, Some(SystemCall::Exit(0))));

        // Verify destination file exists
        let content = state.vfs.read_file(&PathBuf::from("/dst.txt")).unwrap();
        assert_eq!(content, b"hello");
    }

    #[test]
    fn test_cp_preserve_permissions() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/src.txt"), b"hello".to_vec())
            .unwrap();
        state.vfs.chmod(&PathBuf::from("/src.txt"), 0o755).unwrap();

        let mut cp = Cp::new(
            1, // ProcessId
            PathBuf::from("/src.txt"),
            PathBuf::from("/dst.txt"),
            false,
            true, // preserve
            false,
        );

        cp_main_loop(&mut cp, &mut state);
        assert!(cp.completed);

        // Verify permissions preserved
        let src_perms = state
            .vfs
            .get_permissions(&PathBuf::from("/src.txt"))
            .unwrap();
        let dst_perms = state
            .vfs
            .get_permissions(&PathBuf::from("/dst.txt"))
            .unwrap();
        assert_eq!(src_perms.mode, dst_perms.mode);
    }

    #[test]
    fn test_cp_source_not_found() {
        let mut state = KernelState::new();

        let mut cp = Cp::new(
            1, // ProcessId
            PathBuf::from("/nonexistent.txt"),
            PathBuf::from("/dst.txt"),
            false,
            false,
            false,
        );

        let syscall = cp_main_loop(&mut cp, &mut state);
        assert!(!cp.completed);
        assert!(cp.error.is_some());
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 2, .. })));
    }

    // ============================================================================
    // Mv Tests (WOS-PROG-018)
    // ============================================================================

    #[test]
    fn test_mv_basic_move() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/src.txt"), b"hello".to_vec())
            .unwrap();

        let mut mv = Mv::new(
            1, // ProcessId
            PathBuf::from("/src.txt"),
            PathBuf::from("/dst.txt"),
            false,
        );

        let syscall = mv_main_loop(&mut mv, &mut state);
        assert!(mv.completed);
        assert!(mv.error.is_none());
        assert!(matches!(syscall, Some(SystemCall::Exit(0))));

        // Verify destination exists and source does not
        assert!(state.vfs.exists(&PathBuf::from("/dst.txt")));
        assert!(!state.vfs.exists(&PathBuf::from("/src.txt")));

        let content = state.vfs.read_file(&PathBuf::from("/dst.txt")).unwrap();
        assert_eq!(content, b"hello");
    }

    #[test]
    fn test_mv_preserves_permissions() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/src.txt"), b"hello".to_vec())
            .unwrap();
        state.vfs.chmod(&PathBuf::from("/src.txt"), 0o755).unwrap();

        let src_perms = state
            .vfs
            .get_permissions(&PathBuf::from("/src.txt"))
            .unwrap();

        let mut mv = Mv::new(
            1, // ProcessId
            PathBuf::from("/src.txt"),
            PathBuf::from("/dst.txt"),
            false,
        );

        mv_main_loop(&mut mv, &mut state);
        assert!(mv.completed);

        // Verify permissions preserved
        let dst_perms = state
            .vfs
            .get_permissions(&PathBuf::from("/dst.txt"))
            .unwrap();
        assert_eq!(src_perms.mode, dst_perms.mode);
    }

    #[test]
    fn test_mv_source_not_found() {
        let mut state = KernelState::new();

        let mut mv = Mv::new(
            1, // ProcessId
            PathBuf::from("/nonexistent.txt"),
            PathBuf::from("/dst.txt"),
            false,
        );

        let syscall = mv_main_loop(&mut mv, &mut state);
        assert!(!mv.completed);
        assert!(mv.error.is_some());
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 2, .. })));
    }

    // ============================================================================
    // Mkdir Tests (WOS-PROG-020)
    // ============================================================================

    #[test]
    fn test_mkdir_basic() {
        let mut state = KernelState::new();

        let mut mkdir = Mkdir::new(1, PathBuf::from("/testdir"), false, None);

        let syscall = mkdir_main_loop(&mut mkdir, &mut state);
        assert!(mkdir.completed);
        assert!(mkdir.error.is_none());
        assert!(matches!(syscall, Some(SystemCall::Exit(0))));

        // Verify directory exists
        assert!(state.vfs.exists(&PathBuf::from("/testdir")));
        assert!(state.vfs.is_directory(&PathBuf::from("/testdir")));
    }

    #[test]
    fn test_mkdir_with_parents() {
        let mut state = KernelState::new();

        let mut mkdir = Mkdir::new(
            1, // ProcessId
            PathBuf::from("/a/b/c"),
            true, // parents flag
            None,
        );

        mkdir_main_loop(&mut mkdir, &mut state);
        assert!(mkdir.completed);
        assert!(mkdir.error.is_none());

        // Verify all directories exist
        assert!(state.vfs.exists(&PathBuf::from("/a")));
        assert!(state.vfs.exists(&PathBuf::from("/a/b")));
        assert!(state.vfs.exists(&PathBuf::from("/a/b/c")));
        assert!(state.vfs.is_directory(&PathBuf::from("/a/b/c")));
    }

    #[test]
    fn test_mkdir_with_mode() {
        let mut state = KernelState::new();

        let mut mkdir = Mkdir::new(
            1, // ProcessId
            PathBuf::from("/testdir"),
            false,
            Some(0o755),
        );

        mkdir_main_loop(&mut mkdir, &mut state);
        assert!(mkdir.completed);

        // Verify permissions
        let perms = state
            .vfs
            .get_permissions(&PathBuf::from("/testdir"))
            .unwrap();
        assert_eq!(perms.mode, 0o755);
    }

    #[test]
    fn test_mkdir_parents_already_exists() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/a")).unwrap();

        let mut mkdir = Mkdir::new(1, PathBuf::from("/a/b"), true, None);

        mkdir_main_loop(&mut mkdir, &mut state);
        assert!(mkdir.completed);
        assert!(mkdir.error.is_none());

        assert!(state.vfs.exists(&PathBuf::from("/a/b")));
    }

    #[test]
    fn test_mkdir_parent_is_file() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"hello".to_vec())
            .unwrap();

        let mut mkdir = Mkdir::new(1, PathBuf::from("/file.txt/dir"), true, None);

        mkdir_main_loop(&mut mkdir, &mut state);
        assert!(!mkdir.completed);
        assert!(mkdir.error.is_some());
        assert!(mkdir.error.as_ref().unwrap().contains("not a directory"));
    }

    // ============================================================================
    // Rm Tests (WOS-PROG-019)
    // ============================================================================

    #[test]
    fn test_rm_basic_file() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"hello".to_vec())
            .unwrap();

        let mut rm = Rm::new(1, vec![PathBuf::from("/file.txt")], false, false, false);

        let syscall = rm_main_loop(&mut rm, &mut state);
        assert!(rm.completed);
        assert!(rm.error.is_none());
        assert!(matches!(syscall, Some(SystemCall::Exit(0))));

        // Verify file was removed
        assert!(!state.vfs.exists(&PathBuf::from("/file.txt")));
    }

    #[test]
    fn test_rm_nonexistent_file() {
        let mut state = KernelState::new();

        let mut rm = Rm::new(
            1,
            vec![PathBuf::from("/nonexistent.txt")],
            false,
            false,
            false,
        );

        let syscall = rm_main_loop(&mut rm, &mut state);
        assert!(!rm.completed);
        assert!(rm.error.is_some());
        assert!(rm
            .error
            .as_ref()
            .unwrap()
            .contains("No such file or directory"));
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 2, .. })));
    }

    #[test]
    fn test_rm_directory_without_recursive() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/dir")).unwrap();

        let mut rm = Rm::new(1, vec![PathBuf::from("/dir")], false, false, false);

        let syscall = rm_main_loop(&mut rm, &mut state);
        assert!(!rm.completed);
        assert!(rm.error.is_some());
        assert!(rm.error.as_ref().unwrap().contains("Is a directory"));
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 2, .. })));
    }

    #[test]
    fn test_rm_directory_recursive() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/dir")).unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/dir/file.txt"), b"hello".to_vec())
            .unwrap();

        let mut rm = Rm::new(
            1,
            vec![PathBuf::from("/dir")],
            true, // recursive
            false,
            false,
        );

        rm_main_loop(&mut rm, &mut state);
        assert!(rm.completed);
        assert!(rm.error.is_none());

        // Verify directory and contents removed
        assert!(!state.vfs.exists(&PathBuf::from("/dir")));
        assert!(!state.vfs.exists(&PathBuf::from("/dir/file.txt")));
    }

    #[test]
    fn test_rm_force_nonexistent() {
        let mut state = KernelState::new();

        let mut rm = Rm::new(
            1,
            vec![PathBuf::from("/nonexistent.txt")],
            false,
            true, // force
            false,
        );

        let syscall = rm_main_loop(&mut rm, &mut state);
        assert!(rm.completed);
        assert!(rm.error.is_none());
        assert!(matches!(syscall, Some(SystemCall::Exit(0))));
    }

    #[test]
    fn test_rm_multiple_files() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"hello".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), b"world".to_vec())
            .unwrap();

        let mut rm = Rm::new(
            1,
            vec![PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt")],
            false,
            false,
            false,
        );

        rm_main_loop(&mut rm, &mut state);
        assert!(rm.completed);
        assert!(rm.error.is_none());

        // Verify both files removed
        assert!(!state.vfs.exists(&PathBuf::from("/file1.txt")));
        assert!(!state.vfs.exists(&PathBuf::from("/file2.txt")));
    }

    #[test]
    fn test_rm_nested_directories() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/a")).unwrap();
        state.vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/a/b/file.txt"), b"test".to_vec())
            .unwrap();

        let mut rm = Rm::new(1, vec![PathBuf::from("/a")], true, false, false);

        rm_main_loop(&mut rm, &mut state);
        assert!(rm.completed);
        assert!(rm.error.is_none());

        // Verify entire tree removed
        assert!(!state.vfs.exists(&PathBuf::from("/a")));
        assert!(!state.vfs.exists(&PathBuf::from("/a/b")));
        assert!(!state.vfs.exists(&PathBuf::from("/a/b/file.txt")));
    }

    // ============================================================================
    // Chmod Tests (WOS-PROG-014)
    // ============================================================================

    #[test]
    fn test_chmod_basic() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"hello".to_vec())
            .unwrap();

        let mut chmod = Chmod::new(1, 0o755, vec![PathBuf::from("/file.txt")], false);

        let syscall = chmod_main_loop(&mut chmod, &mut state);
        assert!(chmod.completed);
        assert!(chmod.error.is_none());
        assert!(matches!(syscall, Some(SystemCall::Exit(0))));

        // Verify permissions changed
        let perms = state
            .vfs
            .get_permissions(&PathBuf::from("/file.txt"))
            .unwrap();
        assert_eq!(perms.mode, 0o755);
    }

    #[test]
    fn test_chmod_nonexistent_file() {
        let mut state = KernelState::new();

        let mut chmod = Chmod::new(1, 0o755, vec![PathBuf::from("/nonexistent.txt")], false);

        let syscall = chmod_main_loop(&mut chmod, &mut state);
        assert!(!chmod.completed);
        assert!(chmod.error.is_some());
        assert!(chmod
            .error
            .as_ref()
            .unwrap()
            .contains("No such file or directory"));
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 2, .. })));
    }

    #[test]
    fn test_chmod_multiple_files() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"hello".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), b"world".to_vec())
            .unwrap();

        let mut chmod = Chmod::new(
            1,
            0o644,
            vec![PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt")],
            false,
        );

        chmod_main_loop(&mut chmod, &mut state);
        assert!(chmod.completed);
        assert!(chmod.error.is_none());

        // Verify both files changed
        let perms1 = state
            .vfs
            .get_permissions(&PathBuf::from("/file1.txt"))
            .unwrap();
        let perms2 = state
            .vfs
            .get_permissions(&PathBuf::from("/file2.txt"))
            .unwrap();
        assert_eq!(perms1.mode, 0o644);
        assert_eq!(perms2.mode, 0o644);
    }

    #[test]
    fn test_chmod_directory() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/dir")).unwrap();

        let mut chmod = Chmod::new(1, 0o755, vec![PathBuf::from("/dir")], false);

        chmod_main_loop(&mut chmod, &mut state);
        assert!(chmod.completed);
        assert!(chmod.error.is_none());

        // Verify directory permissions changed
        let perms = state.vfs.get_permissions(&PathBuf::from("/dir")).unwrap();
        assert_eq!(perms.mode, 0o755);
    }

    #[test]
    fn test_chmod_recursive() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/dir")).unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/dir/file.txt"), b"test".to_vec())
            .unwrap();
        state
            .vfs
            .create_directory(PathBuf::from("/dir/subdir"))
            .unwrap();

        let mut chmod = Chmod::new(
            1,
            0o700,
            vec![PathBuf::from("/dir")],
            true, // recursive
        );

        chmod_main_loop(&mut chmod, &mut state);
        assert!(chmod.completed);
        assert!(chmod.error.is_none());

        // Verify all permissions changed
        let dir_perms = state.vfs.get_permissions(&PathBuf::from("/dir")).unwrap();
        let file_perms = state
            .vfs
            .get_permissions(&PathBuf::from("/dir/file.txt"))
            .unwrap();
        let subdir_perms = state
            .vfs
            .get_permissions(&PathBuf::from("/dir/subdir"))
            .unwrap();

        assert_eq!(dir_perms.mode, 0o700);
        assert_eq!(file_perms.mode, 0o700);
        assert_eq!(subdir_perms.mode, 0o700);
    }

    // ============================================================================
    // Find Tests (WOS-PROG-005)
    // ============================================================================

    #[test]
    fn test_find_basic() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"hello".to_vec())
            .unwrap();

        let mut find = Find::new(1, PathBuf::from("/"), None, None);

        let syscall = find_main_loop(&mut find, &mut state);
        assert!(find.completed);
        assert!(find.error.is_none());

        // Should find at least the root directory and the file
        assert!(find.matches.len() >= 2);
        assert!(find.matches.contains(&PathBuf::from("/")));
        assert!(find.matches.contains(&PathBuf::from("/file.txt")));
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 1, .. })));
    }

    #[test]
    fn test_find_with_name_pattern() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/test.txt"), b"hello".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file.log"), b"world".to_vec())
            .unwrap();

        let mut find = Find::new(1, PathBuf::from("/"), Some("test".to_string()), None);

        find_main_loop(&mut find, &mut state);
        assert!(find.completed);

        // Should only find files with "test" in the name
        assert_eq!(find.matches.len(), 1);
        assert!(find.matches.contains(&PathBuf::from("/test.txt")));
    }

    #[test]
    fn test_find_type_file() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/dir")).unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"hello".to_vec())
            .unwrap();

        let mut find = Find::new(1, PathBuf::from("/"), None, Some('f'));

        find_main_loop(&mut find, &mut state);
        assert!(find.completed);

        // Should only find files, not directories
        assert_eq!(find.matches.len(), 1);
        assert!(find.matches.contains(&PathBuf::from("/file.txt")));
        assert!(!find.matches.contains(&PathBuf::from("/")));
        assert!(!find.matches.contains(&PathBuf::from("/dir")));
    }

    #[test]
    fn test_find_type_directory() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/dir")).unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"hello".to_vec())
            .unwrap();

        let mut find = Find::new(1, PathBuf::from("/"), None, Some('d'));

        find_main_loop(&mut find, &mut state);
        assert!(find.completed);

        // Should only find directories, not files
        // Note: KernelState may have default directories like /bin, /dev, etc.
        assert!(find.matches.len() >= 2);
        assert!(find.matches.contains(&PathBuf::from("/")));
        assert!(find.matches.contains(&PathBuf::from("/dir")));
        assert!(!find.matches.contains(&PathBuf::from("/file.txt")));
    }

    #[test]
    fn test_find_nested_directories() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/a")).unwrap();
        state.vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/a/b/file.txt"), b"test".to_vec())
            .unwrap();

        let mut find = Find::new(1, PathBuf::from("/a"), None, None);

        find_main_loop(&mut find, &mut state);
        assert!(find.completed);

        // Should find all nested items
        assert!(find.matches.len() >= 3);
        assert!(find.matches.contains(&PathBuf::from("/a")));
        assert!(find.matches.contains(&PathBuf::from("/a/b")));
        assert!(find.matches.contains(&PathBuf::from("/a/b/file.txt")));
    }

    #[test]
    fn test_find_nonexistent_path() {
        let mut state = KernelState::new();

        let mut find = Find::new(1, PathBuf::from("/nonexistent"), None, None);

        let syscall = find_main_loop(&mut find, &mut state);
        assert!(!find.completed);
        assert!(find.error.is_some());
        assert!(find
            .error
            .as_ref()
            .unwrap()
            .contains("No such file or directory"));
        assert!(matches!(syscall, Some(SystemCall::Write { fd: 2, .. })));
    }

    #[test]
    fn test_find_combined_filters() {
        let mut state = KernelState::new();
        state.vfs.create_directory(PathBuf::from("/test")).unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/test.txt"), b"hello".to_vec())
            .unwrap();

        let mut find = Find::new(1, PathBuf::from("/"), Some("test".to_string()), Some('f'));

        find_main_loop(&mut find, &mut state);
        assert!(find.completed);

        // Should only find files with "test" in name
        assert_eq!(find.matches.len(), 1);
        assert!(find.matches.contains(&PathBuf::from("/test.txt")));
        assert!(!find.matches.contains(&PathBuf::from("/test")));
    }

    // ============================================================================
    // Sort Tests (WOS-PROG-009)
    // ============================================================================

    #[test]
    fn test_sort_alphabetic() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"zebra\napple\nbanana\n".to_vec())
            .unwrap();

        let mut sort = Sort::new(1, Some(PathBuf::from("/file.txt")), false, false, false);

        sort_main_loop(&mut sort, &mut state);
        assert!(sort.completed);
        assert_eq!(sort.output, "apple\nbanana\nzebra\n");
    }

    #[test]
    fn test_sort_numeric() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/numbers.txt"), b"100\n2\n30\n".to_vec())
            .unwrap();

        let mut sort = Sort::new(1, Some(PathBuf::from("/numbers.txt")), true, false, false);

        sort_main_loop(&mut sort, &mut state);
        assert!(sort.completed);
        assert_eq!(sort.output, "2\n30\n100\n");
    }

    #[test]
    fn test_sort_reverse() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a\nb\nc\n".to_vec())
            .unwrap();

        let mut sort = Sort::new(1, Some(PathBuf::from("/file.txt")), false, true, false);

        sort_main_loop(&mut sort, &mut state);
        assert!(sort.completed);
        assert_eq!(sort.output, "c\nb\na\n");
    }

    #[test]
    fn test_sort_unique() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a\nb\na\nc\nb\n".to_vec())
            .unwrap();

        let mut sort = Sort::new(1, Some(PathBuf::from("/file.txt")), false, false, true);

        sort_main_loop(&mut sort, &mut state);
        assert!(sort.completed);
        assert_eq!(sort.output, "a\nb\nc\n");
    }

    // ============================================================================
    // Uniq Tests (WOS-PROG-010)
    // ============================================================================

    #[test]
    fn test_uniq_basic() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a\na\nb\nc\nc\nc\n".to_vec())
            .unwrap();

        let mut uniq = Uniq::new(1, Some(PathBuf::from("/file.txt")), false, false, false);

        uniq_main_loop(&mut uniq, &mut state);
        assert!(uniq.completed);
        assert_eq!(uniq.output, "a\nb\nc\n");
    }

    #[test]
    fn test_uniq_count() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a\na\nb\nc\nc\nc\n".to_vec())
            .unwrap();

        let mut uniq = Uniq::new(1, Some(PathBuf::from("/file.txt")), true, false, false);

        uniq_main_loop(&mut uniq, &mut state);
        assert!(uniq.completed);
        assert!(uniq.output.contains("2 a"));
        assert!(uniq.output.contains("1 b"));
        assert!(uniq.output.contains("3 c"));
    }

    #[test]
    fn test_uniq_only_duplicates() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a\na\nb\nc\nc\n".to_vec())
            .unwrap();

        let mut uniq = Uniq::new(1, Some(PathBuf::from("/file.txt")), false, true, false);

        uniq_main_loop(&mut uniq, &mut state);
        assert!(uniq.completed);
        assert_eq!(uniq.output, "a\nc\n");
    }

    #[test]
    fn test_uniq_only_unique() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a\na\nb\nc\nc\n".to_vec())
            .unwrap();

        let mut uniq = Uniq::new(1, Some(PathBuf::from("/file.txt")), false, false, true);

        uniq_main_loop(&mut uniq, &mut state);
        assert!(uniq.completed);
        assert_eq!(uniq.output, "b\n");
    }

    // ============================================================================
    // Cut Tests (WOS-PROG-011)
    // ============================================================================

    #[test]
    fn test_cut_fields_basic() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a\tb\tc\nd\te\tf\n".to_vec())
            .unwrap();

        let mut cut = Cut::new(1, Some(PathBuf::from("/file.txt")))
            .with_fields(vec![1, 3]);

        cut_main_loop(&mut cut, &mut state);
        assert!(cut.completed);
        assert_eq!(cut.output, "a\tc\nd\tf\n");
    }

    #[test]
    fn test_cut_fields_with_custom_delimiter() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a,b,c,d\ne,f,g,h\n".to_vec())
            .unwrap();

        let mut cut = Cut::new(1, Some(PathBuf::from("/file.txt")))
            .with_fields(vec![2, 4])
            .with_delimiter(',');

        cut_main_loop(&mut cut, &mut state);
        assert!(cut.completed);
        assert_eq!(cut.output, "b,d\nf,h\n");
    }

    #[test]
    fn test_cut_characters() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"abcdef\nghijkl\n".to_vec())
            .unwrap();

        let mut cut = Cut::new(1, Some(PathBuf::from("/file.txt")))
            .with_characters(vec![1, 3, 5]);

        cut_main_loop(&mut cut, &mut state);
        assert!(cut.completed);
        assert_eq!(cut.output, "ace\ngik\n");
    }

    #[test]
    fn test_cut_bytes() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"hello\nworld\n".to_vec())
            .unwrap();

        let mut cut = Cut::new(1, Some(PathBuf::from("/file.txt")))
            .with_bytes(vec![1, 2, 5]);

        cut_main_loop(&mut cut, &mut state);
        assert!(cut.completed);
        assert_eq!(cut.output, "heo\nwod\n");
    }

    #[test]
    fn test_cut_fields_out_of_range() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"a\tb\tc\n".to_vec())
            .unwrap();

        let mut cut = Cut::new(1, Some(PathBuf::from("/file.txt")))
            .with_fields(vec![1, 5, 10]);

        cut_main_loop(&mut cut, &mut state);
        assert!(cut.completed);
        // Only field 1 exists, fields 5 and 10 are out of range
        assert_eq!(cut.output, "a\n");
    }

    #[test]
    fn test_cut_characters_partial() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"abc\n".to_vec())
            .unwrap();

        let mut cut = Cut::new(1, Some(PathBuf::from("/file.txt")))
            .with_characters(vec![2, 10]);

        cut_main_loop(&mut cut, &mut state);
        assert!(cut.completed);
        // Only character 2 exists
        assert_eq!(cut.output, "b\n");
    }

    #[test]
    fn test_cut_empty_file() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file.txt"), b"".to_vec())
            .unwrap();

        let mut cut = Cut::new(1, Some(PathBuf::from("/file.txt")))
            .with_fields(vec![1]);

        cut_main_loop(&mut cut, &mut state);
        assert!(cut.completed);
        assert_eq!(cut.output, "");
    }

    #[test]
    fn test_cut_missing_file() {
        let mut state = KernelState::new();

        let mut cut = Cut::new(1, Some(PathBuf::from("/missing.txt")))
            .with_fields(vec![1]);

        cut_main_loop(&mut cut, &mut state);
        assert!(cut.error.is_some());
        assert!(cut.error.as_ref().unwrap().contains("cannot read"));
    }

    // ============================================================================
    // Diff Tests (WOS-PROG-012)
    // ============================================================================

    #[test]
    fn test_diff_identical_files() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"line1\nline2\n".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), b"line1\nline2\n".to_vec())
            .unwrap();

        let mut diff = Diff::new(1, PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"));

        let syscall = diff_main_loop(&mut diff, &mut state);
        assert!(diff.completed);
        assert_eq!(diff.output, "");
        assert!(matches!(syscall, Some(SystemCall::Exit(0))));
    }

    #[test]
    fn test_diff_different_files_normal() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"line1\nline2\n".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), b"line1\nline3\n".to_vec())
            .unwrap();

        let mut diff = Diff::new(1, PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"));

        diff_main_loop(&mut diff, &mut state);
        assert!(diff.completed);
        assert!(!diff.output.is_empty());
        assert!(diff.output.contains("2c2"));
        assert!(diff.output.contains("< line2"));
        assert!(diff.output.contains("> line3"));
    }

    #[test]
    fn test_diff_unified_format() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"line1\nline2\nline3\n".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), b"line1\nmodified\nline3\n".to_vec())
            .unwrap();

        let mut diff = Diff::new(1, PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"))
            .with_unified(3);

        diff_main_loop(&mut diff, &mut state);
        assert!(diff.completed);
        assert!(diff.output.contains("---"));
        assert!(diff.output.contains("+++"));
        assert!(diff.output.contains("@@"));
        assert!(diff.output.contains("-line2"));
        assert!(diff.output.contains("+modified"));
    }

    #[test]
    fn test_diff_brief_mode() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"line1\n".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), b"line2\n".to_vec())
            .unwrap();

        let mut diff = Diff::new(1, PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"))
            .with_brief();

        diff_main_loop(&mut diff, &mut state);
        assert!(diff.completed);
        assert_eq!(diff.output, "Files /file1.txt and /file2.txt differ\n");
    }

    #[test]
    fn test_diff_ignore_whitespace() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"line1\n  line2  \n".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), b"line1\nline2\n".to_vec())
            .unwrap();

        let mut diff = Diff::new(1, PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"))
            .with_ignore_whitespace();

        diff_main_loop(&mut diff, &mut state);
        assert!(diff.completed);
        assert_eq!(diff.output, ""); // Should be identical when ignoring whitespace
    }

    #[test]
    fn test_diff_added_lines() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"line1\n".to_vec())
            .unwrap();
        state
            .vfs
            .create_file(
                PathBuf::from("/file2.txt"),
                b"line1\nline2\nline3\n".to_vec(),
            )
            .unwrap();

        let mut diff = Diff::new(1, PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"));

        diff_main_loop(&mut diff, &mut state);
        assert!(diff.completed);
        assert!(!diff.output.is_empty());
        assert!(diff.output.contains(">"));
    }

    #[test]
    fn test_diff_deleted_lines() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file1.txt"),
                b"line1\nline2\nline3\n".to_vec(),
            )
            .unwrap();
        state
            .vfs
            .create_file(PathBuf::from("/file2.txt"), b"line1\n".to_vec())
            .unwrap();

        let mut diff = Diff::new(1, PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"));

        diff_main_loop(&mut diff, &mut state);
        assert!(diff.completed);
        assert!(!diff.output.is_empty());
        assert!(diff.output.contains("<"));
    }

    #[test]
    fn test_diff_missing_file() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(PathBuf::from("/file1.txt"), b"line1\n".to_vec())
            .unwrap();

        let mut diff = Diff::new(
            1,
            PathBuf::from("/file1.txt"),
            PathBuf::from("/missing.txt"),
        );

        diff_main_loop(&mut diff, &mut state);
        assert!(diff.error.is_some());
        assert!(diff.error.as_ref().unwrap().contains("cannot read"));
    }

    // ============================================================================
    // Sed Tests (WOS-PROG-003)
    // ============================================================================

    #[test]
    fn test_sed_substitute_basic() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"hello world\nhello again\n".to_vec(),
            )
            .unwrap();

        let mut sed = Sed::new(1, "s/hello/hi/".to_string(), Some(PathBuf::from("/file.txt")));

        sed_main_loop(&mut sed, &mut state);
        assert!(sed.completed);
        assert_eq!(sed.output, "hi world\nhi again\n");
    }

    #[test]
    fn test_sed_substitute_global() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"foo bar foo baz\n".to_vec(),
            )
            .unwrap();

        let mut sed = Sed::new(1, "s/foo/FOO/g".to_string(), Some(PathBuf::from("/file.txt")));

        sed_main_loop(&mut sed, &mut state);
        assert!(sed.completed);
        assert_eq!(sed.output, "FOO bar FOO baz\n");
    }

    #[test]
    fn test_sed_delete_line() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"line1\nline2\nline3\n".to_vec(),
            )
            .unwrap();

        let mut sed = Sed::new(1, "2d".to_string(), Some(PathBuf::from("/file.txt")));

        sed_main_loop(&mut sed, &mut state);
        assert!(sed.completed);
        assert_eq!(sed.output, "line1\nline3\n");
    }

    #[test]
    fn test_sed_delete_range() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"line1\nline2\nline3\nline4\n".to_vec(),
            )
            .unwrap();

        let mut sed = Sed::new(1, "2,3d".to_string(), Some(PathBuf::from("/file.txt")));

        sed_main_loop(&mut sed, &mut state);
        assert!(sed.completed);
        assert_eq!(sed.output, "line1\nline4\n");
    }

    #[test]
    fn test_sed_print_command() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"line1\nline2\nline3\n".to_vec(),
            )
            .unwrap();

        let mut sed = Sed::new(1, "2p".to_string(), Some(PathBuf::from("/file.txt")));

        sed_main_loop(&mut sed, &mut state);
        assert!(sed.completed);
        // Without -n, line 2 is printed twice (once by p, once automatically)
        assert_eq!(sed.output, "line1\nline2\nline2\nline3\n");
    }

    #[test]
    fn test_sed_quiet_mode_with_print() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"line1\nline2\nline3\n".to_vec(),
            )
            .unwrap();

        let mut sed = Sed::new(1, "2p".to_string(), Some(PathBuf::from("/file.txt")))
            .with_quiet();

        sed_main_loop(&mut sed, &mut state);
        assert!(sed.completed);
        // With -n, only explicit p command prints
        assert_eq!(sed.output, "line2\n");
    }

    #[test]
    fn test_sed_multiple_commands() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"foo bar\nbaz qux\n".to_vec(),
            )
            .unwrap();

        let mut sed = Sed::new(
            1,
            "s/foo/FOO/;s/bar/BAR/".to_string(),
            Some(PathBuf::from("/file.txt")),
        );

        sed_main_loop(&mut sed, &mut state);
        assert!(sed.completed);
        assert_eq!(sed.output, "FOO BAR\nbaz qux\n");
    }

    #[test]
    fn test_sed_in_place_editing() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"hello world\n".to_vec(),
            )
            .unwrap();

        let mut sed = Sed::new(1, "s/hello/hi/".to_string(), Some(PathBuf::from("/file.txt")))
            .with_in_place();

        sed_main_loop(&mut sed, &mut state);
        if let Some(ref err) = sed.error {
            panic!("sed error: {}", err);
        }
        assert!(sed.completed);

        // Check that file was modified in place
        let content = state.vfs.read_file(&PathBuf::from("/file.txt")).unwrap();
        assert_eq!(String::from_utf8(content).unwrap(), "hi world\n");
    }

    #[test]
    fn test_sed_missing_file() {
        let mut state = KernelState::new();

        let mut sed = Sed::new(
            1,
            "s/foo/bar/".to_string(),
            Some(PathBuf::from("/missing.txt")),
        );

        sed_main_loop(&mut sed, &mut state);
        assert!(sed.error.is_some());
        assert!(sed.error.as_ref().unwrap().contains("cannot read"));
    }

    // ============================================================================
    // Awk Tests (WOS-PROG-004)
    // ============================================================================

    #[test]
    fn test_awk_print_field() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"alice 25\nbob 30\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(1, "{print $1}".to_string(), Some(PathBuf::from("/file.txt")));

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "alice\nbob\n");
    }

    #[test]
    fn test_awk_print_multiple_fields() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"alice 25 engineer\nbob 30 doctor\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(1, "{print $2}".to_string(), Some(PathBuf::from("/file.txt")));

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "25\n30\n");
    }

    #[test]
    fn test_awk_print_nr() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"line1\nline2\nline3\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(1, "{print NR}".to_string(), Some(PathBuf::from("/file.txt")));

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "1\n2\n3\n");
    }

    #[test]
    fn test_awk_print_nf() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"a b c\nd e\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(1, "{print NF}".to_string(), Some(PathBuf::from("/file.txt")));

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "3\n2\n");
    }

    #[test]
    fn test_awk_pattern_match() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"apple\nbanana\napricot\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(
            1,
            "/ap/ {print $0}".to_string(),
            Some(PathBuf::from("/file.txt")),
        );

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "apple\napricot\n");
    }

    #[test]
    fn test_awk_nr_condition() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"line1\nline2\nline3\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(
            1,
            "NR==2 {print $0}".to_string(),
            Some(PathBuf::from("/file.txt")),
        );

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "line2\n");
    }

    #[test]
    fn test_awk_begin_block() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"line1\nline2\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(
            1,
            "BEGIN {print START}".to_string(),
            Some(PathBuf::from("/file.txt")),
        );

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "START\n");
    }

    #[test]
    fn test_awk_end_block() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"line1\nline2\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(
            1,
            "END {print DONE}".to_string(),
            Some(PathBuf::from("/file.txt")),
        );

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "DONE\n");
    }

    #[test]
    fn test_awk_field_separator() {
        let mut state = KernelState::new();
        state
            .vfs
            .create_file(
                PathBuf::from("/file.txt"),
                b"a,b,c\nd,e,f\n".to_vec(),
            )
            .unwrap();

        let mut awk = Awk::new(1, "{print $2}".to_string(), Some(PathBuf::from("/file.txt")))
            .with_field_separator(",".to_string());

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.completed);
        assert_eq!(awk.output, "b\ne\n");
    }

    #[test]
    fn test_awk_missing_file() {
        let mut state = KernelState::new();

        let mut awk = Awk::new(
            1,
            "{print $1}".to_string(),
            Some(PathBuf::from("/missing.txt")),
        );

        awk_main_loop(&mut awk, &mut state);
        assert!(awk.error.is_some());
        assert!(awk.error.as_ref().unwrap().contains("cannot read"));
    }
}
