//! Virtual File System
//!
//! Persistent data structure-based VFS using im-rs for O(1) cloning.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Virtual file system with persistent data structures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VirtualFileSystem {
    /// Files stored as persistent HashMap (O(1) clone)
    files: im::HashMap<PathBuf, FileEntry>,
    /// Current working directory
    cwd: PathBuf,
}

/// File entry in the VFS
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    /// File contents
    pub content: Vec<u8>,
    /// File permissions (simplified)
    pub permissions: FilePermissions,
}

/// Simplified file permissions
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FilePermissions {
    /// Can read
    pub read: bool,
    /// Can write
    pub write: bool,
    /// Can execute
    pub execute: bool,
}

impl VirtualFileSystem {
    /// Create a new empty VFS
    pub fn new() -> Self {
        Self {
            files: im::HashMap::new(),
            cwd: PathBuf::from("/"),
        }
    }

    /// Get current working directory
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_creation() {
        let vfs = VirtualFileSystem::new();
        assert_eq!(vfs.cwd(), &PathBuf::from("/"));
    }

    #[test]
    fn test_vfs_clone_cheap() {
        let vfs = VirtualFileSystem::new();
        let _cloned = vfs.clone();
        // Clone should be O(1) due to persistent data structures
        assert_eq!(vfs, _cloned);
    }
}
