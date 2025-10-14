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

impl FilePermissions {
    /// Create read-write permissions
    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
        }
    }

    /// Create read-only permissions
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
        }
    }
}

impl Default for FilePermissions {
    fn default() -> Self {
        Self::read_write()
    }
}

impl FileEntry {
    /// Create new file with content
    pub fn new(content: Vec<u8>, permissions: FilePermissions) -> Self {
        Self {
            content,
            permissions,
        }
    }

    /// Create empty file with default permissions
    pub fn empty() -> Self {
        Self {
            content: Vec::new(),
            permissions: FilePermissions::default(),
        }
    }
}

/// Error type for VFS operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VfsError {
    /// File not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// File already exists
    AlreadyExists,
    /// Invalid path
    InvalidPath,
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

    /// Create a file
    pub fn create_file(&mut self, path: PathBuf, content: Vec<u8>) -> Result<(), VfsError> {
        if self.files.contains_key(&path) {
            return Err(VfsError::AlreadyExists);
        }
        let entry = FileEntry::new(content, FilePermissions::default());
        self.files.insert(path, entry);
        Ok(())
    }

    /// Read file contents
    pub fn read_file(&self, path: &PathBuf) -> Result<Vec<u8>, VfsError> {
        let entry = self.files.get(path).ok_or(VfsError::NotFound)?;
        if !entry.permissions.read {
            return Err(VfsError::PermissionDenied);
        }
        Ok(entry.content.clone())
    }

    /// Write to file (overwrites existing content)
    pub fn write_file(&mut self, path: &PathBuf, content: Vec<u8>) -> Result<(), VfsError> {
        let entry = self.files.get(path).ok_or(VfsError::NotFound)?;
        if !entry.permissions.write {
            return Err(VfsError::PermissionDenied);
        }
        let new_entry = FileEntry::new(content, entry.permissions.clone());
        self.files.insert(path.clone(), new_entry);
        Ok(())
    }

    /// Check if file exists
    pub fn exists(&self, path: &PathBuf) -> bool {
        self.files.contains_key(path)
    }

    /// Delete a file
    pub fn delete_file(&mut self, path: &PathBuf) -> Result<(), VfsError> {
        if !self.files.contains_key(path) {
            return Err(VfsError::NotFound);
        }
        self.files.remove(path);
        Ok(())
    }

    /// Get file permissions
    pub fn get_permissions(&self, path: &PathBuf) -> Result<FilePermissions, VfsError> {
        let entry = self.files.get(path).ok_or(VfsError::NotFound)?;
        Ok(entry.permissions.clone())
    }

    /// Set file permissions
    pub fn set_permissions(
        &mut self,
        path: &PathBuf,
        permissions: FilePermissions,
    ) -> Result<(), VfsError> {
        let entry = self.files.get(path).ok_or(VfsError::NotFound)?;
        let new_entry = FileEntry::new(entry.content.clone(), permissions);
        self.files.insert(path.clone(), new_entry);
        Ok(())
    }

    /// List all files (returns paths)
    pub fn list_files(&self) -> Vec<PathBuf> {
        self.files.keys().cloned().collect()
    }

    /// Get file count
    pub fn file_count(&self) -> usize {
        self.files.len()
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
        assert_eq!(vfs.file_count(), 0);
    }

    #[test]
    fn test_vfs_clone_cheap() {
        let vfs = VirtualFileSystem::new();
        let _cloned = vfs.clone();
        // Clone should be O(1) due to persistent data structures
        assert_eq!(vfs, _cloned);
    }

    #[test]
    fn test_create_file() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        let content = b"Hello, World!".to_vec();

        let result = vfs.create_file(path.clone(), content.clone());
        assert!(result.is_ok());
        assert_eq!(vfs.file_count(), 1);
        assert!(vfs.exists(&path));
    }

    #[test]
    fn test_create_file_already_exists() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), vec![]).unwrap();
        let result = vfs.create_file(path, vec![]);
        assert_eq!(result, Err(VfsError::AlreadyExists));
    }

    #[test]
    fn test_read_file() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        let content = b"Hello, World!".to_vec();

        vfs.create_file(path.clone(), content.clone()).unwrap();
        let read_content = vfs.read_file(&path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/nonexistent.txt");

        let result = vfs.read_file(&path);
        assert_eq!(result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_write_file() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), b"Initial".to_vec()).unwrap();
        let new_content = b"Updated".to_vec();
        vfs.write_file(&path, new_content.clone()).unwrap();

        let read_content = vfs.read_file(&path).unwrap();
        assert_eq!(read_content, new_content);
    }

    #[test]
    fn test_write_nonexistent_file() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/nonexistent.txt");

        let result = vfs.write_file(&path, vec![]);
        assert_eq!(result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_delete_file() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), vec![]).unwrap();
        assert_eq!(vfs.file_count(), 1);

        vfs.delete_file(&path).unwrap();
        assert_eq!(vfs.file_count(), 0);
        assert!(!vfs.exists(&path));
    }

    #[test]
    fn test_delete_nonexistent_file() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/nonexistent.txt");

        let result = vfs.delete_file(&path);
        assert_eq!(result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_file_permissions() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), vec![]).unwrap();

        // Default permissions should be read-write
        let perms = vfs.get_permissions(&path).unwrap();
        assert!(perms.read);
        assert!(perms.write);
        assert!(!perms.execute);

        // Change to read-only
        vfs.set_permissions(&path, FilePermissions::read_only())
            .unwrap();
        let perms = vfs.get_permissions(&path).unwrap();
        assert!(perms.read);
        assert!(!perms.write);
    }

    #[test]
    fn test_read_permission_denied() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), b"Secret".to_vec()).unwrap();

        // Set to no read permission
        let no_read = FilePermissions {
            read: false,
            write: true,
            execute: false,
        };
        vfs.set_permissions(&path, no_read).unwrap();

        let result = vfs.read_file(&path);
        assert_eq!(result, Err(VfsError::PermissionDenied));
    }

    #[test]
    fn test_write_permission_denied() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), vec![]).unwrap();
        vfs.set_permissions(&path, FilePermissions::read_only())
            .unwrap();

        let result = vfs.write_file(&path, b"New content".to_vec());
        assert_eq!(result, Err(VfsError::PermissionDenied));
    }

    #[test]
    fn test_list_files() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file1.txt"), vec![])
            .unwrap();
        vfs.create_file(PathBuf::from("/file2.txt"), vec![])
            .unwrap();
        vfs.create_file(PathBuf::from("/file3.txt"), vec![])
            .unwrap();

        let files = vfs.list_files();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&PathBuf::from("/file1.txt")));
        assert!(files.contains(&PathBuf::from("/file2.txt")));
        assert!(files.contains(&PathBuf::from("/file3.txt")));
    }
}
