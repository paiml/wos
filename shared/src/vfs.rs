//! Virtual File System
//!
//! Persistent data structure-based VFS using im-rs for O(1) cloning.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Unique inode number
type InodeNumber = u64;

/// Timestamp in milliseconds since Unix epoch
type Timestamp = u64;

/// File type for stat
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileType {
    /// Regular file
    RegularFile,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
}

/// File metadata returned by stat
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileStat {
    /// File type
    pub file_type: FileType,
    /// File size in bytes
    pub size: u64,
    /// Access time (atime) - last time file was read
    pub atime: Timestamp,
    /// Modification time (mtime) - last time file content was modified
    pub mtime: Timestamp,
    /// Change time (ctime) - last time metadata was changed
    pub ctime: Timestamp,
    /// Inode number
    pub ino: InodeNumber,
    /// Number of hard links
    pub nlinks: u64,
    /// File mode (permissions)
    pub mode: u32,
    /// Owner UID
    pub uid: u32,
    /// Owner GID
    pub gid: u32,
}

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
    /// Current user ID (for permission checks)
    current_uid: u32,
    /// Current group ID (for permission checks)
    current_gid: u32,
    /// File creation mask (umask)
    umask: u32,
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
    /// Number of hard links to this inode
    pub nlinks: u64,
    /// Access time (atime) - last read
    pub atime: Timestamp,
    /// Modification time (mtime) - last content change
    pub mtime: Timestamp,
    /// Change time (ctime) - last metadata change
    pub ctime: Timestamp,
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

/// Unix-style file permissions
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FilePermissions {
    /// Unix permission mode bits (rwxrwxrwx + special bits)
    pub mode: u32,
    /// Owner user ID
    pub uid: u32,
    /// Owner group ID
    pub gid: u32,
}

impl FilePermissions {
    /// Create read-write permissions (0644 - rw-r--r--)
    pub fn read_write() -> Self {
        Self {
            mode: 0o644,
            uid: 0,
            gid: 0,
        }
    }

    /// Create read-only permissions (0444 - r--r--r--)
    pub fn read_only() -> Self {
        Self {
            mode: 0o444,
            uid: 0,
            gid: 0,
        }
    }

    /// Create with specific mode, uid, gid
    pub fn new(mode: u32, uid: u32, gid: u32) -> Self {
        Self { mode, uid, gid }
    }

    /// Check if mode has read permission for owner
    pub fn owner_can_read(&self) -> bool {
        (self.mode & 0o400) != 0
    }

    /// Check if mode has write permission for owner
    pub fn owner_can_write(&self) -> bool {
        (self.mode & 0o200) != 0
    }

    /// Check if mode has execute permission for owner
    pub fn owner_can_execute(&self) -> bool {
        (self.mode & 0o100) != 0
    }

    /// Check if mode has read permission for group
    pub fn group_can_read(&self) -> bool {
        (self.mode & 0o040) != 0
    }

    /// Check if mode has write permission for group
    pub fn group_can_write(&self) -> bool {
        (self.mode & 0o020) != 0
    }

    /// Check if mode has execute permission for group
    pub fn group_can_execute(&self) -> bool {
        (self.mode & 0o010) != 0
    }

    /// Check if mode has read permission for others
    pub fn other_can_read(&self) -> bool {
        (self.mode & 0o004) != 0
    }

    /// Check if mode has write permission for others
    pub fn other_can_write(&self) -> bool {
        (self.mode & 0o002) != 0
    }

    /// Check if mode has execute permission for others
    pub fn other_can_execute(&self) -> bool {
        (self.mode & 0o001) != 0
    }

    /// Check if setuid bit is set
    pub fn has_setuid(&self) -> bool {
        (self.mode & 0o4000) != 0
    }

    /// Check if setgid bit is set
    pub fn has_setgid(&self) -> bool {
        (self.mode & 0o2000) != 0
    }

