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
}
