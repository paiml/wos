//! Virtual File System
//!
//! Persistent data structure-based VFS using im-rs for O(1) cloning.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Unique inode number
type InodeNumber = u64;

/// Virtual file system with persistent data structures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VirtualFileSystem {
    /// Inodes stored as persistent HashMap (O(1) clone)
    inodes: im::HashMap<InodeNumber, Inode>,
    /// Root directory inode number (always 1)
    root_ino: InodeNumber,
    /// Next available inode number
    next_ino: InodeNumber,
    /// Current working directory
    cwd: PathBuf,
}

/// Inode representing a file or directory
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Inode {
    /// Inode number
    pub ino: InodeNumber,
    /// Inode type
    pub inode_type: InodeType,
    /// File permissions (simplified)
    pub permissions: FilePermissions,
}

/// Type of inode
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InodeType {
    /// Regular file
    File {
        /// File contents
        content: Vec<u8>,
    },
    /// Directory
    Directory {
        /// Directory entries (name -> inode number)
        entries: im::HashMap<String, InodeNumber>,
    },
    /// Symbolic link
    Symlink {
        /// Target path (can be relative or absolute)
        target: PathBuf,
    },
}

/// Directory entry for listing
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DirectoryEntry {
    /// Entry name
    pub name: String,
    /// Inode number
    pub ino: InodeNumber,
    /// Entry type
    pub entry_type: EntryType,
}

/// Type of directory entry
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    /// File entry
    File,
    /// Directory entry
    Directory,
    /// Symlink entry
    Symlink,
}

impl DirectoryEntry {
    /// Check if entry is a file
    pub fn is_file(&self) -> bool {
        matches!(self.entry_type, EntryType::File)
    }

    /// Check if entry is a directory
    pub fn is_directory(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory)
    }

    /// Check if entry is a symlink
    pub fn is_symlink(&self) -> bool {
        matches!(self.entry_type, EntryType::Symlink)
    }
}

/// Legacy file entry for compatibility
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
    /// Not a directory
    NotADirectory,
    /// Is a directory
    IsADirectory,
    /// Directory not empty
    DirectoryNotEmpty,
    /// Symbolic link loop detected
    SymlinkLoop,
}

impl VirtualFileSystem {
    /// Create a new VFS with root directory and standard directory structure
    pub fn new() -> Self {
        let root_ino = 1;
        let mut inodes = im::HashMap::new();

        // Create root directory
        let root = Inode {
            ino: root_ino,
            inode_type: InodeType::Directory {
                entries: im::HashMap::new(),
            },
            permissions: FilePermissions::default(),
        };
        inodes.insert(root_ino, root);

        let mut vfs = Self {
            inodes,
            root_ino,
            next_ino: 2,
            cwd: PathBuf::from("/"),
        };

        // Create standard directory structure
        let _ = vfs.create_directory(PathBuf::from("/home"));
        let _ = vfs.create_directory(PathBuf::from("/home/user"));
        let _ = vfs.create_directory(PathBuf::from("/tmp"));
        let _ = vfs.create_directory(PathBuf::from("/etc"));
        let _ = vfs.create_directory(PathBuf::from("/bin"));
        let _ = vfs.create_directory(PathBuf::from("/dev"));
        let _ = vfs.create_directory(PathBuf::from("/proc"));
        let _ = vfs.create_directory(PathBuf::from("/scripts"));
        let _ = vfs.create_directory(PathBuf::from("/usr"));
        let _ = vfs.create_directory(PathBuf::from("/usr/local"));
        let _ = vfs.create_directory(PathBuf::from("/usr/local/bin"));

        vfs
    }

    /// Get current working directory
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Maximum symlink depth to prevent infinite loops
    const MAX_SYMLINK_DEPTH: usize = 40;

    /// Resolve path to inode number, following symlinks
    fn resolve_path(&self, path: &Path) -> Result<InodeNumber, VfsError> {
        self.resolve_path_internal(path, true, 0)
    }

    /// Resolve path to inode number without following the final component if it's a symlink
    fn resolve_path_no_follow(&self, path: &Path) -> Result<InodeNumber, VfsError> {
        self.resolve_path_internal(path, false, 0)
    }