    /// Check if sticky bit is set
    pub fn has_sticky(&self) -> bool {
        (self.mode & 0o1000) != 0
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

/// Helper function to get current timestamp
fn current_timestamp() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

impl VirtualFileSystem {
    /// Create a new VFS with root directory and standard directory structure
    pub fn new() -> Self {
        let root_ino = 1;
        let mut inodes = im::HashMap::new();

        // Create root directory with proper permissions (0755 - rwxr-xr-x)
        let now = current_timestamp();
        let root = Inode {
            ino: root_ino,
            inode_type: InodeType::Directory {
                entries: im::HashMap::new(),
            },
            permissions: FilePermissions::new(0o755, 0, 0), // Root owned, world-readable/executable
            nlinks: 1,
            atime: now,
            mtime: now,
            ctime: now,
        };
        inodes.insert(root_ino, root);

        let mut vfs = Self {
            inodes,
            root_ino,
            next_ino: 2,
            cwd: PathBuf::from("/"),
            current_uid: 0, // Default to root
            current_gid: 0, // Default to root group
            umask: 0o022,   // Default umask (rw-r--r-- for files, rwxr-xr-x for dirs)
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

    /// Create a new VFS with specific user context
    pub fn new_with_context(uid: u32, gid: u32) -> Self {
        let mut vfs = Self::new();
        vfs.current_uid = uid;
        vfs.current_gid = gid;
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
                    // Check execute permission on directory (needed to traverse)
                    if !self.check_permission(inode, self.current_uid, self.current_gid, 1) {
                        return Err(VfsError::PermissionDenied);
                    }

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

        // Create permissions with current user context and umask applied
        let mode = 0o777 & !self.umask; // Default directory mode 0777 with umask applied

        // If parent directory has setgid bit, inherit its GID
        let gid = if parent.permissions.has_setgid() {
            parent.permissions.gid
        } else {
            self.current_gid
        };

        let permissions = FilePermissions::new(mode, self.current_uid, gid);

        let now = current_timestamp();
        let new_dir = Inode {
            ino: new_ino,
            inode_type: InodeType::Directory {
                entries: im::HashMap::new(),
            },
            permissions,
            nlinks: 1,
            atime: now,
            mtime: now,
            ctime: now,
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
            nlinks: parent.nlinks,
            atime: parent.atime,
            mtime: now, // Parent's mtime changes when entry added
            ctime: now, // Parent's ctime changes when entry added
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

        let now = current_timestamp();
        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
            nlinks: parent.nlinks,
            atime: parent.atime,
            mtime: now, // Parent's mtime changes when entry removed
            ctime: now, // Parent's ctime changes when entry removed
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

        // Symlinks typically have 0777 permissions (permissions are checked on target)
        let permissions = FilePermissions::new(0o777, self.current_uid, self.current_gid);

        let now = current_timestamp();
        let new_symlink = Inode {
            ino: new_ino,
            inode_type: InodeType::Symlink { target },
            permissions,
            nlinks: 1,
            atime: now,
            mtime: now,
            ctime: now,
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
            nlinks: parent.nlinks,
            atime: parent.atime,
            mtime: now, // Parent's mtime changes when entry added
            ctime: now, // Parent's ctime changes when entry added
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

        // Create permissions with current user context and umask applied
        let mode = 0o666 & !self.umask; // Default file mode 0666 with umask applied

        // If parent directory has setgid bit, inherit its GID
        let gid = if parent.permissions.has_setgid() {
            parent.permissions.gid
        } else {
            self.current_gid
        };

        let permissions = FilePermissions::new(mode, self.current_uid, gid);

        let now = current_timestamp();
        let new_file = Inode {
            ino: new_ino,
            inode_type: InodeType::File { content },
            permissions,
            nlinks: 1,
            atime: now,
            mtime: now,
            ctime: now,
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
            nlinks: parent.nlinks,
            atime: parent.atime,
            mtime: now, // Parent's mtime changes when entry added
            ctime: now, // Parent's ctime changes when entry added
        };

        self.inodes.insert(parent_ino, updated_parent);

        Ok(())
    }

    /// Read file contents (uses current user context)
    pub fn read_file(&mut self, path: &Path) -> Result<Vec<u8>, VfsError> {
        let uid = self.current_uid;
        let gid = self.current_gid;
        self.read_file_as(path, uid, gid)
    }

    /// Write to file (overwrites existing content, uses current user context)
    pub fn write_file(&mut self, path: &Path, content: Vec<u8>) -> Result<(), VfsError> {
        let uid = self.current_uid;
        let gid = self.current_gid;
        self.write_file_as(path, content, uid, gid)
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

        let now = current_timestamp();
        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
            nlinks: parent.nlinks,
            atime: parent.atime,
            mtime: now, // Parent's mtime changes when entry removed
            ctime: now, // Parent's ctime changes when entry removed
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

        let now = current_timestamp();
        let updated_inode = Inode {
            ino,
            inode_type: inode.inode_type.clone(),
            permissions,
            nlinks: inode.nlinks,
            atime: inode.atime,
            mtime: inode.mtime,
            ctime: now, // ctime changes when permissions change
        };

        self.inodes.insert(ino, updated_inode);
        Ok(())
    }

    /// Create a hard link (multiple directory entries pointing to same inode)
    pub fn link(&mut self, old_path: PathBuf, new_path: PathBuf) -> Result<(), VfsError> {
        // Get the inode of the existing file
        let old_ino = self.resolve_path(&old_path)?;
        let inode = self.inodes.get(&old_ino).ok_or(VfsError::NotFound)?;

        // Check if it's a directory (POSIX restriction: no hard links to directories)
        if matches!(inode.inode_type, InodeType::Directory { .. }) {
            return Err(VfsError::PermissionDenied);
        }

        // Get parent directory for new path
        let (parent_ino, name) = self.resolve_parent(&new_path)?;
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

        // Add new directory entry pointing to existing inode
        let mut new_entries = entries;
        new_entries.insert(name, old_ino);

        let now = current_timestamp();
        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
            nlinks: parent.nlinks,
            atime: parent.atime,
            mtime: now, // Parent's mtime changes when entry added
            ctime: now, // Parent's ctime changes when entry added
        };

        self.inodes.insert(parent_ino, updated_parent);

        // Increment the link count on the inode
        let inode = self.inodes.get(&old_ino).ok_or(VfsError::NotFound)?.clone();
        let updated_inode = Inode {
            ino: old_ino,
            inode_type: inode.inode_type.clone(),
            permissions: inode.permissions.clone(),
            nlinks: inode.nlinks + 1,
            atime: inode.atime,
            mtime: inode.mtime,
            ctime: inode.ctime, // ctime doesn't change for hard links (only nlinks metadata changed)
        };
        self.inodes.insert(old_ino, updated_inode);

        Ok(())
    }

    /// Remove a directory entry (decrement reference count, delete inode if refcount reaches 0)
    pub fn unlink(&mut self, path: &Path) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?.clone();

        // Get parent and remove directory entry
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

        let now = current_timestamp();
        let updated_parent = Inode {
            ino: parent_ino,
            inode_type: InodeType::Directory {
                entries: new_entries,
            },
            permissions: parent.permissions,
            nlinks: parent.nlinks,
            atime: parent.atime,
            mtime: now, // Parent's mtime changes when entry removed
            ctime: now, // Parent's ctime changes when entry removed
        };

        self.inodes.insert(parent_ino, updated_parent);

        // Decrement link count
        let new_nlinks = inode.nlinks - 1;

        if new_nlinks == 0 {
            // Last link removed - delete the inode
            self.inodes.remove(&ino);
        } else {
            // Still has links - just decrement count
            let updated_inode = Inode {
                ino,
                inode_type: inode.inode_type.clone(),
                permissions: inode.permissions.clone(),
                nlinks: new_nlinks,
                atime: inode.atime,
                mtime: inode.mtime,
                ctime: inode.ctime, // ctime doesn't change for hard links (only nlinks metadata changed)
            };
            self.inodes.insert(ino, updated_inode);
        }

        Ok(())
    }

    /// Get the link count (number of hard links) for a file
    pub fn get_link_count(&self, path: &Path) -> Result<u64, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(inode.nlinks)
    }

    /// Get the inode number for a path
    pub fn get_inode_number(&self, path: &Path) -> Result<InodeNumber, VfsError> {
        self.resolve_path(path)
    }

    /// Check if an inode exists
    pub fn inode_exists(&self, ino: InodeNumber) -> bool {
        self.inodes.contains_key(&ino)
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

    // ========== Unix-style Permission Methods ==========

    /// Check if a user has specific permission on a file
    fn check_permission(&self, inode: &Inode, uid: u32, gid: u32, mode_bit: u32) -> bool {
        // Root bypasses all permission checks
        if uid == 0 {
            return true;
        }

        let perms = &inode.permissions;

        // Check owner permissions
        if uid == perms.uid {
            return (perms.mode & (mode_bit << 6)) != 0;
        }

        // Check group permissions
        if gid == perms.gid {
            return (perms.mode & (mode_bit << 3)) != 0;
        }

        // Check other permissions
        (perms.mode & mode_bit) != 0
    }

    /// Change file mode (permissions)
    pub fn chmod(&mut self, path: &Path, mode: u32) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?.clone();

        // Only owner or root can change mode
        if self.current_uid != 0 && self.current_uid != inode.permissions.uid {
            return Err(VfsError::PermissionDenied);
        }

        let mut new_perms = inode.permissions.clone();
        new_perms.mode = mode;

        let now = current_timestamp();
        let updated_inode = Inode {
            ino,
            inode_type: inode.inode_type.clone(),
            permissions: new_perms,
            nlinks: inode.nlinks,
            atime: inode.atime,
            mtime: inode.mtime,
            ctime: now, // ctime changes when permissions change
        };

        self.inodes.insert(ino, updated_inode);
        Ok(())
    }

    /// Change file owner (uid and/or gid)
    pub fn chown(
        &mut self,
        path: &Path,
        uid: Option<u32>,
        gid: Option<u32>,
    ) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?.clone();

        // Only root can change ownership
        // (In a real system, owner can change group if they're a member)
        if self.current_uid != 0 {
            return Err(VfsError::PermissionDenied);
        }

        let mut new_perms = inode.permissions.clone();
        if let Some(new_uid) = uid {
            new_perms.uid = new_uid;
        }
        if let Some(new_gid) = gid {
            new_perms.gid = new_gid;
        }

        let now = current_timestamp();
        let updated_inode = Inode {
            ino,
            inode_type: inode.inode_type.clone(),
            permissions: new_perms,
            nlinks: inode.nlinks,
            atime: inode.atime,
            mtime: inode.mtime,
            ctime: now, // ctime changes when owner changes
        };

        self.inodes.insert(ino, updated_inode);
        Ok(())
    }

    /// Get file mode
    pub fn get_mode(&self, path: &Path) -> Result<u32, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(inode.permissions.mode)
    }

    /// Get file owner UID
    pub fn get_owner(&self, path: &Path) -> Result<u32, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(inode.permissions.uid)
    }