    /// Internal path resolution with symlink following control
    fn resolve_path_internal(
        &self,
        path: &Path,
        follow_final: bool,
        depth: usize,
    ) -> Result<InodeNumber, VfsError> {
        if depth > Self::MAX_SYMLINK_DEPTH {
            return Err(VfsError::SymlinkLoop);
        }

        if path == Path::new("/") {
            return Ok(self.root_ino);
        }

        let components: Vec<&str> = path
            .to_str()
            .ok_or(VfsError::InvalidPath)?
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut current_ino = self.root_ino;
        let mut current_path = PathBuf::from("/");

        for (idx, component) in components.iter().enumerate() {
            let is_last = idx == components.len() - 1;
            let inode = self.inodes.get(&current_ino).ok_or(VfsError::NotFound)?;

            match &inode.inode_type {
                InodeType::Directory { entries } => {
                    current_ino = *entries.get(*component).ok_or(VfsError::NotFound)?;

                    // Update current path for relative symlink resolution
                    if current_path.to_str() == Some("/") {
                        current_path = PathBuf::from(format!("/{}", component));
                    } else {
                        current_path =
                            PathBuf::from(format!("{}/{}", current_path.display(), component));
                    }

                    // Follow symlinks in intermediate path components (always)
                    // or in final component if follow_final is true
                    if !is_last || follow_final {
                        let next_inode = self.inodes.get(&current_ino).ok_or(VfsError::NotFound)?;
                        if let InodeType::Symlink { target } = &next_inode.inode_type {
                            // Resolve symlink target
                            let resolved_target = if target.is_absolute() {
                                target.clone()
                            } else {
                                // Relative symlink - resolve relative to parent directory
                                let parent = current_path.parent().unwrap_or(Path::new("/"));
                                parent.join(target)
                            };
                            // Recursively resolve with increased depth
                            current_ino =
                                self.resolve_path_internal(&resolved_target, true, depth + 1)?;
                        }
                    }
                }
                InodeType::Symlink { target } => {
                    // This happens if a symlink appears in the middle of the path
                    let resolved_target = if target.is_absolute() {
                        target.clone()
                    } else {
                        let parent = current_path.parent().unwrap_or(Path::new("/"));
                        parent.join(target)
                    };
                    // Continue with remaining components after symlink
                    let remaining: PathBuf = components[idx..].iter().collect();
                    let new_path = resolved_target.join(remaining);
                    return self.resolve_path_internal(&new_path, follow_final, depth + 1);
                }
                InodeType::File { .. } => {
                    return Err(VfsError::NotADirectory);
                }
            }
        }

        Ok(current_ino)
    }

    /// Get parent directory inode and entry name
    fn resolve_parent(&self, path: &Path) -> Result<(InodeNumber, String), VfsError> {
        let path_str = path.to_str().ok_or(VfsError::InvalidPath)?;

        if path_str == "/" {
            return Err(VfsError::InvalidPath);
        }

        let components: Vec<&str> = path_str
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        if components.is_empty() {
            return Err(VfsError::InvalidPath);
        }

        let name = components.last().unwrap().to_string();

        if components.len() == 1 {
            return Ok((self.root_ino, name));
        }

        let parent_components = &components[..components.len() - 1];
        let mut current_ino = self.root_ino;

        for component in parent_components {
            let inode = self.inodes.get(&current_ino).ok_or(VfsError::NotFound)?;
            match &inode.inode_type {
                InodeType::Directory { entries } => {
                    current_ino = *entries.get(*component).ok_or(VfsError::NotFound)?;
                }
                InodeType::File { .. } | InodeType::Symlink { .. } => {
                    return Err(VfsError::NotADirectory);
                }
            }
        }

        Ok((current_ino, name))
    }

    /// Create a directory
    pub fn create_directory(&mut self, path: PathBuf) -> Result<(), VfsError> {
        let (parent_ino, name) = self.resolve_parent(&path)?;

        let parent = self
            .inodes
            .get(&parent_ino)
            .ok_or(VfsError::NotFound)?
            .clone();

        let entries = match &parent.inode_type {
            InodeType::Directory { entries } => entries.clone(),
            InodeType::File { .. } | InodeType::Symlink { .. } => {
                return Err(VfsError::NotADirectory)
            }
        };

        if entries.contains_key(&name) {
            return Err(VfsError::AlreadyExists);
        }

        // Create new directory inode
        let new_ino = self.next_ino;
        self.next_ino += 1;

        let new_dir = Inode {
            ino: new_ino,
            inode_type: InodeType::Directory {
                entries: im::HashMap::new(),
            },
            permissions: FilePermissions::default(),
        };

        self.inodes.insert(new_ino, new_dir);

        // Add to parent directory
        let mut new_entries = entries;
        new_entries.insert(name, new_ino);

        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
        };

        self.inodes.insert(parent_ino, updated_parent);