    /// Get file group GID
    pub fn get_group(&self, path: &Path) -> Result<u32, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(inode.permissions.gid)
    }

    /// Read file as specific user (permission check)
    pub fn read_file_as(&mut self, path: &Path, uid: u32, gid: u32) -> Result<Vec<u8>, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?.clone();

        // Check read permission (mode_bit 4 = read)
        if !self.check_permission(&inode, uid, gid, 4) {
            return Err(VfsError::PermissionDenied);
        }

        // Update atime
        let now = current_timestamp();
        let updated_inode = Inode {
            ino,
            inode_type: inode.inode_type.clone(),
            permissions: inode.permissions.clone(),
            nlinks: inode.nlinks,
            atime: now,           // Update on read
            mtime: inode.mtime,   // Preserve mtime
            ctime: inode.ctime,   // Preserve ctime
        };
        self.inodes.insert(ino, updated_inode);

        match &inode.inode_type {
            InodeType::File { content } => Ok(content.clone()),
            InodeType::Directory { .. } => Err(VfsError::IsADirectory),
            InodeType::Symlink { .. } => Err(VfsError::NotFound),
        }
    }

    /// Write file as specific user (permission check)
    pub fn write_file_as(
        &mut self,
        path: &Path,
        content: Vec<u8>,
        uid: u32,
        gid: u32,
    ) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?.clone();

        // Check write permission (mode_bit 2 = write)
        if !self.check_permission(&inode, uid, gid, 2) {
            return Err(VfsError::PermissionDenied);
        }

        match &inode.inode_type {
            InodeType::File { .. } => {
                let now = current_timestamp();
                let updated_inode = Inode {
                    ino,
                    inode_type: InodeType::File { content },
                    permissions: inode.permissions.clone(),
                    nlinks: inode.nlinks,
                    atime: inode.atime,
                    mtime: now, // mtime changes when file content is written
                    ctime: now, // ctime changes when file content is written
                };
                self.inodes.insert(ino, updated_inode);
                Ok(())
            }
            InodeType::Directory { .. } => Err(VfsError::IsADirectory),
            InodeType::Symlink { .. } => Err(VfsError::NotFound),
        }
    }

    /// Check if file has execute permission for current user
    pub fn can_execute(&self, path: &Path) -> Result<bool, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        // Check execute permission (mode_bit 1 = execute)
        Ok(self.check_permission(inode, self.current_uid, self.current_gid, 1))
    }

    /// Check if file has setuid bit set
    pub fn is_setuid(&self, path: &Path) -> Result<bool, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(inode.permissions.has_setuid())
    }

    /// Check if file has setgid bit set
    pub fn is_setgid(&self, path: &Path) -> Result<bool, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(inode.permissions.has_setgid())
    }

    /// Check if file has sticky bit set
    pub fn is_sticky(&self, path: &Path) -> Result<bool, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(inode.permissions.has_sticky())
    }

    /// Set umask (file creation mask)
    pub fn set_umask(&mut self, umask: u32) {
        self.umask = umask & 0o777; // Only keep permission bits
    }

    /// Get effective UID after setuid execution
    pub fn get_effective_uid(&self, path: &Path, uid: u32) -> Result<u32, VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        if inode.permissions.has_setuid() {
            // When setuid is set, effective UID becomes file owner's UID
            Ok(inode.permissions.uid)
        } else {
            // Otherwise, effective UID is the same as real UID
            Ok(uid)
        }
    }

    /// Set current user context (for switching users, e.g., su/sudo)
    pub fn set_context(&mut self, uid: u32, gid: u32) {
        self.current_uid = uid;
        self.current_gid = gid;
    }

    // ========== File Metadata Methods ==========

    /// Get file metadata (stat) - follows symlinks
    pub fn stat(&self, path: &Path) -> Result<FileStat, VfsError> {
        let ino = self.resolve_path(path)?; // follows symlinks
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(self.inode_to_filestat(inode))
    }

    /// Get file metadata (lstat) - does NOT follow symlinks
    pub fn lstat(&self, path: &Path) -> Result<FileStat, VfsError> {
        let ino = self.resolve_path_no_follow(path)?; // doesn't follow final symlink
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(self.inode_to_filestat(inode))
    }

    /// Helper to convert Inode to FileStat
    fn inode_to_filestat(&self, inode: &Inode) -> FileStat {
        let (file_type, size) = match &inode.inode_type {
            InodeType::File { content } => (FileType::RegularFile, content.len() as u64),
            InodeType::Directory { .. } => (FileType::Directory, 4096), // Fixed size for directories
            InodeType::Symlink { .. } => (FileType::Symlink, 0), // Symlinks have size 0
        };

        FileStat {
            file_type,
            size,
            atime: inode.atime,
            mtime: inode.mtime,
            ctime: inode.ctime,
            ino: inode.ino,
            nlinks: inode.nlinks,
            mode: inode.permissions.mode,
            uid: inode.permissions.uid,
            gid: inode.permissions.gid,
        }
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
        let mut vfs = VirtualFileSystem::new();
        assert_eq!(vfs.cwd(), &PathBuf::from("/"));
        assert_eq!(vfs.file_count(), 0);
    }

    #[test]
    fn test_vfs_clone_cheap() {
        let mut vfs = VirtualFileSystem::new();
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
        let mut vfs = VirtualFileSystem::new();
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

        // Default permissions should be read-write (0644)
        let perms = vfs.get_permissions(&path).unwrap();
        assert!(perms.owner_can_read());
        assert!(perms.owner_can_write());
        assert!(!perms.owner_can_execute());

        // Change to read-only (0444)
        vfs.set_permissions(&path, FilePermissions::read_only())
            .unwrap();
        let perms = vfs.get_permissions(&path).unwrap();
        assert!(perms.owner_can_read());
        assert!(!perms.owner_can_write());
    }

    #[test]
    fn test_read_permission_denied() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000); // Non-root user
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), b"Secret".to_vec()).unwrap();

        // Set to no read permission (write-only: 0200)
        let no_read = FilePermissions::new(0o200, 1000, 1000);
        vfs.set_permissions(&path, no_read).unwrap();

        let result = vfs.read_file(&path);
        assert_eq!(result, Err(VfsError::PermissionDenied));
    }

    #[test]
    fn test_write_permission_denied() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000); // Non-root user
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), vec![]).unwrap();

        // Set to read-only (0444)
        vfs.set_permissions(&path, FilePermissions::new(0o444, 1000, 1000))
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
        let mut vfs = VirtualFileSystem::new();
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
        let mut vfs = VirtualFileSystem::new();
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

    // ========================================================================
    // WOS-FS-003: Hard Links and Inode Reference Counting Tests (RED PHASE)
    // ========================================================================

    #[test]
    fn test_create_hard_link() {
        let mut vfs = VirtualFileSystem::new();

        // Create original file
        vfs.create_file(PathBuf::from("/original.txt"), b"content".to_vec())
            .unwrap();

        // Create hard link
        let result = vfs.link(
            PathBuf::from("/original.txt"),
            PathBuf::from("/hardlink.txt"),
        );
        assert!(result.is_ok(), "Should create hard link successfully");

        // Both paths should exist
        assert!(vfs.exists(&PathBuf::from("/original.txt")));
        assert!(vfs.exists(&PathBuf::from("/hardlink.txt")));
    }

    #[test]
    fn test_hard_link_shares_inode() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file1.txt"), vec![])
            .unwrap();
        vfs.link(PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"))
            .unwrap();

        // Both should point to the same inode
        let inode1 = vfs.get_inode_number(&PathBuf::from("/file1.txt")).unwrap();
        let inode2 = vfs.get_inode_number(&PathBuf::from("/file2.txt")).unwrap();
        assert_eq!(inode1, inode2, "Hard links should share the same inode");
    }

    #[test]
    fn test_hard_link_shares_content() {
        let mut vfs = VirtualFileSystem::new();

        let content = b"shared content".to_vec();
        vfs.create_file(PathBuf::from("/file1.txt"), content.clone())
            .unwrap();
        vfs.link(PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"))
            .unwrap();

        // Content should be the same
        let content1 = vfs.read_file(&PathBuf::from("/file1.txt")).unwrap();
        let content2 = vfs.read_file(&PathBuf::from("/file2.txt")).unwrap();
        assert_eq!(content1, content2, "Hard links should share content");
        assert_eq!(content1, content, "Content should match original");
    }

    #[test]
    fn test_write_through_hard_link() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file1.txt"), b"initial".to_vec())
            .unwrap();
        vfs.link(PathBuf::from("/file1.txt"), PathBuf::from("/file2.txt"))
            .unwrap();

        // Write through one link
        vfs.write_file(&PathBuf::from("/file2.txt"), b"updated".to_vec())
            .unwrap();

        // Read through the other link
        let content = vfs.read_file(&PathBuf::from("/file1.txt")).unwrap();
        assert_eq!(
            content,
            b"updated".to_vec(),
            "Write through one hard link should be visible in others"
        );
    }

    #[test]
    fn test_unlink_decrements_refcount() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();
        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/link.txt"))
            .unwrap();

        // Initial link count should be 2
        assert_eq!(
            vfs.get_link_count(&PathBuf::from("/file.txt")).unwrap(),
            2,
            "Link count should be 2 after creating hard link"
        );

        // Unlink one
        vfs.unlink(&PathBuf::from("/link.txt")).unwrap();

        // Link count should be 1
        assert_eq!(
            vfs.get_link_count(&PathBuf::from("/file.txt")).unwrap(),
            1,
            "Link count should be 1 after unlinking"
        );
    }

    #[test]
    fn test_file_persists_with_links() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();
        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/link.txt"))
            .unwrap();

        // Unlink original
        vfs.unlink(&PathBuf::from("/file.txt")).unwrap();

        // Link should still exist and be readable
        assert!(vfs.exists(&PathBuf::from("/link.txt")));
        let content = vfs.read_file(&PathBuf::from("/link.txt")).unwrap();
        assert_eq!(
            content,
            b"data".to_vec(),
            "File should persist through link"
        );
    }

    #[test]
    fn test_file_deleted_when_refcount_zero() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();
        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/link.txt"))
            .unwrap();

        let inode_num = vfs.get_inode_number(&PathBuf::from("/file.txt")).unwrap();

        // Unlink both
        vfs.unlink(&PathBuf::from("/file.txt")).unwrap();
        vfs.unlink(&PathBuf::from("/link.txt")).unwrap();

        // Inode should be freed
        assert!(
            !vfs.inode_exists(inode_num),
            "Inode should be freed when refcount reaches 0"
        );
    }

    #[test]
    fn test_hard_link_to_nonexistent() {
        let mut vfs = VirtualFileSystem::new();

        let result = vfs.link(
            PathBuf::from("/nonexistent.txt"),
            PathBuf::from("/link.txt"),
        );
        assert_eq!(
            result,
            Err(VfsError::NotFound),
            "Hard link to non-existent file should fail"
        );
    }

    #[test]
    fn test_hard_link_to_directory() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_directory(PathBuf::from("/dir")).unwrap();

        let result = vfs.link(PathBuf::from("/dir"), PathBuf::from("/dirlink"));
        assert!(
            result.is_err(),
            "Hard link to directory should fail (POSIX restriction)"
        );
    }

    #[test]
    fn test_get_link_count() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        // Initial link count should be 1
        assert_eq!(vfs.get_link_count(&PathBuf::from("/file.txt")).unwrap(), 1);

        // Add hard links
        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/link1.txt"))
            .unwrap();
        assert_eq!(vfs.get_link_count(&PathBuf::from("/file.txt")).unwrap(), 2);

        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/link2.txt"))
            .unwrap();
        assert_eq!(vfs.get_link_count(&PathBuf::from("/file.txt")).unwrap(), 3);
    }

    // Integration tests (more complex scenarios)

    #[test]
    fn test_hard_link_complex_scenario() {
        let mut vfs = VirtualFileSystem::new();

        // Create file with multiple hard links in different directories
        vfs.create_directory(PathBuf::from("/dir1")).unwrap();
        vfs.create_directory(PathBuf::from("/dir2")).unwrap();

        vfs.create_file(PathBuf::from("/dir1/file.txt"), b"data".to_vec())
            .unwrap();
        vfs.link(
            PathBuf::from("/dir1/file.txt"),
            PathBuf::from("/dir2/file.txt"),
        )
        .unwrap();
        vfs.link(PathBuf::from("/dir1/file.txt"), PathBuf::from("/root.txt"))
            .unwrap();

        // All should share content
        assert_eq!(
            vfs.read_file(&PathBuf::from("/dir1/file.txt")).unwrap(),
            b"data".to_vec()
        );
        assert_eq!(
            vfs.read_file(&PathBuf::from("/dir2/file.txt")).unwrap(),
            b"data".to_vec()
        );
        assert_eq!(
            vfs.read_file(&PathBuf::from("/root.txt")).unwrap(),
            b"data".to_vec()
        );

        // Link count should be 3
        assert_eq!(
            vfs.get_link_count(&PathBuf::from("/dir1/file.txt"))
                .unwrap(),
            3
        );
    }

    #[test]
    fn test_unlink_vs_delete_file() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();
        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/link.txt"))
            .unwrap();

        // unlink() should work with hard links
        vfs.unlink(&PathBuf::from("/file.txt")).unwrap();
        assert!(vfs.exists(&PathBuf::from("/link.txt")));

        // delete_file() should be equivalent to unlink() for files
        vfs.unlink(&PathBuf::from("/link.txt")).unwrap();
        assert!(!vfs.exists(&PathBuf::from("/link.txt")));
    }

    #[test]
    fn test_hard_link_preserves_permissions() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();
        vfs.set_permissions(&PathBuf::from("/file.txt"), FilePermissions::read_only())
            .unwrap();

        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/link.txt"))
            .unwrap();

        // Permissions should be shared (same inode)
        let perms = vfs.get_permissions(&PathBuf::from("/link.txt")).unwrap();
        assert!(perms.owner_can_read());
        assert!(!perms.owner_can_write());
    }

    #[test]
    fn test_hard_link_and_symlink_combined() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();
        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/hardlink.txt"))
            .unwrap();
        vfs.create_symlink(
            PathBuf::from("/symlink.txt"),
            PathBuf::from("/hardlink.txt"),
        )
        .unwrap();

        // Read through symlink -> hard link -> file
        let content = vfs.read_file(&PathBuf::from("/symlink.txt")).unwrap();
        assert_eq!(content, b"data".to_vec());
    }

    #[test]
    fn test_link_count_after_directory_operations() {
        let mut vfs = VirtualFileSystem::new();

        vfs.create_directory(PathBuf::from("/dir")).unwrap();
        vfs.create_file(PathBuf::from("/dir/file.txt"), vec![])
            .unwrap();
        vfs.link(PathBuf::from("/dir/file.txt"), PathBuf::from("/link.txt"))
            .unwrap();

        assert_eq!(
            vfs.get_link_count(&PathBuf::from("/dir/file.txt")).unwrap(),
            2
        );

        // Moving/renaming not implemented yet, but unlink should work
        vfs.unlink(&PathBuf::from("/dir/file.txt")).unwrap();
        assert_eq!(vfs.get_link_count(&PathBuf::from("/link.txt")).unwrap(), 1);
    }

    // ========================================================================
    // WOS-FS-004: File Permissions and Ownership Tests (RED PHASE)
    // ========================================================================

    #[test]
    fn test_chmod_changes_mode() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        // Change to read-only for owner, no access for group/other (0400)
        vfs.chmod(&PathBuf::from("/file.txt"), 0o400).unwrap();

        let mode = vfs.get_mode(&PathBuf::from("/file.txt")).unwrap();
        assert_eq!(mode & 0o777, 0o400, "Mode should be 0400");
    }

    #[test]
    fn test_chmod_octal_notation() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        // Test various octal permissions
        vfs.chmod(&PathBuf::from("/file.txt"), 0o755).unwrap();
        assert_eq!(
            vfs.get_mode(&PathBuf::from("/file.txt")).unwrap() & 0o777,
            0o755
        );

        vfs.chmod(&PathBuf::from("/file.txt"), 0o644).unwrap();
        assert_eq!(
            vfs.get_mode(&PathBuf::from("/file.txt")).unwrap() & 0o777,
            0o644
        );
    }

    #[test]
    fn test_chown_changes_owner() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000); // Create as UID 1000
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        // Switch to root to change ownership (only root can chown)
        vfs.set_context(0, 0);

        // Change owner to UID 1001
        vfs.chown(&PathBuf::from("/file.txt"), Some(1001), None)
            .unwrap();

        let uid = vfs.get_owner(&PathBuf::from("/file.txt")).unwrap();
        assert_eq!(uid, 1001, "UID should be 1001");
    }

    #[test]
    fn test_chgrp_changes_group() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        // Switch to root to change group (only root can chown/chgrp)
        vfs.set_context(0, 0);

        // Change group to GID 1001
        vfs.chown(&PathBuf::from("/file.txt"), None, Some(1001))
            .unwrap();

        let gid = vfs.get_group(&PathBuf::from("/file.txt")).unwrap();
        assert_eq!(gid, 1001, "GID should be 1001");
    }

    #[test]
    fn test_read_permission_check() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        // Remove read permission for owner
        vfs.chmod(&PathBuf::from("/file.txt"), 0o200).unwrap(); // Write-only

        let result = vfs.read_file(&PathBuf::from("/file.txt"));
        assert_eq!(
            result,
            Err(VfsError::PermissionDenied),
            "Read should be denied"
        );
    }

    #[test]
    fn test_write_permission_check() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        // Remove write permission for owner
        vfs.chmod(&PathBuf::from("/file.txt"), 0o444).unwrap(); // Read-only

        let result = vfs.write_file(&PathBuf::from("/file.txt"), b"new".to_vec());
        assert_eq!(
            result,
            Err(VfsError::PermissionDenied),
            "Write should be denied"
        );
    }

    #[test]
    fn test_execute_permission_check() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/script.sh"), b"#!/bin/sh".to_vec())
            .unwrap();

        // Remove execute permission
        vfs.chmod(&PathBuf::from("/script.sh"), 0o644).unwrap();

        let result = vfs.can_execute(&PathBuf::from("/script.sh")).unwrap();
        assert!(!result, "Execute should be denied");

        // Add execute permission
        vfs.chmod(&PathBuf::from("/script.sh"), 0o755).unwrap();
        assert!(
            vfs.can_execute(&PathBuf::from("/script.sh")).unwrap(),
            "Execute should be allowed"
        );
    }

    #[test]
    fn test_owner_permissions_vs_other() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        // Owner can read (0400), others cannot
        vfs.chmod(&PathBuf::from("/file.txt"), 0o400).unwrap();

        // Owner (UID 1000) can read
        let result = vfs.read_file_as(&PathBuf::from("/file.txt"), 1000, 1000);
        assert!(result.is_ok(), "Owner should be able to read");

        // Other (UID 1001) cannot read
        let result = vfs.read_file_as(&PathBuf::from("/file.txt"), 1001, 1001);
        assert_eq!(
            result,
            Err(VfsError::PermissionDenied),
            "Other should not read"
        );
    }

    #[test]
    fn test_group_permissions() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        // Group can read (0040), owner and other cannot
        vfs.chmod(&PathBuf::from("/file.txt"), 0o040).unwrap();

        // Group member (same GID) can read
        let result = vfs.read_file_as(&PathBuf::from("/file.txt"), 1001, 1000);
        assert!(result.is_ok(), "Group member should be able to read");

        // Non-group member cannot read
        let result = vfs.read_file_as(&PathBuf::from("/file.txt"), 1001, 1001);
        assert_eq!(
            result,
            Err(VfsError::PermissionDenied),
            "Non-group cannot read"
        );
    }

    #[test]
    fn test_root_bypasses_permissions() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        // No permissions for anyone (0000)
        vfs.chmod(&PathBuf::from("/file.txt"), 0o000).unwrap();

        // Root (UID 0) can still read
        let result = vfs.read_file_as(&PathBuf::from("/file.txt"), 0, 0);
        assert!(result.is_ok(), "Root should bypass permission checks");

        // Non-root cannot read
        let result = vfs.read_file_as(&PathBuf::from("/file.txt"), 1000, 1000);
        assert_eq!(
            result,
            Err(VfsError::PermissionDenied),
            "Non-root should be denied"
        );
    }

    #[test]
    fn test_permission_denied_on_read() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/secret.txt"), b"classified".to_vec())
            .unwrap();

        vfs.chmod(&PathBuf::from("/secret.txt"), 0o000).unwrap();

        let result = vfs.read_file(&PathBuf::from("/secret.txt"));
        assert_eq!(result, Err(VfsError::PermissionDenied));
    }

    #[test]
    fn test_permission_denied_on_write() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/readonly.txt"), vec![])
            .unwrap();

        vfs.chmod(&PathBuf::from("/readonly.txt"), 0o444).unwrap();

        let result = vfs.write_file(&PathBuf::from("/readonly.txt"), b"try to write".to_vec());
        assert_eq!(result, Err(VfsError::PermissionDenied));
    }

    #[test]
    fn test_setuid_bit() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/setuid_prog"), vec![])
            .unwrap();

        // Set setuid bit (04755)
        vfs.chmod(&PathBuf::from("/setuid_prog"), 0o4755).unwrap();

        let mode = vfs.get_mode(&PathBuf::from("/setuid_prog")).unwrap();
        assert_eq!(mode & 0o4000, 0o4000, "Setuid bit should be set");
        assert!(
            vfs.is_setuid(&PathBuf::from("/setuid_prog")).unwrap(),
            "is_setuid should return true"
        );
    }

    #[test]
    fn test_setgid_bit() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/setgid_prog"), vec![])
            .unwrap();

        // Set setgid bit (02755)
        vfs.chmod(&PathBuf::from("/setgid_prog"), 0o2755).unwrap();

        let mode = vfs.get_mode(&PathBuf::from("/setgid_prog")).unwrap();
        assert_eq!(mode & 0o2000, 0o2000, "Setgid bit should be set");
        assert!(
            vfs.is_setgid(&PathBuf::from("/setgid_prog")).unwrap(),
            "is_setgid should return true"
        );
    }

    #[test]
    fn test_sticky_bit() {
        let mut vfs = VirtualFileSystem::new_with_context(0, 0);

        // /tmp already exists from new(), just set sticky bit (01777)
        vfs.chmod(&PathBuf::from("/tmp"), 0o1777).unwrap();

        let mode = vfs.get_mode(&PathBuf::from("/tmp")).unwrap();
        assert_eq!(mode & 0o1000, 0o1000, "Sticky bit should be set");
        assert!(
            vfs.is_sticky(&PathBuf::from("/tmp")).unwrap(),
            "is_sticky should return true"
        );
    }

    // Integration tests

    #[test]
    fn test_full_permission_workflow() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);

        // Create file as user 1000
        vfs.create_file(PathBuf::from("/doc.txt"), b"content".to_vec())
            .unwrap();

        // Check initial permissions (should be 0644 by default)
        let mode = vfs.get_mode(&PathBuf::from("/doc.txt")).unwrap();
        assert_eq!(mode & 0o777, 0o644);

        // Change permissions to 0600
        vfs.chmod(&PathBuf::from("/doc.txt"), 0o600).unwrap();

        // Owner can read/write
        assert!(vfs
            .read_file_as(&PathBuf::from("/doc.txt"), 1000, 1000)
            .is_ok());
        assert!(vfs
            .write_file_as(&PathBuf::from("/doc.txt"), b"new".to_vec(), 1000, 1000)
            .is_ok());

        // Others cannot read
        assert_eq!(
            vfs.read_file_as(&PathBuf::from("/doc.txt"), 1001, 1001),
            Err(VfsError::PermissionDenied)
        );
    }

    #[test]
    fn test_chmod_chown_combined() {
        let mut vfs = VirtualFileSystem::new_with_context(0, 0); // Root
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        // Change ownership and permissions
        vfs.chown(&PathBuf::from("/file.txt"), Some(1000), Some(1000))
            .unwrap();
        vfs.chmod(&PathBuf::from("/file.txt"), 0o600).unwrap();

        assert_eq!(vfs.get_owner(&PathBuf::from("/file.txt")).unwrap(), 1000);
        assert_eq!(vfs.get_group(&PathBuf::from("/file.txt")).unwrap(), 1000);
        assert_eq!(
            vfs.get_mode(&PathBuf::from("/file.txt")).unwrap() & 0o777,
            0o600
        );
    }

    #[test]
    fn test_permission_inheritance() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_directory(PathBuf::from("/dir")).unwrap();

        // Switch to root to change ownership
        vfs.set_context(0, 0);

        // Set directory GID and setgid bit
        vfs.chown(&PathBuf::from("/dir"), None, Some(2000)).unwrap();
        vfs.chmod(&PathBuf::from("/dir"), 0o2775).unwrap();

        // Switch back to UID 1000 for file creation
        vfs.set_context(1000, 1000);

        // Create file in directory - should inherit group
        vfs.create_file(PathBuf::from("/dir/file.txt"), vec![])
            .unwrap();

        let gid = vfs.get_group(&PathBuf::from("/dir/file.txt")).unwrap();
        assert_eq!(
            gid, 2000,
            "File should inherit directory's GID when setgid is set"
        );
    }

    #[test]
    fn test_umask_application() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);

        // Set umask to 0022
        vfs.set_umask(0o022);

        // Create file - should get 0644 (0666 & ~0022)
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        let mode = vfs.get_mode(&PathBuf::from("/file.txt")).unwrap();
        assert_eq!(mode & 0o777, 0o644, "File should respect umask");

        // Create directory - should get 0755 (0777 & ~0022)
        vfs.create_directory(PathBuf::from("/dir")).unwrap();

        let mode = vfs.get_mode(&PathBuf::from("/dir")).unwrap();
        assert_eq!(mode & 0o777, 0o755, "Directory should respect umask");
    }

    #[test]
    fn test_permission_check_on_directory() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_directory(PathBuf::from("/dir")).unwrap();
        vfs.create_file(PathBuf::from("/dir/file.txt"), b"data".to_vec())
            .unwrap();

        // Remove execute permission on directory (cannot traverse)
        vfs.chmod(&PathBuf::from("/dir"), 0o644).unwrap();

        // Cannot access file without execute on directory
        let result = vfs.read_file_as(&PathBuf::from("/dir/file.txt"), 1000, 1000);
        assert_eq!(
            result,
            Err(VfsError::PermissionDenied),
            "Need execute on dir to access files"
        );
    }

    #[test]
    fn test_setuid_execution() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/sudo"), vec![]).unwrap();

        // Switch to root to change ownership
        vfs.set_context(0, 0);

        // Make it owned by root with setuid
        vfs.chown(&PathBuf::from("/sudo"), Some(0), Some(0))
            .unwrap();
        vfs.chmod(&PathBuf::from("/sudo"), 0o4755).unwrap();

        // When executed by user 1000, effective UID should become 0
        let euid = vfs
            .get_effective_uid(&PathBuf::from("/sudo"), 1000)
            .unwrap();
        assert_eq!(euid, 0, "Setuid should make effective UID = file owner UID");
    }

    #[test]
    fn test_group_permission_hierarchy() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);
        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        // Owner: no permission, Group: read, Other: no permission (0040)
        vfs.chmod(&PathBuf::from("/file.txt"), 0o040).unwrap();

        // Owner cannot read (owner permissions checked first)
        let result = vfs.read_file_as(&PathBuf::from("/file.txt"), 1000, 1000);
        assert_eq!(
            result,
            Err(VfsError::PermissionDenied),
            "Owner check before group"
        );

        // Group member (different UID, same GID) can read
        let result = vfs.read_file_as(&PathBuf::from("/file.txt"), 1001, 1000);
        assert!(result.is_ok(), "Group member should be able to read");
    }

    #[test]
    fn test_permission_chain() {
        let mut vfs = VirtualFileSystem::new_with_context(1000, 1000);

        // Create nested structure
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        vfs.create_file(PathBuf::from("/a/b/file.txt"), b"data".to_vec())
            .unwrap();

        // Remove execute from /a
        vfs.chmod(&PathBuf::from("/a"), 0o644).unwrap();

        // Cannot access /a/b/file.txt without execute on /a
        let result = vfs.read_file_as(&PathBuf::from("/a/b/file.txt"), 1000, 1000);
        assert_eq!(
            result,
            Err(VfsError::PermissionDenied),
            "Need execute on all path components"
        );
    }

    // ========================================================================
    // WOS-FS-005: File Metadata (stat, timestamps)
    // ========================================================================

    // Unit tests (10 tests)

    #[test]
    fn test_stat_returns_metadata() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), b"hello".to_vec())
            .unwrap();

        let stat = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        assert_eq!(stat.size, 5, "File size should be 5 bytes");
        assert_eq!(stat.file_type, FileType::RegularFile);
    }

    #[test]
    fn test_stat_directory() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/dir")).unwrap();

        let stat = vfs.stat(&PathBuf::from("/dir")).unwrap();
        assert_eq!(stat.file_type, FileType::Directory);
    }

    #[test]
    fn test_stat_symlink() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/target.txt"), vec![]).unwrap();
        vfs.create_symlink(PathBuf::from("/link"), PathBuf::from("/target.txt"))
            .unwrap();

        let stat = vfs.lstat(&PathBuf::from("/link")).unwrap(); // lstat doesn't follow symlink
        assert_eq!(stat.file_type, FileType::Symlink);
    }

    #[test]
    fn test_timestamps_initialized() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        let stat = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        assert!(stat.atime > 0, "atime should be set");
        assert!(stat.mtime > 0, "mtime should be set");
        assert!(stat.ctime > 0, "ctime should be set");
    }

    #[test]
    fn test_atime_updated_on_read() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        let stat_before = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        let atime_before = stat_before.atime;

        // Simulate time passing
        std::thread::sleep(std::time::Duration::from_millis(10));

        vfs.read_file(&PathBuf::from("/file.txt")).unwrap();

        let stat_after = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        assert!(
            stat_after.atime >= atime_before,
            "atime should be updated on read"
        );
    }

    #[test]
    fn test_mtime_updated_on_write() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        let stat_before = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        let mtime_before = stat_before.mtime;

        // Simulate time passing
        std::thread::sleep(std::time::Duration::from_millis(10));

        vfs.write_file(&PathBuf::from("/file.txt"), b"new".to_vec())
            .unwrap();

        let stat_after = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        assert!(
            stat_after.mtime > mtime_before,
            "mtime should be updated on write"
        );
    }

    #[test]
    fn test_ctime_updated_on_chmod() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        let stat_before = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        let ctime_before = stat_before.ctime;

        // Simulate time passing
        std::thread::sleep(std::time::Duration::from_millis(10));

        vfs.chmod(&PathBuf::from("/file.txt"), 0o644).unwrap();

        let stat_after = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        assert!(
            stat_after.ctime > ctime_before,
            "ctime should be updated on chmod"
        );
    }

    #[test]
    fn test_file_size_tracked() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), b"hello world".to_vec())
            .unwrap();

        let stat = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        assert_eq!(stat.size, 11, "File size should be 11 bytes");

        // Write different content
        vfs.write_file(&PathBuf::from("/file.txt"), b"hi".to_vec())
            .unwrap();

        let stat = vfs.stat(&PathBuf::from("/file.txt")).unwrap();
        assert_eq!(stat.size, 2, "File size should be updated to 2 bytes");
    }

    #[test]
    fn test_directory_size() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/dir")).unwrap();

        let stat = vfs.stat(&PathBuf::from("/dir")).unwrap();
        // Directory size is typically fixed (e.g., 4096 or 0)
        assert!(stat.size >= 0, "Directory should have a size");
    }

    #[test]
    fn test_stat_nonexistent_file() {
        let mut vfs = VirtualFileSystem::new();
        let result = vfs.stat(&PathBuf::from("/nonexistent.txt"));
        assert_eq!(result, Err(VfsError::NotFound));
    }

    // Integration tests (4 tests)

    #[test]
    fn test_metadata_workflow() {
        let mut vfs = VirtualFileSystem::new();

        // Create file
        vfs.create_file(PathBuf::from("/doc.txt"), b"initial".to_vec())
            .unwrap();

        let stat1 = vfs.stat(&PathBuf::from("/doc.txt")).unwrap();
        assert_eq!(stat1.size, 7);

        std::thread::sleep(std::time::Duration::from_millis(10));

        // Modify file
        vfs.write_file(&PathBuf::from("/doc.txt"), b"modified content".to_vec())
            .unwrap();

        let stat2 = vfs.stat(&PathBuf::from("/doc.txt")).unwrap();
        assert_eq!(stat2.size, 16);
        assert!(stat2.mtime > stat1.mtime, "mtime should increase");

        std::thread::sleep(std::time::Duration::from_millis(10));

        // Change permissions
        vfs.chmod(&PathBuf::from("/doc.txt"), 0o444).unwrap();

        let stat3 = vfs.stat(&PathBuf::from("/doc.txt")).unwrap();
        assert!(stat3.ctime > stat2.ctime, "ctime should increase");
    }

    #[test]
    fn test_stat_vs_lstat() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/target.txt"), b"hello".to_vec())
            .unwrap();
        vfs.create_symlink(PathBuf::from("/link"), PathBuf::from("/target.txt"))
            .unwrap();

        // stat follows symlink
        let stat = vfs.stat(&PathBuf::from("/link")).unwrap();
        assert_eq!(stat.file_type, FileType::RegularFile);
        assert_eq!(stat.size, 5);

        // lstat doesn't follow symlink
        let lstat = vfs.lstat(&PathBuf::from("/link")).unwrap();
        assert_eq!(lstat.file_type, FileType::Symlink);
    }

    #[test]
    fn test_timestamps_preserved_on_hard_link() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), vec![]).unwrap();

        let stat_original = vfs.stat(&PathBuf::from("/file.txt")).unwrap();

        // Create hard link
        vfs.link(PathBuf::from("/file.txt"), PathBuf::from("/link.txt"))
            .unwrap();

        let stat_link = vfs.stat(&PathBuf::from("/link.txt")).unwrap();

        // Hard links share the same inode, so timestamps should be identical
        assert_eq!(stat_link.atime, stat_original.atime);
        assert_eq!(stat_link.mtime, stat_original.mtime);
        assert_eq!(stat_link.ctime, stat_original.ctime);
    }

    #[test]
    fn test_metadata_after_operations() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Read should update atime
        vfs.read_file(&PathBuf::from("/test.txt")).unwrap();
        let stat1 = vfs.stat(&PathBuf::from("/test.txt")).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        // Write should update mtime and ctime
        vfs.write_file(&PathBuf::from("/test.txt"), b"new".to_vec())
            .unwrap();
        let stat2 = vfs.stat(&PathBuf::from("/test.txt")).unwrap();
        assert!(stat2.mtime > stat1.mtime);
        assert!(stat2.ctime > stat1.ctime);
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
                mode in 0u32..0o7777,
                uid in 0u32..10000,
                gid in 0u32..10000,
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(path);
                let perms = FilePermissions::new(mode, uid, gid);

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

            // ========================================================================
            // WOS-FS-003: Hard Link Property Tests
            // ========================================================================

            /// Property: Hard links always share content
            #[test]
            fn proptest_hard_links_share_content(
                file_path in prop::string::string_regex("/[a-z]{1,10}\\.txt").unwrap(),
                link_path in prop::string::string_regex("/[a-z]{1,10}_link\\.txt").unwrap(),
                content in prop::collection::vec(any::<u8>(), 0..256),
            ) {
                prop_assume!(file_path != link_path);

                let mut vfs = VirtualFileSystem::new();
                let file = PathBuf::from(file_path);
                let link = PathBuf::from(link_path);

                vfs.create_file(file.clone(), content.clone()).unwrap();
                vfs.link(file.clone(), link.clone()).unwrap();

                // Both should have same content
                let content1 = vfs.read_file(&file).unwrap();
                let content2 = vfs.read_file(&link).unwrap();
                prop_assert_eq!(&content1, &content2);
                prop_assert_eq!(&content1, &content);

                // Write through one link
                let new_content: Vec<u8> = (0..content.len()).map(|i| !content[i]).collect();
                vfs.write_file(&link, new_content.clone()).unwrap();

                // Read through other link
                let content3 = vfs.read_file(&file).unwrap();
                prop_assert_eq!(content3, new_content);
            }

            /// Property: Reference count matches number of links
            #[test]
            fn proptest_refcount_matches_links(
                base_path in prop::string::string_regex("/[a-z]{1,10}\\.txt").unwrap(),
                num_links in 0_usize..10_usize,
            ) {
                let mut vfs = VirtualFileSystem::new();
                let base = PathBuf::from(&base_path);

                vfs.create_file(base.clone(), vec![]).unwrap();

                // Create hard links
                for i in 0..num_links {
                    let link_path = PathBuf::from(format!("/link{}.txt", i));
                    vfs.link(base.clone(), link_path).unwrap();
                }

                // Link count should be num_links + 1 (original + links)
                let link_count = vfs.get_link_count(&base).unwrap();
                prop_assert_eq!(link_count, (num_links + 1) as u64);
            }

            /// Property: Unlink operations are atomic (refcount always consistent)
            #[test]
            fn proptest_unlink_atomicity(
                file_path in prop::string::string_regex("/[a-z]{1,10}\\.txt").unwrap(),
                link_paths in prop::collection::vec(
                    prop::string::string_regex("/link[a-z0-9]{1,5}\\.txt").unwrap(),
                    1..8
                ),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let file = PathBuf::from(&file_path);

                vfs.create_file(file.clone(), b"data".to_vec()).unwrap();

                // Create unique links
                let mut unique_links = std::collections::HashSet::new();
                unique_links.insert(file_path.clone());

                for link_path_str in &link_paths {
                    if unique_links.insert(link_path_str.clone()) {
                        let link_path = PathBuf::from(link_path_str);
                        if vfs.link(file.clone(), link_path).is_ok() {
                            // After each link, verify link count
                            let count = vfs.get_link_count(&file).unwrap();
                            prop_assert_eq!(count as usize, unique_links.len());
                        }
                    }
                }

                // Unlink all but the original
                for link_path_str in &link_paths {
                    let link_path = PathBuf::from(link_path_str);
                    if vfs.exists(&link_path) {
                        vfs.unlink(&link_path).unwrap();
                        // Verify refcount decreased
                        if vfs.exists(&file) {
                            let count = vfs.get_link_count(&file).unwrap();
                            prop_assert!(count > 0);
                        }
                    }
                }
            }
        }
    }
}