        Ok(())
    }

    /// Remove an empty directory
    pub fn remove_directory(&mut self, path: &Path) -> Result<(), VfsError> {
        if path.as_os_str() == "/" {
            return Err(VfsError::PermissionDenied);
        }

        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?.clone();

        let entries = match &inode.inode_type {
            InodeType::Directory { entries } => entries,
            InodeType::File { .. } | InodeType::Symlink { .. } => {
                return Err(VfsError::NotADirectory)
            }
        };

        if !entries.is_empty() {
            return Err(VfsError::DirectoryNotEmpty);
        }

        // Remove from parent
        let (parent_ino, name) = self.resolve_parent(path)?;
        let parent = self
            .inodes
            .get(&parent_ino)
            .ok_or(VfsError::NotFound)?
            .clone();

        let parent_entries = match &parent.inode_type {
            InodeType::Directory { entries } => entries.clone(),
            InodeType::File { .. } | InodeType::Symlink { .. } => {
                return Err(VfsError::NotADirectory)
            }
        };

        let mut new_entries = parent_entries;
        new_entries.remove(&name);

        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
        };

        self.inodes.insert(parent_ino, updated_parent);
        self.inodes.remove(&ino);

        Ok(())
    }

    /// Check if path is a directory
    pub fn is_directory(&self, path: &Path) -> bool {
        if let Ok(ino) = self.resolve_path(path) {
            if let Some(inode) = self.inodes.get(&ino) {
                return matches!(inode.inode_type, InodeType::Directory { .. });
            }
        }
        false
    }

    /// Check if path is a symbolic link (without following it)
    pub fn is_symlink(&self, path: &Path) -> bool {
        if let Ok(ino) = self.resolve_path_no_follow(path) {
            if let Some(inode) = self.inodes.get(&ino) {
                return matches!(inode.inode_type, InodeType::Symlink { .. });
            }
        }
        false
    }

    /// Create a symbolic link
    pub fn create_symlink(&mut self, link_path: PathBuf, target: PathBuf) -> Result<(), VfsError> {
        let (parent_ino, name) = self.resolve_parent(&link_path)?;

        let parent = self
            .inodes
            .get(&parent_ino)
            .ok_or(VfsError::NotFound)?
            .clone();

        let entries = match &parent.inode_type {
            InodeType::Directory { entries } => entries.clone(),
            InodeType::File { .. } => return Err(VfsError::NotADirectory),
            InodeType::Symlink { .. } => return Err(VfsError::NotADirectory),
        };

        if entries.contains_key(&name) {
            return Err(VfsError::AlreadyExists);
        }

        // Create new symlink inode
        let new_ino = self.next_ino;
        self.next_ino += 1;

        let new_symlink = Inode {
            ino: new_ino,
            inode_type: InodeType::Symlink { target },
            permissions: FilePermissions::default(),
        };

        self.inodes.insert(new_ino, new_symlink);

        // Add to parent directory
        let mut new_entries = entries;
        new_entries.insert(name, new_ino);

        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
        };

        self.inodes.insert(parent_ino, updated_parent);

        Ok(())
    }

    /// Read the target of a symbolic link (without following it)
    pub fn readlink(&self, path: &Path) -> Result<PathBuf, VfsError> {
        let ino = self.resolve_path_no_follow(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        match &inode.inode_type {
            InodeType::Symlink { target } => Ok(target.clone()),
            InodeType::File { .. } | InodeType::Directory { .. } => Err(VfsError::InvalidPath),
        }
    }

    /// Follow a symbolic link to its final target path
    pub fn stat_follow(&self, path: &Path) -> Result<PathBuf, VfsError> {
        self.follow_symlink_to_target(path, 0)
    }

    /// Internal helper to follow symlinks and return the final target path
    fn follow_symlink_to_target(&self, path: &Path, depth: usize) -> Result<PathBuf, VfsError> {
        if depth > Self::MAX_SYMLINK_DEPTH {
            return Err(VfsError::SymlinkLoop);
        }

        let ino = self.resolve_path_no_follow(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        match &inode.inode_type {
            InodeType::Symlink { target } => {
                // Resolve the target (which might be relative or absolute)
                let resolved_target = if target.is_absolute() {
                    target.clone()
                } else {
                    // Relative symlink - resolve relative to parent directory
                    let parent = path.parent().unwrap_or(Path::new("/"));
                    parent.join(target)
                };
                // Recursively follow
                self.follow_symlink_to_target(&resolved_target, depth + 1)
            }
            InodeType::File { .. } | InodeType::Directory { .. } => {
                // Not a symlink, return the path itself
                Ok(path.to_path_buf())
            }
        }
    }

    /// List directory entries
    pub fn list_directory(&self, path: &Path) -> Result<Vec<DirectoryEntry>, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        let entries = match &inode.inode_type {
            InodeType::Directory { entries } => entries,
            InodeType::File { .. } => return Err(VfsError::NotADirectory),
            InodeType::Symlink { .. } => return Err(VfsError::NotADirectory),
        };

        let mut result = Vec::new();
        for (name, &child_ino) in entries.iter() {
            let child_inode = self.inodes.get(&child_ino).ok_or(VfsError::NotFound)?;
            let entry_type = match &child_inode.inode_type {
                InodeType::File { .. } => EntryType::File,
                InodeType::Directory { .. } => EntryType::Directory,
                InodeType::Symlink { .. } => EntryType::Symlink,
            };

            result.push(DirectoryEntry {
                name: name.clone(),
                ino: child_ino,
                entry_type,
            });
        }

        Ok(result)
    }

    /// Create a file
    pub fn create_file(&mut self, path: PathBuf, content: Vec<u8>) -> Result<(), VfsError> {
        let (parent_ino, name) = self.resolve_parent(&path)?;

        let parent = self
            .inodes
            .get(&parent_ino)
            .ok_or(VfsError::NotFound)?
            .clone();

        let entries = match &parent.inode_type {
            InodeType::Directory { entries } => entries.clone(),
            InodeType::File { .. } | InodeType::Symlink { .. } => {
                return Err(VfsError::NotADirectory)
            }
        };

        if entries.contains_key(&name) {
            return Err(VfsError::AlreadyExists);
        }

        // Create new file inode
        let new_ino = self.next_ino;
        self.next_ino += 1;

        let new_file = Inode {
            ino: new_ino,
            inode_type: InodeType::File { content },
            permissions: FilePermissions::default(),
        };

        self.inodes.insert(new_ino, new_file);

        // Add to parent directory
        let mut new_entries = entries;
        new_entries.insert(name, new_ino);

        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
        };

        self.inodes.insert(parent_ino, updated_parent);

        Ok(())
    }

    /// Read file contents
    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        if !inode.permissions.read {
            return Err(VfsError::PermissionDenied);
        }

        match &inode.inode_type {
            InodeType::File { content } => Ok(content.clone()),
            InodeType::Directory { .. } => Err(VfsError::IsADirectory),
            InodeType::Symlink { .. } => {
                // This shouldn't happen since resolve_path follows symlinks
                // but handle it for safety
                Err(VfsError::NotFound)
            }
        }
    }

    /// Write to file (overwrites existing content)
    pub fn write_file(&mut self, path: &Path, content: Vec<u8>) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        if !inode.permissions.write {
            return Err(VfsError::PermissionDenied);
        }

        match &inode.inode_type {
            InodeType::File { .. } => {
                let updated_inode = Inode {
                    ino,
                    inode_type: InodeType::File { content },
                    permissions: inode.permissions.clone(),
                };
                self.inodes.insert(ino, updated_inode);
                Ok(())
            }
            InodeType::Directory { .. } => Err(VfsError::IsADirectory),
            InodeType::Symlink { .. } => Err(VfsError::NotFound),
        }
    }

    /// Check if file exists
    pub fn exists(&self, path: &Path) -> bool {
        self.resolve_path(path).is_ok()
    }

    /// Delete a file
    pub fn delete_file(&mut self, path: &Path) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?.clone();

        // Ensure it's a file, not a directory
        if matches!(inode.inode_type, InodeType::Directory { .. }) {
            return Err(VfsError::IsADirectory);
        }

        // Remove from parent
        let (parent_ino, name) = self.resolve_parent(path)?;
        let parent = self
            .inodes
            .get(&parent_ino)
            .ok_or(VfsError::NotFound)?
            .clone();

        let parent_entries = match &parent.inode_type {
            InodeType::Directory { entries } => entries.clone(),
            InodeType::File { .. } | InodeType::Symlink { .. } => {
                return Err(VfsError::NotADirectory)
            }
        };

        let mut new_entries = parent_entries;
        new_entries.remove(&name);

        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
        };

        self.inodes.insert(parent_ino, updated_parent);
        self.inodes.remove(&ino);

        Ok(())
    }

    /// Get file permissions
    pub fn get_permissions(&self, path: &Path) -> Result<FilePermissions, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(inode.permissions.clone())
    }

    /// Set file permissions
    pub fn set_permissions(
        &mut self,
        path: &Path,
        permissions: FilePermissions,
    ) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        let updated_inode = Inode {
            ino,
            inode_type: inode.inode_type.clone(),
            permissions,
        };

        self.inodes.insert(ino, updated_inode);
        Ok(())
    }

    /// List all files (returns paths) - legacy method for compatibility
    pub fn list_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.collect_files_recursive(&PathBuf::from("/"), self.root_ino, &mut files);
        files
    }

    /// Helper to recursively collect file paths
    fn collect_files_recursive(
        &self,
        current_path: &Path,
        ino: InodeNumber,
        files: &mut Vec<PathBuf>,
    ) {
        if let Some(inode) = self.inodes.get(&ino) {
            match &inode.inode_type {
                InodeType::File { .. } => {
                    files.push(current_path.to_path_buf());
                }
                InodeType::Directory { entries } => {
                    for (name, &child_ino) in entries.iter() {
                        let child_path = if current_path.to_str() == Some("/") {
                            PathBuf::from(format!("/{}", name))
                        } else {
                            PathBuf::from(format!("{}/{}", current_path.display(), name))
                        };
                        self.collect_files_recursive(&child_path, child_ino, files);
                    }
                }
                InodeType::Symlink { .. } => {
                    // Skip symlinks to avoid infinite loops in legacy list_files()
                    // Real implementations would track visited inodes
                }
            }
        }
    }

    /// Get file count - legacy method for compatibility
    pub fn file_count(&self) -> usize {
        self.list_files().len()
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

    // ========================================================================
    // WOS-FS-001: Directory Support Tests (RED PHASE - Should fail initially)
    // ========================================================================

    #[test]
    fn test_create_directory() {
        let mut vfs = VirtualFileSystem::new();
        let dir_path = PathBuf::from("/mydir");

        let result = vfs.create_directory(dir_path.clone());
        assert!(result.is_ok(), "Should create directory successfully");
        assert!(vfs.is_directory(&dir_path), "Path should be a directory");
    }

    #[test]
    fn test_create_nested_directory() {
        let mut vfs = VirtualFileSystem::new();

        // Create parent first
        vfs.create_directory(PathBuf::from("/mydir")).unwrap();
        vfs.create_directory(PathBuf::from("/mydir/subdir"))
            .unwrap();

        assert!(vfs.is_directory(&PathBuf::from("/mydir")));
        assert!(vfs.is_directory(&PathBuf::from("/mydir/subdir")));
    }

    #[test]
    fn test_create_directory_already_exists() {
        let mut vfs = VirtualFileSystem::new();
        let dir_path = PathBuf::from("/mydir");

        vfs.create_directory(dir_path.clone()).unwrap();
        let result = vfs.create_directory(dir_path);

        assert_eq!(
            result,
            Err(VfsError::AlreadyExists),
            "Should fail when directory exists"
        );
    }

    #[test]
    fn test_create_directory_parent_not_found() {
        let mut vfs = VirtualFileSystem::new();
        let nested_path = PathBuf::from("/nonexistent/subdir/docs");

        let result = vfs.create_directory(nested_path);
        assert_eq!(
            result,
            Err(VfsError::NotFound),
            "Should fail when parent doesn't exist"
        );
    }

    #[test]
    fn test_remove_empty_directory() {
        let mut vfs = VirtualFileSystem::new();
        let dir_path = PathBuf::from("/emptydir");

        vfs.create_directory(dir_path.clone()).unwrap();
        let result = vfs.remove_directory(&dir_path);

        assert!(result.is_ok(), "Should remove empty directory");
        assert!(
            !vfs.is_directory(&dir_path),
            "Directory should no longer exist"
        );
    }

    #[test]
    fn test_remove_directory_not_empty() {
        let mut vfs = VirtualFileSystem::new();

        // Create directory with a file
        vfs.create_directory(PathBuf::from("/testdir")).unwrap();
        vfs.create_file(PathBuf::from("/testdir/file.txt"), vec![])
            .unwrap();

        let result = vfs.remove_directory(&PathBuf::from("/testdir"));
        assert!(result.is_err(), "Should fail to remove non-empty directory");
    }

    #[test]
    fn test_remove_directory_not_found() {
        let mut vfs = VirtualFileSystem::new();
        let result = vfs.remove_directory(&PathBuf::from("/nonexistent"));
        assert_eq!(result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_list_directory_entries() {
        let mut vfs = VirtualFileSystem::new();

        // Create directory structure
        vfs.create_directory(PathBuf::from("/listtest")).unwrap();
        vfs.create_file(PathBuf::from("/listtest/file1.txt"), vec![])
            .unwrap();
        vfs.create_file(PathBuf::from("/listtest/file2.txt"), vec![])
            .unwrap();
        vfs.create_directory(PathBuf::from("/listtest/subdir"))
            .unwrap();

        let entries = vfs.list_directory(&PathBuf::from("/listtest")).unwrap();
        assert_eq!(entries.len(), 3, "Should list all entries in directory");

        // Check entries contain expected items
        let entry_names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
        assert!(entry_names.contains(&"file1.txt".to_string()));
        assert!(entry_names.contains(&"file2.txt".to_string()));
        assert!(entry_names.contains(&"subdir".to_string()));
    }

    #[test]
    fn test_list_directory_not_found() {
        let vfs = VirtualFileSystem::new();
        let result = vfs.list_directory(&PathBuf::from("/nonexistent"));
        assert_eq!(result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_create_file_in_subdirectory() {
        let mut vfs = VirtualFileSystem::new();

        // Create directory structure
        vfs.create_directory(PathBuf::from("/mydir")).unwrap();
        vfs.create_directory(PathBuf::from("/mydir/subdir"))
            .unwrap();

        // Create file in subdirectory
        let file_path = PathBuf::from("/mydir/subdir/test.txt");
        let content = b"Hello from subdirectory".to_vec();

        vfs.create_file(file_path.clone(), content.clone()).unwrap();
        let read_content = vfs.read_file(&file_path).unwrap();

        assert_eq!(read_content, content);
    }

    #[test]
    fn test_create_file_parent_directory_missing() {
        let mut vfs = VirtualFileSystem::new();
        let file_path = PathBuf::from("/nonexistent/subdir/test.txt");

        let result = vfs.create_file(file_path, vec![]);
        assert_eq!(
            result,
            Err(VfsError::NotFound),
            "Should fail when parent directory doesn't exist"
        );
    }

    #[test]
    fn test_path_resolution_with_directories() {
        let mut vfs = VirtualFileSystem::new();

        // Create nested structure
        vfs.create_directory(PathBuf::from("/level1")).unwrap();
        vfs.create_directory(PathBuf::from("/level1/level2"))
            .unwrap();
        vfs.create_directory(PathBuf::from("/level1/level2/level3"))
            .unwrap();

        let deep_path = PathBuf::from("/level1/level2/level3/file.txt");
        vfs.create_file(deep_path.clone(), b"Deep file".to_vec())
            .unwrap();

        let content = vfs.read_file(&deep_path).unwrap();
        assert_eq!(content, b"Deep file".to_vec());
    }

    #[test]
    fn test_is_directory_vs_is_file() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_directory(PathBuf::from("/testdir")).unwrap();
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        assert!(
            vfs.is_directory(&PathBuf::from("/")),
            "Root should be directory"
        );
        assert!(
            vfs.is_directory(&PathBuf::from("/testdir")),
            "/testdir should be directory"
        );
        assert!(
            !vfs.is_directory(&PathBuf::from("/file.txt")),
            "/file.txt should not be directory"
        );
    }

    #[test]
    fn test_directory_entry_types() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_directory(PathBuf::from("/typestest")).unwrap();
        vfs.create_file(PathBuf::from("/typestest/file.txt"), vec![])
            .unwrap();
        vfs.create_directory(PathBuf::from("/typestest/subdir"))
            .unwrap();

        let entries = vfs.list_directory(&PathBuf::from("/typestest")).unwrap();

        for entry in entries {
            match entry.name.as_str() {
                "file.txt" => assert!(entry.is_file(), "file.txt should be a file"),
                "subdir" => assert!(entry.is_directory(), "subdir should be a directory"),
                _ => panic!("Unexpected entry: {}", entry.name),
            }
        }
    }

    #[test]
    fn test_root_directory_exists() {
        let vfs = VirtualFileSystem::new();
        assert!(
            vfs.is_directory(&PathBuf::from("/")),
            "Root directory should always exist"
        );
    }

    #[test]
    fn test_delete_file_in_subdirectory() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_directory(PathBuf::from("/deltest")).unwrap();
        let file_path = PathBuf::from("/deltest/file.txt");
        vfs.create_file(file_path.clone(), vec![]).unwrap();

        vfs.delete_file(&file_path).unwrap();
        assert!(!vfs.exists(&file_path), "File should be deleted");

        // Directory should still exist
        assert!(vfs.is_directory(&PathBuf::from("/deltest")));
    }

    #[test]
    fn test_cannot_create_file_over_directory() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_directory(PathBuf::from("/conflictdir")).unwrap();
        let result = vfs.create_file(PathBuf::from("/conflictdir"), vec![]);

        assert_eq!(
            result,
            Err(VfsError::AlreadyExists),
            "Cannot create file over directory"
        );
    }

    #[test]
    fn test_cannot_create_directory_over_file() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();
        let result = vfs.create_directory(PathBuf::from("/file.txt"));

        assert_eq!(
            result,
            Err(VfsError::AlreadyExists),
            "Cannot create directory over file"
        );
    }

    // ========================================================================
    // WOS-FS-002: Symbolic Link Support Tests (RED PHASE)
    // ========================================================================

    #[test]
    fn test_create_symlink() {
        let mut vfs = VirtualFileSystem::new();

        // Create target file
        vfs.create_file(PathBuf::from("/target.txt"), b"content".to_vec())
            .unwrap();

        // Create symlink
        let result = vfs.create_symlink(PathBuf::from("/link.txt"), PathBuf::from("/target.txt"));
        assert!(result.is_ok(), "Should create symlink successfully");
        assert!(
            vfs.is_symlink(&PathBuf::from("/link.txt")),
            "Path should be a symlink"
        );
    }

    #[test]
    fn test_readlink() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/target.txt"), vec![])
            .unwrap();
        vfs.create_symlink(PathBuf::from("/link.txt"), PathBuf::from("/target.txt"))
            .unwrap();

        let target = vfs.readlink(&PathBuf::from("/link.txt")).unwrap();
        assert_eq!(
            target,
            PathBuf::from("/target.txt"),
            "Should return correct target"
        );
    }

    #[test]
    fn test_read_through_symlink() {
        let mut vfs = VirtualFileSystem::new();

        // Create target file with content
        let content = b"Hello via symlink".to_vec();
        vfs.create_file(PathBuf::from("/target.txt"), content.clone())
            .unwrap();

        // Create symlink
        vfs.create_symlink(PathBuf::from("/link.txt"), PathBuf::from("/target.txt"))
            .unwrap();

        // Read through symlink
        let read_content = vfs.read_file(&PathBuf::from("/link.txt")).unwrap();
        assert_eq!(read_content, content, "Should read file through symlink");
    }

    #[test]
    fn test_symlink_to_directory() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_directory(PathBuf::from("/targetdir")).unwrap();
        vfs.create_symlink(PathBuf::from("/linkdir"), PathBuf::from("/targetdir"))
            .unwrap();

        // Should be able to list through symlink
        let result = vfs.list_directory(&PathBuf::from("/linkdir"));
        assert!(result.is_ok(), "Should list directory through symlink");
    }

    #[test]
    fn test_symlink_chain() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();
        vfs.create_symlink(PathBuf::from("/link1"), PathBuf::from("/file.txt"))
            .unwrap();
        vfs.create_symlink(PathBuf::from("/link2"), PathBuf::from("/link1"))
            .unwrap();
        vfs.create_symlink(PathBuf::from("/link3"), PathBuf::from("/link2"))
            .unwrap();

        // Should follow chain
        let content = vfs.read_file(&PathBuf::from("/link3")).unwrap();
        assert_eq!(content, b"data".to_vec(), "Should follow symlink chain");
    }

    #[test]
    fn test_symlink_loop_detection() {
        let mut vfs = VirtualFileSystem::new();

        // Create circular symlinks
        vfs.create_symlink(PathBuf::from("/link1"), PathBuf::from("/link2"))
            .unwrap();
        vfs.create_symlink(PathBuf::from("/link2"), PathBuf::from("/link1"))
            .unwrap();

        // Should detect loop
        let result = vfs.read_file(&PathBuf::from("/link1"));
        assert!(result.is_err(), "Should detect symlink loop");
    }

    #[test]
    fn test_symlink_max_depth() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        // Create very long symlink chain (more than max depth)
        for i in 0..50 {
            let from = PathBuf::from(format!("/link{}", i));
            let to = if i == 0 {
                PathBuf::from("/file.txt")
            } else {
                PathBuf::from(format!("/link{}", i - 1))
            };
            vfs.create_symlink(from, to).unwrap();
        }

        // Should fail with max depth exceeded
        let result = vfs.read_file(&PathBuf::from("/link49"));
        assert!(result.is_err(), "Should enforce max symlink depth");
    }

    #[test]
    fn test_lstat_vs_stat() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();
        vfs.create_symlink(PathBuf::from("/link.txt"), PathBuf::from("/file.txt"))
            .unwrap();

        // lstat should report symlink
        assert!(
            vfs.is_symlink(&PathBuf::from("/link.txt")),
            "lstat should see symlink"
        );

        // stat (following links) should see file
        let target = vfs.stat_follow(&PathBuf::from("/link.txt")).unwrap();
        assert!(!vfs.is_symlink(&target), "stat should follow to target");
    }

    #[test]
    fn test_dangling_symlink() {
        let mut vfs = VirtualFileSystem::new();

        // Create symlink to non-existent target
        vfs.create_symlink(
            PathBuf::from("/link.txt"),
            PathBuf::from("/nonexistent.txt"),
        )
        .unwrap();

        // readlink should work (doesn't follow)
        let target = vfs.readlink(&PathBuf::from("/link.txt")).unwrap();
        assert_eq!(target, PathBuf::from("/nonexistent.txt"));

        // read_file should fail (follows link)
        let result = vfs.read_file(&PathBuf::from("/link.txt"));
        assert_eq!(
            result,
            Err(VfsError::NotFound),
            "Dangling symlink should error when followed"
        );
    }

    #[test]
    fn test_symlink_in_path_resolution() {
        let mut vfs = VirtualFileSystem::new();

        // Create /real/dir/file.txt
        vfs.create_directory(PathBuf::from("/real")).unwrap();
        vfs.create_directory(PathBuf::from("/real/dir")).unwrap();
        vfs.create_file(PathBuf::from("/real/dir/file.txt"), b"data".to_vec())
            .unwrap();

        // Create symlink /linked -> /real
        vfs.create_symlink(PathBuf::from("/linked"), PathBuf::from("/real"))
            .unwrap();

        // Should be able to access /linked/dir/file.txt
        let content = vfs
            .read_file(&PathBuf::from("/linked/dir/file.txt"))
            .unwrap();
        assert_eq!(content, b"data".to_vec(), "Should resolve symlink in path");
    }

    #[test]
    fn test_relative_symlink() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_directory(PathBuf::from("/mydir")).unwrap();
        vfs.create_file(PathBuf::from("/mydir/file.txt"), vec![])
            .unwrap();

        // Create relative symlink
        vfs.create_symlink(PathBuf::from("/mydir/link.txt"), PathBuf::from("file.txt"))
            .unwrap();

        let target = vfs.readlink(&PathBuf::from("/mydir/link.txt")).unwrap();
        assert_eq!(
            target,
            PathBuf::from("file.txt"),
            "Should store relative path"
        );
    }

    // Property-based tests using proptest
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: File count always equals list_files().len()
            #[test]
            fn proptest_file_count_consistency(
                paths in prop::collection::vec(
                    prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
                    0..50
                ),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let mut unique_paths = std::collections::HashSet::new();

                for path_str in paths {
                    let path = PathBuf::from(&path_str);
                    if unique_paths.insert(path_str) {
                        let _ = vfs.create_file(path, vec![]);
                    }
                }

                prop_assert_eq!(vfs.file_count(), vfs.list_files().len());
                prop_assert_eq!(vfs.file_count(), unique_paths.len());
            }

            /// Property: Read after write returns written content
            #[test]
            fn proptest_read_after_write(
                path in prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
                content in prop::collection::vec(any::<u8>(), 0..1024),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(path);

                vfs.create_file(path.clone(), content.clone()).unwrap();
                let read_content = vfs.read_file(&path).unwrap();

                prop_assert_eq!(read_content, content);
            }

            /// Property: Cloning preserves all state
            #[test]
            fn proptest_clone_preserves_state(
                operations in prop::collection::vec(
                    (
                        prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
                        prop::collection::vec(any::<u8>(), 0..256),
                    ),
                    0..30
                ),
            ) {
                let mut vfs = VirtualFileSystem::new();

                for (path_str, content) in operations {
                    let path = PathBuf::from(path_str);
                    let _ = vfs.create_file(path, content);
                }

                let cloned = vfs.clone();
                prop_assert_eq!(vfs.file_count(), cloned.file_count());
                prop_assert_eq!(vfs.list_files(), cloned.list_files());
                prop_assert_eq!(vfs, cloned);
            }

            /// Property: Delete followed by create succeeds
            #[test]
            fn proptest_delete_then_create(
                path in prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
                content1 in prop::collection::vec(any::<u8>(), 0..256),
                content2 in prop::collection::vec(any::<u8>(), 0..256),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(path);

                vfs.create_file(path.clone(), content1).unwrap();
                vfs.delete_file(&path).unwrap();
                let result = vfs.create_file(path.clone(), content2.clone());

                prop_assert!(result.is_ok());
                let read_content = vfs.read_file(&path).unwrap();
                prop_assert_eq!(read_content, content2);
            }

            /// Property: Multiple writes preserve last write
            #[test]
            fn proptest_multiple_writes(
                path in prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
                writes in prop::collection::vec(
                    prop::collection::vec(any::<u8>(), 0..256),
                    1..20
                ),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(path);

                vfs.create_file(path.clone(), vec![]).unwrap();

                let mut last_content = vec![];
                for content in writes {
                    vfs.write_file(&path, content.clone()).unwrap();
                    last_content = content;
                }

                let read_content = vfs.read_file(&path).unwrap();
                prop_assert_eq!(read_content, last_content);
            }

            /// Property: Failed operations don't change file count (atomicity)
            #[test]
            fn proptest_atomicity_on_failure(
                existing_path in prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
                nonexistent_path in prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
            ) {
                prop_assume!(existing_path != nonexistent_path);

                let mut vfs = VirtualFileSystem::new();
                let existing = PathBuf::from(existing_path);
                let nonexistent = PathBuf::from(nonexistent_path);

                vfs.create_file(existing.clone(), vec![]).unwrap();
                let count_before = vfs.file_count();

                // Try operations that should fail
                let _ = vfs.create_file(existing.clone(), vec![]); // AlreadyExists
                let _ = vfs.read_file(&nonexistent); // NotFound
                let _ = vfs.write_file(&nonexistent, vec![]); // NotFound
                let _ = vfs.delete_file(&nonexistent); // NotFound

                prop_assert_eq!(vfs.file_count(), count_before);
            }

            /// Property: Permission changes are idempotent
            #[test]
            fn proptest_permission_idempotent(
                path in prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
                read in any::<bool>(),
                write in any::<bool>(),
                execute in any::<bool>(),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(path);
                let perms = FilePermissions { read, write, execute };

                vfs.create_file(path.clone(), vec![]).unwrap();

                // Set permissions multiple times
                vfs.set_permissions(&path, perms.clone()).unwrap();
                vfs.set_permissions(&path, perms.clone()).unwrap();
                vfs.set_permissions(&path, perms.clone()).unwrap();

                let final_perms = vfs.get_permissions(&path).unwrap();
                prop_assert_eq!(final_perms, perms);
            }

            /// Property: Exists is consistent with list_files
            #[test]
            fn proptest_exists_consistency(
                paths in prop::collection::vec(
                    prop::string::string_regex("/[a-z0-9_]{1,20}\\.txt").unwrap(),
                    0..30
                ),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let mut unique_paths = std::collections::HashSet::new();

                for path_str in &paths {
                    let path = PathBuf::from(path_str);
                    if unique_paths.insert(path_str.clone()) {
                        let _ = vfs.create_file(path, vec![]);
                    }
                }

                let file_list = vfs.list_files();
                for path_str in unique_paths {
                    let path = PathBuf::from(path_str);
                    prop_assert!(vfs.exists(&path));
                    prop_assert!(file_list.contains(&path));
                }
            }

            // ========================================================================
            // WOS-FS-001: Directory Support Property Tests
            // ========================================================================

            /// Property: Directory tree operations are consistent
            #[test]
            fn proptest_directory_tree_consistency(
                dir_names in prop::collection::vec(
                    prop::string::string_regex("[a-z]{1,10}").unwrap(),
                    1..10
                ),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let mut created_dirs = vec![PathBuf::from("/")];

                // Create nested directory structure
                for dir_name in &dir_names {
                    let path = PathBuf::from(format!("/{}", dir_name));
                    if vfs.create_directory(path.clone()).is_ok() {
                        created_dirs.push(path);
                    }
                }

                // All created directories should exist and be directories
                for dir in &created_dirs {
                    prop_assert!(vfs.is_directory(dir), "Created directory should exist: {:?}", dir);
                }
            }

            /// Property: Cannot remove non-empty directory
            #[test]
            fn proptest_cannot_remove_nonempty_directory(
                dir_name in prop::string::string_regex("[a-z]{1,10}").unwrap(),
                file_name in prop::string::string_regex("[a-z]{1,10}\\.txt").unwrap(),
            ) {
                prop_assume!(dir_name != file_name);

                let mut vfs = VirtualFileSystem::new();
                let dir_path = PathBuf::from(format!("/{}", dir_name));
                let file_path = PathBuf::from(format!("/{}/{}", dir_name, file_name));

                vfs.create_directory(dir_path.clone()).unwrap();
                vfs.create_file(file_path, vec![]).unwrap();

                let result = vfs.remove_directory(&dir_path);
                prop_assert!(result.is_err(), "Should not remove non-empty directory");
            }

            /// Property: Directory listing contains all created entries
            #[test]
            fn proptest_directory_listing_complete(
                parent_dir in prop::string::string_regex("[a-z]{1,10}").unwrap(),
                child_names in prop::collection::vec(
                    prop::string::string_regex("[a-z]{1,10}").unwrap(),
                    1..15
                ),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let parent_path = PathBuf::from(format!("/{}", parent_dir));

                vfs.create_directory(parent_path.clone()).unwrap();

                let mut created_count = 0;
                for child_name in &child_names {
                    let child_path = PathBuf::from(format!("/{}/{}", parent_dir, child_name));
                    // Mix of files and directories
                    if created_count % 2 == 0 {
                        if vfs.create_file(child_path, vec![]).is_ok() {
                            created_count += 1;
                        }
                    } else {
                        if vfs.create_directory(child_path).is_ok() {
                            created_count += 1;
                        }
                    }
                }

                let entries = vfs.list_directory(&parent_path).unwrap();
                prop_assert_eq!(entries.len(), created_count, "Directory listing should match created entries");
            }

            /// Property: Path resolution is deterministic
            #[test]
            fn proptest_path_resolution_deterministic(
                dir_path in prop::string::string_regex("[a-z]{1,10}").unwrap(),
                file_name in prop::string::string_regex("[a-z]{1,10}\\.txt").unwrap(),
                content in prop::collection::vec(any::<u8>(), 0..256),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let dir = PathBuf::from(format!("/{}", dir_path));
                let file = PathBuf::from(format!("/{}/{}", dir_path, file_name));

                vfs.create_directory(dir).unwrap();
                vfs.create_file(file.clone(), content.clone()).unwrap();

                // Multiple reads should return same content
                let read1 = vfs.read_file(&file).unwrap();
                let read2 = vfs.read_file(&file).unwrap();
                let read3 = vfs.read_file(&file).unwrap();

                prop_assert_eq!(&read1, &content);
                prop_assert_eq!(&read2, &content);
                prop_assert_eq!(&read3, &content);
            }

            /// Property: Empty directory removal is idempotent
            #[test]
            fn proptest_empty_directory_removal(
                dir_name in prop::string::string_regex("[a-z]{1,10}").unwrap(),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let dir_path = PathBuf::from(format!("/{}", dir_name));

                vfs.create_directory(dir_path.clone()).unwrap();
                prop_assert!(vfs.is_directory(&dir_path));

                vfs.remove_directory(&dir_path).unwrap();
                prop_assert!(!vfs.is_directory(&dir_path));

                // Second removal should fail
                let result = vfs.remove_directory(&dir_path);
                prop_assert!(result.is_err());
            }
        }
    }
}
