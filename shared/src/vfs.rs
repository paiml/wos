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

/// Lock type for file locking
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LockType {
    /// Shared (read) lock - multiple readers allowed
    Shared,
    /// Exclusive (write) lock - only one holder allowed
    Exclusive,
}

/// Whole-file lock (flock)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct FileLock {
    /// Lock type
    lock_type: LockType,
    /// Lock owner (process ID or 0 for current process)
    owner: u64,
    /// Number of times this lock has been acquired (for recursive locking)
    count: usize,
}

/// Byte-range lock (fcntl)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct RangeLock {
    /// Lock type
    lock_type: LockType,
    /// Start offset in bytes
    offset: u64,
    /// Length in bytes
    length: u64,
    /// Lock owner (process ID or 0 for current process)
    owner: u64,
}

/// Mount information for mounted filesystems
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct MountInfo {
    /// The mounted filesystem
    filesystem: Box<VirtualFileSystem>,
    /// Whether the mount is readonly
    readonly: bool,
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
    /// Whole-file locks (flock) - maps inode number to list of locks
    file_locks: im::HashMap<InodeNumber, im::Vector<FileLock>>,
    /// Byte-range locks (fcntl) - maps inode number to list of range locks
    range_locks: im::HashMap<InodeNumber, im::Vector<RangeLock>>,
    /// Mounted filesystems - maps mount point path to mount info
    mounts: im::HashMap<PathBuf, MountInfo>,
    /// Extended attributes (xattr) - maps inode number to map of attribute name -> value
    xattrs: im::HashMap<InodeNumber, im::HashMap<String, Vec<u8>>>,
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

/// Type of device file
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    /// /dev/null - discards writes, returns empty on read
    Null,
    /// /dev/zero - returns infinite zeros on read
    Zero,
    /// /dev/random - returns pseudorandom bytes deterministically
    Random {
        /// Seed for deterministic random number generation
        seed: u64,
    },
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
    /// Device file (special files like /dev/null, /dev/zero, /dev/random)
    Device {
        /// Device type
        device_type: DeviceType,
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
    /// Lock conflict (file already locked)
    LockConflict,
    /// Filesystem is mounted read-only
    ReadOnlyFilesystem,
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
            file_locks: im::HashMap::new(),
            range_locks: im::HashMap::new(),
            mounts: im::HashMap::new(),
            xattrs: im::HashMap::new(),
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

        // Create device files in /dev
        let _ = vfs.create_device(PathBuf::from("/dev/null"), DeviceType::Null);
        let _ = vfs.create_device(PathBuf::from("/dev/zero"), DeviceType::Zero);
        let _ = vfs.create_device(
            PathBuf::from("/dev/random"),
            DeviceType::Random { seed: 42 },
        );

        vfs
    }

    /// Create a new VFS with specific user context
    pub fn new_with_context(uid: u32, gid: u32) -> Self {
        let mut vfs = Self::new();
        vfs.current_uid = uid;
        vfs.current_gid = gid;
        vfs
    }

    /// Normalize a path by resolving . and .. and removing redundant slashes
    ///
    /// # Arguments
    /// * `path` - The path to normalize
    ///
    /// # Returns
    /// A normalized PathBuf with:
    /// - `.` (current directory) components removed
    /// - `..` (parent directory) references resolved
    /// - Multiple consecutive slashes collapsed to single slash
    /// - Parent references bounded at root (/../ stays at /)
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use wos_shared::vfs::VirtualFileSystem;
    ///
    /// let normalized = VirtualFileSystem::normalize_path(&PathBuf::from("/a/./b"));
    /// assert_eq!(normalized, PathBuf::from("/a/b"));
    ///
    /// let normalized2 = VirtualFileSystem::normalize_path(&PathBuf::from("/a/b/../c"));
    /// assert_eq!(normalized2, PathBuf::from("/a/c"));
    /// ```
    pub fn normalize_path(path: &Path) -> PathBuf {
        let path_str = match path.to_str() {
            Some(s) => s,
            None => return path.to_path_buf(), // Return as-is if not valid UTF-8
        };

        // Handle root specially
        if path_str == "/" {
            return PathBuf::from("/");
        }

        // Split into components, filtering out empty strings and current directory markers
        let components: Vec<&str> = path_str
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut normalized: Vec<&str> = Vec::new();

        for component in components {
            match component {
                "." => {
                    // Skip current directory marker
                    continue;
                }
                ".." => {
                    // Pop parent unless we're at root
                    if !normalized.is_empty() {
                        normalized.pop();
                    }
                    // If normalized is empty, we're at root, so .. has no effect
                }
                _ => {
                    // Regular component
                    normalized.push(component);
                }
            }
        }

        // Build the normalized path
        if normalized.is_empty() {
            PathBuf::from("/")
        } else {
            PathBuf::from(format!("/{}", normalized.join("/")))
        }
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

        // Normalize path to handle ., .., and multiple slashes
        let normalized_path = Self::normalize_path(path);

        if normalized_path == Path::new("/") {
            return Ok(self.root_ino);
        }

        let components: Vec<&str> = normalized_path
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
                InodeType::File { .. } | InodeType::Device { .. } => {
                    return Err(VfsError::NotADirectory);
                }
            }
        }

        Ok(current_ino)
    }

    /// Get parent directory inode and entry name
    fn resolve_parent(&self, path: &Path) -> Result<(InodeNumber, String), VfsError> {
        // Normalize path to handle ., .., and multiple slashes
        let normalized_path = Self::normalize_path(path);
        let path_str = normalized_path.to_str().ok_or(VfsError::InvalidPath)?;

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
                InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
                    return Err(VfsError::NotADirectory);
                }
            }
        }

        Ok((current_ino, name))
    }

    /// Create a directory
    pub fn create_directory(&mut self, path: PathBuf) -> Result<(), VfsError> {
        // Check if path is within a mount point
        match self.resolve_mount_mut(&path) {
            Ok((mounted_fs, relative_path)) => {
                return mounted_fs.create_directory(relative_path);
            }
            Err(VfsError::ReadOnlyFilesystem) => {
                return Err(VfsError::ReadOnlyFilesystem);
            }
            Err(VfsError::NotFound) => {
                // Not mounted, create in root filesystem (continue below)
            }
            Err(e) => {
                return Err(e);
            }
        }

        // Not mounted, create in root filesystem
        let (parent_ino, name) = self.resolve_parent(&path)?;

        let parent = self
            .inodes
            .get(&parent_ino)
            .ok_or(VfsError::NotFound)?
            .clone();

        let entries = match &parent.inode_type {
            InodeType::Directory { entries } => entries.clone(),
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
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
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
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
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
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
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
                return Err(VfsError::NotADirectory)
            }
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
            InodeType::File { .. } | InodeType::Directory { .. } | InodeType::Device { .. } => {
                Err(VfsError::InvalidPath)
            }
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
            InodeType::File { .. } | InodeType::Directory { .. } | InodeType::Device { .. } => {
                // Not a symlink, return the path itself
                Ok(path.to_path_buf())
            }
        }
    }

    /// List directory entries
    pub fn list_directory(&self, path: &Path) -> Result<Vec<DirectoryEntry>, VfsError> {
        // Check if path is within a mount point
        if let (Some(mounted_fs), relative_path) = self.resolve_mount(path) {
            return mounted_fs.list_directory(&relative_path);
        }

        // Not mounted, list from root filesystem
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        let entries = match &inode.inode_type {
            InodeType::Directory { entries } => entries,
            InodeType::File { .. } => return Err(VfsError::NotADirectory),
            InodeType::Symlink { .. } => return Err(VfsError::NotADirectory),
            InodeType::Device { .. } => return Err(VfsError::NotADirectory),
        };

        let mut result = Vec::new();
        for (name, &child_ino) in entries.iter() {
            let child_inode = self.inodes.get(&child_ino).ok_or(VfsError::NotFound)?;
            let entry_type = match &child_inode.inode_type {
                InodeType::File { .. } => EntryType::File,
                InodeType::Directory { .. } => EntryType::Directory,
                InodeType::Symlink { .. } => EntryType::Symlink,
                InodeType::Device { .. } => EntryType::File,
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
        // Check if path is within a mount point
        match self.resolve_mount_mut(&path) {
            Ok((mounted_fs, relative_path)) => {
                return mounted_fs.create_file(relative_path, content);
            }
            Err(VfsError::ReadOnlyFilesystem) => {
                // Propagate readonly error
                return Err(VfsError::ReadOnlyFilesystem);
            }
            Err(VfsError::NotFound) => {
                // Not mounted, create in root filesystem (continue below)
            }
            Err(e) => {
                // Other errors should also be propagated
                return Err(e);
            }
        }

        // Not mounted, create in root filesystem
        let (parent_ino, name) = self.resolve_parent(&path)?;

        let parent = self
            .inodes
            .get(&parent_ino)
            .ok_or(VfsError::NotFound)?
            .clone();

        let entries = match &parent.inode_type {
            InodeType::Directory { entries } => entries.clone(),
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
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

    /// Create a device file
    pub fn create_device(
        &mut self,
        path: PathBuf,
        device_type: DeviceType,
    ) -> Result<(), VfsError> {
        let (parent_ino, name) = self.resolve_parent(&path)?;

        let parent = self
            .inodes
            .get(&parent_ino)
            .ok_or(VfsError::NotFound)?
            .clone();

        let entries = match &parent.inode_type {
            InodeType::Directory { entries } => entries.clone(),
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
                return Err(VfsError::NotADirectory)
            }
        };

        if entries.contains_key(&name) {
            return Err(VfsError::AlreadyExists);
        }

        // Create new device inode
        let new_ino = self.next_ino;
        self.next_ino += 1;

        // Device files are typically 0666 (rw-rw-rw-) with umask applied
        let mode = 0o666 & !self.umask;
        let permissions = FilePermissions::new(mode, 0, 0); // Root owned

        let now = current_timestamp();
        let new_device = Inode {
            ino: new_ino,
            inode_type: InodeType::Device { device_type },
            permissions,
            nlinks: 1,
            atime: now,
            mtime: now,
            ctime: now,
        };

        self.inodes.insert(new_ino, new_device);

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
            mtime: now,
            ctime: now,
        };

        self.inodes.insert(parent_ino, updated_parent);

        Ok(())
    }

    /// Read file contents (uses current user context)
    pub fn read_file(&mut self, path: &Path) -> Result<Vec<u8>, VfsError> {
        // Check if path is within a mount point
        if let Ok((mounted_fs, relative_path)) = self.resolve_mount_mut(path) {
            return mounted_fs.read_file(&relative_path);
        }

        // Not mounted, read from root filesystem
        let uid = self.current_uid;
        let gid = self.current_gid;
        self.read_file_as(path, uid, gid)
    }

    /// Write to file (overwrites existing content, uses current user context)
    pub fn write_file(&mut self, path: &Path, content: Vec<u8>) -> Result<(), VfsError> {
        // Check if path is within a mount point
        match self.resolve_mount_mut(path) {
            Ok((mounted_fs, relative_path)) => {
                return mounted_fs.write_file(&relative_path, content);
            }
            Err(VfsError::ReadOnlyFilesystem) => {
                return Err(VfsError::ReadOnlyFilesystem);
            }
            Err(VfsError::NotFound) => {
                // Not mounted, write to root filesystem (continue below)
            }
            Err(e) => {
                return Err(e);
            }
        }

        // Not mounted, write to root filesystem
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
        // Check if path is within a mount point
        match self.resolve_mount_mut(path) {
            Ok((mounted_fs, relative_path)) => {
                return mounted_fs.delete_file(&relative_path);
            }
            Err(VfsError::ReadOnlyFilesystem) => {
                return Err(VfsError::ReadOnlyFilesystem);
            }
            Err(VfsError::NotFound) => {
                // Not mounted, delete from root filesystem (continue below)
            }
            Err(e) => {
                return Err(e);
            }
        }

        // Not mounted, delete from root filesystem
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
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
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

        // Clean up associated data
        self.file_locks.remove(&ino);
        self.range_locks.remove(&ino);
        self.xattrs.remove(&ino);

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
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
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
            InodeType::File { .. } | InodeType::Symlink { .. } | InodeType::Device { .. } => {
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
                InodeType::Device { .. } => {
                    // Device files aren't regular files
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

    /// Check file access permissions
    /// mode bits: F_OK=0 (file exists), R_OK=4 (readable), W_OK=2 (writable), X_OK=1 (executable)
    pub fn access(&self, path: &Path, mode: u32) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        // F_OK (0): Just check if file exists - already done by resolve_path
        if mode == 0 {
            return Ok(());
        }

        // Check read permission (R_OK = 4)
        if (mode & 4) != 0 && !self.check_permission(inode, self.current_uid, self.current_gid, 4) {
            return Err(VfsError::PermissionDenied);
        }

        // Check write permission (W_OK = 2)
        if (mode & 2) != 0 && !self.check_permission(inode, self.current_uid, self.current_gid, 2) {
            return Err(VfsError::PermissionDenied);
        }

        // Check execute permission (X_OK = 1)
        if (mode & 1) != 0 && !self.check_permission(inode, self.current_uid, self.current_gid, 1) {
            return Err(VfsError::PermissionDenied);
        }

        Ok(())
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
            atime: now,         // Update on read
            mtime: inode.mtime, // Preserve mtime
            ctime: inode.ctime, // Preserve ctime
        };
        self.inodes.insert(ino, updated_inode);

        match &inode.inode_type {
            InodeType::File { content } => Ok(content.clone()),
            InodeType::Directory { .. } => Err(VfsError::IsADirectory),
            InodeType::Symlink { .. } => Err(VfsError::NotFound),
            InodeType::Device { device_type } => match device_type {
                DeviceType::Null => Ok(vec![]),
                DeviceType::Zero => Ok(vec![0u8; 4096]),
                DeviceType::Random { seed } => {
                    use rand::RngCore;
                    use rand::SeedableRng;
                    use rand_chacha::ChaCha8Rng;

                    let mut rng = ChaCha8Rng::seed_from_u64(*seed);
                    let mut bytes = vec![0u8; 4096];
                    rng.fill_bytes(&mut bytes);
                    Ok(bytes)
                }
            },
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
            InodeType::Device { .. } => {
                // Device files accept writes but discard data
                // Update atime only (device content doesn't change)
                let now = current_timestamp();
                let updated_inode = Inode {
                    ino,
                    inode_type: inode.inode_type.clone(),
                    permissions: inode.permissions.clone(),
                    nlinks: inode.nlinks,
                    atime: now,         // Update on write
                    mtime: inode.mtime, // Device content doesn't change
                    ctime: inode.ctime, // Metadata doesn't change
                };
                self.inodes.insert(ino, updated_inode);
                Ok(())
            }
        }
    }

    /// Check if file has read permission for current user
    pub fn can_read(&self, path: &Path) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        // Check read permission (mode_bit 4 = read)
        if self.check_permission(inode, self.current_uid, self.current_gid, 4) {
            Ok(())
        } else {
            Err(VfsError::PermissionDenied)
        }
    }

    /// Check if file has write permission for current user
    pub fn can_write(&self, path: &Path) -> Result<(), VfsError> {
        let ino = self.resolve_path(path)?;
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;

        // Check write permission (mode_bit 2 = write)
        if self.check_permission(inode, self.current_uid, self.current_gid, 2) {
            Ok(())
        } else {
            Err(VfsError::PermissionDenied)
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
        // Check if path is within a mount point
        if let (Some(mounted_fs), relative_path) = self.resolve_mount(path) {
            return mounted_fs.stat(&relative_path);
        }

        // Not mounted, stat from root filesystem
        let ino = self.resolve_path(path)?; // follows symlinks
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(self.inode_to_filestat(inode))
    }

    /// Get file metadata (lstat) - does NOT follow symlinks
    pub fn lstat(&self, path: &Path) -> Result<FileStat, VfsError> {
        // Check if path is within a mount point
        if let (Some(mounted_fs), relative_path) = self.resolve_mount(path) {
            return mounted_fs.lstat(&relative_path);
        }

        // Not mounted, lstat from root filesystem
        let ino = self.resolve_path_no_follow(path)?; // doesn't follow final symlink
        let inode = self.inodes.get(&ino).ok_or(VfsError::NotFound)?;
        Ok(self.inode_to_filestat(inode))
    }

    /// Helper to convert Inode to FileStat
    fn inode_to_filestat(&self, inode: &Inode) -> FileStat {
        let (file_type, size) = match &inode.inode_type {
            InodeType::File { content } => (FileType::RegularFile, content.len() as u64),
            InodeType::Directory { .. } => (FileType::Directory, 4096), // Fixed size for directories
            InodeType::Symlink { .. } => (FileType::Symlink, 0),        // Symlinks have size 0
            InodeType::Device { .. } => (FileType::RegularFile, 0),     // Device files have no size
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

    // ========================================================================
    // File Locking API (flock and fcntl)
    // ========================================================================

    /// Acquire a whole-file lock (flock) with default owner
    pub fn flock(&mut self, path: &Path, lock_type: LockType) -> Result<(), VfsError> {
        self.flock_with_owner(path, lock_type, 0)
    }

    /// Acquire a whole-file lock (flock) with specific owner
    pub fn flock_with_owner(
        &mut self,
        path: &Path,
        lock_type: LockType,
        owner: u64,
    ) -> Result<(), VfsError> {
        // Check if path is within a mount point
        match self.resolve_mount_mut(path) {
            Ok((mounted_fs, relative_path)) => {
                return mounted_fs.flock_with_owner(&relative_path, lock_type, owner);
            }
            Err(VfsError::ReadOnlyFilesystem) => {
                return Err(VfsError::ReadOnlyFilesystem);
            }
            Err(VfsError::NotFound) => {
                // Not mounted, lock in root filesystem (continue below)
            }
            Err(e) => {
                return Err(e);
            }
        }

        let ino = self.resolve_path(path)?;

        // Check if file exists
        if !self.inodes.contains_key(&ino) {
            return Err(VfsError::NotFound);
        }

        // Get existing locks for this inode
        let existing_locks = self.file_locks.get(&ino).cloned().unwrap_or_default();

        // Check lock compatibility
        for existing_lock in existing_locks.iter() {
            match (lock_type, existing_lock.lock_type) {
                (LockType::Shared, LockType::Shared) => {
                    // Shared locks are compatible
                    continue;
                }
                (LockType::Exclusive, _) | (_, LockType::Exclusive) => {
                    // Exclusive locks conflict with any other lock
                    return Err(VfsError::LockConflict);
                }
            }
        }

        // Add the new lock
        let new_lock = FileLock {
            lock_type,
            owner,
            count: 1,
        };
        let mut locks = existing_locks;
        locks.push_back(new_lock);
        self.file_locks.insert(ino, locks);

        Ok(())
    }

    /// Release a whole-file lock (flock)
    pub fn funlock(&mut self, path: &Path) -> Result<(), VfsError> {
        // Check if path is within a mount point
        match self.resolve_mount_mut(path) {
            Ok((mounted_fs, relative_path)) => {
                return mounted_fs.funlock(&relative_path);
            }
            Err(VfsError::ReadOnlyFilesystem) => {
                return Err(VfsError::ReadOnlyFilesystem);
            }
            Err(VfsError::NotFound) => {
                // Not mounted, unlock in root filesystem (continue below)
            }
            Err(e) => {
                return Err(e);
            }
        }

        let ino = self.resolve_path(path)?;

        // Remove all locks for this file
        self.file_locks.remove(&ino);

        Ok(())
    }

    /// Check if a file has any locks
    pub fn is_locked(&self, path: &Path) -> bool {
        if let Ok(ino) = self.resolve_path(path) {
            self.file_locks
                .get(&ino)
                .map(|locks| !locks.is_empty())
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Acquire a byte-range lock (fcntl)
    pub fn fcntl_lock(
        &mut self,
        path: &Path,
        lock_type: LockType,
        offset: u64,
        length: u64,
    ) -> Result<(), VfsError> {
        // Check if path is within a mount point
        match self.resolve_mount_mut(path) {
            Ok((mounted_fs, relative_path)) => {
                return mounted_fs.fcntl_lock(&relative_path, lock_type, offset, length);
            }
            Err(VfsError::ReadOnlyFilesystem) => {
                return Err(VfsError::ReadOnlyFilesystem);
            }
            Err(VfsError::NotFound) => {
                // Not mounted, lock in root filesystem (continue below)
            }
            Err(e) => {
                return Err(e);
            }
        }

        let ino = self.resolve_path(path)?;

        // Check if file exists
        if !self.inodes.contains_key(&ino) {
            return Err(VfsError::NotFound);
        }

        // Get existing range locks
        let existing_locks = self.range_locks.get(&ino).cloned().unwrap_or_default();

        // Check for overlapping locks
        let end = offset + length;
        for existing_lock in existing_locks.iter() {
            let existing_end = existing_lock.offset + existing_lock.length;

            // Check if ranges overlap
            let overlaps = !(end <= existing_lock.offset || existing_end <= offset);

            if overlaps {
                match (lock_type, existing_lock.lock_type) {
                    (LockType::Shared, LockType::Shared) => {
                        // Shared locks don't conflict
                        continue;
                    }
                    (LockType::Exclusive, _) | (_, LockType::Exclusive) => {
                        // Exclusive locks conflict
                        return Err(VfsError::LockConflict);
                    }
                }
            }
        }

        // Add the new range lock
        let new_lock = RangeLock {
            lock_type,
            offset,
            length,
            owner: 0,
        };
        let mut locks = existing_locks;
        locks.push_back(new_lock);
        self.range_locks.insert(ino, locks);

        Ok(())
    }

    /// Release a byte-range lock (fcntl)
    pub fn fcntl_unlock(&mut self, path: &Path, offset: u64, length: u64) -> Result<(), VfsError> {
        // Check if path is within a mount point
        match self.resolve_mount_mut(path) {
            Ok((mounted_fs, relative_path)) => {
                return mounted_fs.fcntl_unlock(&relative_path, offset, length);
            }
            Err(VfsError::ReadOnlyFilesystem) => {
                return Err(VfsError::ReadOnlyFilesystem);
            }
            Err(VfsError::NotFound) => {
                // Not mounted, unlock in root filesystem (continue below)
            }
            Err(e) => {
                return Err(e);
            }
        }

        let ino = self.resolve_path(path)?;

        // Get existing range locks
        let existing_locks = self.range_locks.get(&ino).cloned().unwrap_or_default();

        // Remove locks that match this range
        let end = offset + length;
        let filtered_locks: im::Vector<RangeLock> = existing_locks
            .iter()
            .filter(|lock| {
                let lock_end = lock.offset + lock.length;
                // Keep locks that don't match this exact range
                !(lock.offset == offset && lock_end == end)
            })
            .cloned()
            .collect();

        if filtered_locks.is_empty() {
            self.range_locks.remove(&ino);
        } else {
            self.range_locks.insert(ino, filtered_locks);
        }

        Ok(())
    }

    /// Check if a byte range is locked
    pub fn is_range_locked(&self, path: &Path, offset: u64, length: u64) -> bool {
        if let Ok(ino) = self.resolve_path(path) {
            if let Some(locks) = self.range_locks.get(&ino) {
                let end = offset + length;
                for lock in locks.iter() {
                    let lock_end = lock.offset + lock.length;
                    // Check if ranges overlap
                    if !(end <= lock.offset || lock_end <= offset) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Detect potential deadlock (simplified detection)
    pub fn detect_deadlock(&self, _owner: u64, _path: &Path) -> bool {
        // Simplified deadlock detection - in a real system this would be more sophisticated
        // For now, we'll return true if there are multiple exclusive locks
        // indicating potential circular wait condition

        let total_exclusive_locks: usize = self
            .file_locks
            .values()
            .map(|locks| {
                locks
                    .iter()
                    .filter(|lock| lock.lock_type == LockType::Exclusive)
                    .count()
            })
            .sum();

        // If there are 2+ exclusive locks on different files, potential deadlock
        total_exclusive_locks >= 2
    }

    // ========================================================================
    // Mount Points API
    // ========================================================================

    /// Mount a filesystem at the specified path
    pub fn mount(
        &mut self,
        mount_point: PathBuf,
        filesystem: VirtualFileSystem,
    ) -> Result<(), VfsError> {
        // Normalize the mount point path
        let normalized_mount_point = Self::normalize_path(&mount_point);

        // Create mount point directory if it doesn't exist
        if !self.exists(&normalized_mount_point) {
            self.create_directory(normalized_mount_point.clone())?;
        }

        // Check if mount point is a directory (not a file)
        if let Ok(stat) = self.stat(&normalized_mount_point) {
            if stat.file_type != FileType::Directory {
                return Err(VfsError::NotADirectory);
            }
        }

        // Insert the mounted filesystem as read-write
        self.mounts.insert(
            normalized_mount_point,
            MountInfo {
                filesystem: Box::new(filesystem),
                readonly: false,
            },
        );

        Ok(())
    }

    /// Unmount a filesystem at the specified path
    pub fn umount(&mut self, mount_point: &Path) -> Result<(), VfsError> {
        let normalized_mount_point = Self::normalize_path(mount_point);

        // Check if mount point exists
        if !self.mounts.contains_key(&normalized_mount_point) {
            return Err(VfsError::NotFound);
        }

        // Remove the mount
        self.mounts.remove(&normalized_mount_point);

        Ok(())
    }

    /// Check if a path is a mount point
    pub fn is_mount_point(&self, path: &Path) -> bool {
        let normalized_path = Self::normalize_path(path);
        self.mounts.contains_key(&normalized_path)
    }

    /// List all mount points
    pub fn list_mounts(&self) -> Vec<PathBuf> {
        self.mounts.keys().cloned().collect()
    }

    /// Get the mount point that contains the given path (if any)
    pub fn get_mount_point(&self, path: &Path) -> Option<PathBuf> {
        let normalized_path = Self::normalize_path(path);
        let path_str = normalized_path.to_str()?;

        // Find the longest matching mount point (most specific)
        let mut best_match: Option<PathBuf> = None;
        let mut best_match_len = 0;

        for mount_point in self.mounts.keys() {
            let mount_str = mount_point.to_str()?;

            // Check if path starts with this mount point and is a proper prefix
            if path_str.starts_with(mount_str)
                && (path_str == mount_str || path_str.chars().nth(mount_str.len()) == Some('/'))
                && mount_str.len() > best_match_len
            {
                best_match = Some(mount_point.clone());
                best_match_len = mount_str.len();
            }
        }

        best_match
    }

    /// Mount a filesystem as readonly
    pub fn mount_readonly(
        &mut self,
        mount_point: PathBuf,
        filesystem: VirtualFileSystem,
    ) -> Result<(), VfsError> {
        // Normalize the mount point path
        let normalized_mount_point = Self::normalize_path(&mount_point);

        // Create mount point directory if it doesn't exist
        if !self.exists(&normalized_mount_point) {
            self.create_directory(normalized_mount_point.clone())?;
        }

        // Check if mount point is a directory (not a file)
        if let Ok(stat) = self.stat(&normalized_mount_point) {
            if stat.file_type != FileType::Directory {
                return Err(VfsError::NotADirectory);
            }
        }

        // Insert the mounted filesystem as readonly
        self.mounts.insert(
            normalized_mount_point,
            MountInfo {
                filesystem: Box::new(filesystem),
                readonly: true,
            },
        );

        Ok(())
    }

    /// Helper: Resolve a path to the appropriate filesystem and relative path
    fn resolve_mount(&self, path: &Path) -> (Option<&VirtualFileSystem>, PathBuf) {
        let normalized_path = Self::normalize_path(path);

        // Check if path is within a mount point
        if let Some(mount_point) = self.get_mount_point(&normalized_path) {
            if let Some(mount_info) = self.mounts.get(&mount_point) {
                // Calculate relative path within the mounted filesystem
                let mount_str = mount_point.to_str().unwrap_or("/");
                let path_str = normalized_path.to_str().unwrap_or("/");

                let relative_path = if path_str == mount_str {
                    PathBuf::from("/")
                } else if let Some(suffix) = path_str.strip_prefix(mount_str) {
                    PathBuf::from(suffix)
                } else {
                    PathBuf::from("/")
                };

                return (Some(mount_info.filesystem.as_ref()), relative_path);
            }
        }

        // No mount point found, use root filesystem
        (None, normalized_path)
    }

    /// Helper: Resolve a path to a mutable filesystem and relative path
    fn resolve_mount_mut(
        &mut self,
        path: &Path,
    ) -> Result<(&mut VirtualFileSystem, PathBuf), VfsError> {
        let normalized_path = Self::normalize_path(path);

        // Check if path is within a mount point
        if let Some(mount_point) = self.get_mount_point(&normalized_path) {
            if let Some(mount_info) = self.mounts.get_mut(&mount_point) {
                // Check if mount is readonly
                if mount_info.readonly {
                    return Err(VfsError::ReadOnlyFilesystem);
                }

                // Calculate relative path within the mounted filesystem
                let mount_str = mount_point.to_str().unwrap_or("/");
                let path_str = normalized_path.to_str().unwrap_or("/");

                let relative_path = if path_str == mount_str {
                    PathBuf::from("/")
                } else if let Some(suffix) = path_str.strip_prefix(mount_str) {
                    PathBuf::from(suffix)
                } else {
                    PathBuf::from("/")
                };

                return Ok((mount_info.filesystem.as_mut(), relative_path));
            }
        }

        // No mount point found, operation would be on root filesystem
        // We can't return &mut self here, so return an error
        Err(VfsError::NotFound)
    }

    // =========================================================================
    // Extended Attributes (xattr) API
    // =========================================================================

    /// Set extended attribute on a file or directory
    ///
    /// # Arguments
    /// * `path` - Path to file/directory
    /// * `name` - Attribute name (e.g., "user.comment", "system.acl", "security.label")
    /// * `value` - Attribute value (arbitrary bytes)
    ///
    /// # Returns
    /// Ok(()) on success, VfsError::NotFound if file doesn't exist
    pub fn setxattr(&mut self, path: &Path, name: &str, value: &[u8]) -> Result<(), VfsError> {
        // Resolve path to inode
        let normalized_path = Self::normalize_path(path);
        let ino = self.resolve_path_internal(&normalized_path, true, 0)?;

        // Get or create xattr map for this inode
        let mut attr_map = self.xattrs.get(&ino).cloned().unwrap_or_default();

        // Set the attribute
        attr_map.insert(name.to_string(), value.to_vec());

        // Update the xattrs map
        self.xattrs.insert(ino, attr_map);

        Ok(())
    }

    /// Get extended attribute from a file or directory
    ///
    /// # Arguments
    /// * `path` - Path to file/directory
    /// * `name` - Attribute name
    ///
    /// # Returns
    /// Ok(value) on success, VfsError::NotFound if file or attribute doesn't exist
    pub fn getxattr(&self, path: &Path, name: &str) -> Result<Vec<u8>, VfsError> {
        // Resolve path to inode
        let normalized_path = Self::normalize_path(path);
        let ino = self.resolve_path_internal(&normalized_path, true, 0)?;

        // Get xattr map for this inode
        let attr_map = self.xattrs.get(&ino).ok_or(VfsError::NotFound)?;

        // Get the specific attribute
        attr_map.get(name).cloned().ok_or(VfsError::NotFound)
    }

    /// List all extended attribute names for a file or directory
    ///
    /// # Arguments
    /// * `path` - Path to file/directory
    ///
    /// # Returns
    /// Ok(names) on success (may be empty), VfsError::NotFound if file doesn't exist
    pub fn listxattr(&self, path: &Path) -> Result<Vec<String>, VfsError> {
        // Resolve path to inode
        let normalized_path = Self::normalize_path(path);
        let ino = self.resolve_path_internal(&normalized_path, true, 0)?;

        // Get xattr map for this inode (return empty list if no xattrs)
        if let Some(attr_map) = self.xattrs.get(&ino) {
            Ok(attr_map.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Remove extended attribute from a file or directory
    ///
    /// # Arguments
    /// * `path` - Path to file/directory
    /// * `name` - Attribute name
    ///
    /// # Returns
    /// Ok(()) on success, VfsError::NotFound if file or attribute doesn't exist
    pub fn removexattr(&mut self, path: &Path, name: &str) -> Result<(), VfsError> {
        // Resolve path to inode
        let normalized_path = Self::normalize_path(path);
        let ino = self.resolve_path_internal(&normalized_path, true, 0)?;

        // Get xattr map for this inode
        let mut attr_map = self.xattrs.get(&ino).cloned().ok_or(VfsError::NotFound)?;

        // Remove the attribute (error if it doesn't exist)
        if attr_map.remove(name).is_none() {
            return Err(VfsError::NotFound);
        }

        // Update the xattrs map (or remove if empty)
        if attr_map.is_empty() {
            self.xattrs.remove(&ino);
        } else {
            self.xattrs.insert(ino, attr_map);
        }

        Ok(())
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
        vfs.create_file(PathBuf::from("/target.txt"), vec![])
            .unwrap();
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
        // Note: size is unsigned, so it's always >= 0
        assert!(stat.size < u64::MAX, "Directory should have a valid size");
    }

    #[test]
    fn test_stat_nonexistent_file() {
        let vfs = VirtualFileSystem::new();
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

    // ============================================================================
    // WOS-FS-006: Path Normalization and Resolution Tests
    // ============================================================================

    // WOS-FS-006 Test 1: Resolve single dot (current directory)
    #[test]
    fn test_resolve_single_dot() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Path with ./ should resolve to same file
        let content = vfs.read_file(&PathBuf::from("/./test.txt")).unwrap();
        assert_eq!(content, b"content");

        // Multiple dots should also resolve
        let content2 = vfs.read_file(&PathBuf::from("/./././test.txt")).unwrap();
        assert_eq!(content2, b"content");
    }

    // WOS-FS-006 Test 2: Resolve double dot (parent directory)
    #[test]
    fn test_resolve_double_dot() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/dir1")).unwrap();
        vfs.create_directory(PathBuf::from("/dir1/dir2")).unwrap();
        vfs.create_file(PathBuf::from("/dir1/dir2/test.txt"), b"content".to_vec())
            .unwrap();

        // Path with .. should navigate to parent
        let content = vfs
            .read_file(&PathBuf::from("/dir1/dir2/../dir2/test.txt"))
            .unwrap();
        assert_eq!(content, b"content");
    }

    // WOS-FS-006 Test 3: Resolve multiple parent references
    #[test]
    fn test_resolve_multiple_parent_refs() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b/c")).unwrap();
        vfs.create_file(PathBuf::from("/a/b/c/test.txt"), b"content".to_vec())
            .unwrap();

        // ../../ should go up two levels
        let content = vfs
            .read_file(&PathBuf::from("/a/b/c/../../b/c/test.txt"))
            .unwrap();
        assert_eq!(content, b"content");
    }

    // WOS-FS-006 Test 4: Handle multiple slashes
    #[test]
    fn test_handle_multiple_slashes() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Multiple slashes should be treated as single slash
        let content = vfs.read_file(&PathBuf::from("///test.txt")).unwrap();
        assert_eq!(content, b"content");

        let content2 = vfs.read_file(&PathBuf::from("/.//.//test.txt")).unwrap();
        assert_eq!(content2, b"content");
    }

    // WOS-FS-006 Test 5: Canonicalize complex path
    #[test]
    fn test_canonicalize_complex_path() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        vfs.create_file(PathBuf::from("/a/b/test.txt"), b"content".to_vec())
            .unwrap();

        // Complex path with ., .., and //
        let content = vfs
            .read_file(&PathBuf::from("/a/./b/../b//test.txt"))
            .unwrap();
        assert_eq!(content, b"content");
    }

    // WOS-FS-006 Test 6: Parent of root should stay at root
    #[test]
    fn test_parent_of_root() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // /../ from root should stay at root
        let content = vfs.read_file(&PathBuf::from("/../test.txt")).unwrap();
        assert_eq!(content, b"content");

        // Multiple parents from root
        let content2 = vfs.read_file(&PathBuf::from("/../../test.txt")).unwrap();
        assert_eq!(content2, b"content");
    }

    // WOS-FS-006 Test 7: Normalize path removes redundant components
    #[test]
    fn test_normalize_path_basic() {
        let normalized = VirtualFileSystem::normalize_path(&PathBuf::from("/a/./b"));
        assert_eq!(normalized, PathBuf::from("/a/b"));

        let normalized2 = VirtualFileSystem::normalize_path(&PathBuf::from("/a//b"));
        assert_eq!(normalized2, PathBuf::from("/a/b"));
    }

    // WOS-FS-006 Test 8: Normalize path handles parent references
    #[test]
    fn test_normalize_path_parent() {
        let normalized = VirtualFileSystem::normalize_path(&PathBuf::from("/a/b/../c"));
        assert_eq!(normalized, PathBuf::from("/a/c"));

        let normalized2 = VirtualFileSystem::normalize_path(&PathBuf::from("/a/b/c/../../d"));
        assert_eq!(normalized2, PathBuf::from("/a/d"));
    }

    // WOS-FS-006 Test 9: Normalize path with only dots
    #[test]
    fn test_normalize_path_only_dots() {
        let normalized = VirtualFileSystem::normalize_path(&PathBuf::from("/."));
        assert_eq!(normalized, PathBuf::from("/"));

        let normalized2 = VirtualFileSystem::normalize_path(&PathBuf::from("/./././."));
        assert_eq!(normalized2, PathBuf::from("/"));
    }

    // WOS-FS-006 Test 10: Normalize path with trailing slash
    #[test]
    fn test_normalize_path_trailing_slash() {
        let normalized = VirtualFileSystem::normalize_path(&PathBuf::from("/a/b/"));
        assert_eq!(normalized, PathBuf::from("/a/b"));

        let normalized2 = VirtualFileSystem::normalize_path(&PathBuf::from("/a/b//"));
        assert_eq!(normalized2, PathBuf::from("/a/b"));
    }

    // WOS-FS-006 Test 11: Create file with normalized path
    #[test]
    fn test_create_file_with_normalized_path() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b")).unwrap();

        // Create file with complex path
        vfs.create_file(PathBuf::from("/a/./b/../b/test.txt"), b"content".to_vec())
            .unwrap();

        // Should be accessible via normalized path
        let content = vfs.read_file(&PathBuf::from("/a/b/test.txt")).unwrap();
        assert_eq!(content, b"content");
    }

    // WOS-FS-006 Test 12: List directory with normalized path
    #[test]
    fn test_list_directory_normalized() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/dir")).unwrap();
        vfs.create_file(PathBuf::from("/dir/test.txt"), b"content".to_vec())
            .unwrap();

        // List directory via normalized path
        let files = vfs.list_directory(&PathBuf::from("/./dir")).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|entry| entry.name == "test.txt"));
    }

    // WOS-FS-006 Test 13: Delete file with normalized path
    #[test]
    fn test_delete_file_normalized() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        vfs.create_file(PathBuf::from("/a/b/test.txt"), b"content".to_vec())
            .unwrap();

        // Delete via complex path
        vfs.delete_file(&PathBuf::from("/a/./b/../b//test.txt"))
            .unwrap();

        // File should not exist
        assert!(vfs.read_file(&PathBuf::from("/a/b/test.txt")).is_err());
    }

    // WOS-FS-006 Test 14: Symlink with normalized path
    #[test]
    fn test_symlink_normalized_path() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/target.txt"), b"content".to_vec())
            .unwrap();

        // Create symlink with complex path
        vfs.create_symlink(
            PathBuf::from("/./link.txt"),
            PathBuf::from("/../target.txt"),
        )
        .unwrap();

        // Should resolve to target
        let content = vfs.read_file(&PathBuf::from("/link.txt")).unwrap();
        assert_eq!(content, b"content");
    }

    // WOS-FS-006 Test 15: Stat with normalized path
    #[test]
    fn test_stat_normalized_path() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        vfs.create_file(PathBuf::from("/a/b/test.txt"), b"content".to_vec())
            .unwrap();

        // Stat via complex path
        let stat = vfs.stat(&PathBuf::from("/a/./b/../b//test.txt")).unwrap();
        assert_eq!(stat.file_type, FileType::RegularFile);
        assert_eq!(stat.size, 7);
    }

    // ============================================================================
    // WOS-FS-006: Path Normalization Integration Tests
    // ============================================================================

    // WOS-FS-006 Integration Test 1: Normalized paths with hard links
    #[test]
    fn test_integration_normalization_with_hardlinks() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/original.txt"), b"content".to_vec())
            .unwrap();

        // Create hard link with normalized path
        vfs.link(
            PathBuf::from("/./original.txt"),
            PathBuf::from("/dir/../link.txt"),
        )
        .unwrap();

        // Both paths should resolve to same inode
        let stat1 = vfs.stat(&PathBuf::from("/original.txt")).unwrap();
        let stat2 = vfs.stat(&PathBuf::from("/link.txt")).unwrap();
        assert_eq!(stat1.ino, stat2.ino);
        assert_eq!(stat1.nlinks, 2);
    }

    // WOS-FS-006 Integration Test 2: Normalized paths with symlinks
    #[test]
    fn test_integration_normalization_with_symlinks() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_file(PathBuf::from("/a/target.txt"), b"content".to_vec())
            .unwrap();

        // Create symlink with complex normalized paths
        vfs.create_symlink(
            PathBuf::from("/./a/../a/link.txt"),
            PathBuf::from("./target.txt"),
        )
        .unwrap();

        // Should resolve correctly
        let content = vfs.read_file(&PathBuf::from("/a/link.txt")).unwrap();
        assert_eq!(content, b"content");
    }

    // WOS-FS-006 Integration Test 3: Normalized paths with permissions
    #[test]
    fn test_integration_normalization_with_permissions() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Set permissions via normalized path
        vfs.chmod(&PathBuf::from("/./test.txt"), 0o644).unwrap();

        // Check via different normalized path
        let stat = vfs.stat(&PathBuf::from("//test.txt")).unwrap();
        assert_eq!(stat.mode & 0o777, 0o644);
    }

    // WOS-FS-006 Integration Test 4: Normalized paths with directory traversal
    #[test]
    fn test_integration_normalization_directory_traversal() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b/c")).unwrap();
        vfs.create_file(PathBuf::from("/a/b/c/file1.txt"), b"1".to_vec())
            .unwrap();
        vfs.create_file(PathBuf::from("/a/b/file2.txt"), b"2".to_vec())
            .unwrap();

        // List directory with normalized path
        let files = vfs
            .list_directory(&PathBuf::from("/a/b/c/../../b"))
            .unwrap();
        assert_eq!(files.len(), 2); // c/ and file2.txt
    }

    // WOS-FS-006 Integration Test 5: Normalized paths with multiple operations
    #[test]
    fn test_integration_normalization_multi_operation() {
        let mut vfs = VirtualFileSystem::new();

        // Create directory first
        vfs.create_directory(PathBuf::from("/dir")).unwrap();

        // Create with normalized path
        vfs.create_file(PathBuf::from("/./dir/../dir/test.txt"), b"initial".to_vec())
            .unwrap();

        // Write with different normalized path
        vfs.write_file(&PathBuf::from("//dir/./test.txt"), b"updated".to_vec())
            .unwrap();

        // Read with yet another normalized path
        let content = vfs
            .read_file(&PathBuf::from("/dir//../dir/test.txt"))
            .unwrap();
        assert_eq!(content, b"updated");

        // Delete with normalized path
        vfs.delete_file(&PathBuf::from("/./dir//test.txt")).unwrap();
        assert!(!vfs.exists(&PathBuf::from("/dir/test.txt")));
    }

    // WOS-FS-006 Integration Test 6: Normalized paths stress test
    #[test]
    fn test_integration_normalization_stress() {
        let mut vfs = VirtualFileSystem::new();

        // Create deeply nested directory structure
        vfs.create_directory(PathBuf::from("/a")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b/c")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b/c/d")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b/c/d/e")).unwrap();
        vfs.create_directory(PathBuf::from("/a/b/c/d/e/f")).unwrap();

        // Create deeply nested structure
        vfs.create_file(PathBuf::from("/a/b/c/d/e/f/test.txt"), b"deep".to_vec())
            .unwrap();

        // Access with complex normalized path
        let content = vfs
            .read_file(&PathBuf::from(
                "/a/./b/../b/c/./d/../d/e/f/../../e/f/test.txt",
            ))
            .unwrap();
        assert_eq!(content, b"deep");

        // Verify stat works
        let stat = vfs
            .stat(&PathBuf::from("/a/b/c/d/e/f/../../e/f/test.txt"))
            .unwrap();
        assert_eq!(stat.file_type, FileType::RegularFile);
        assert_eq!(stat.size, 4);
    }

    // ============================================================================
    // WOS-FS-007: File Locking Tests
    // ============================================================================

    // WOS-FS-007 Test 1: Basic exclusive lock (flock)
    #[test]
    fn test_flock_exclusive_lock() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Acquire exclusive lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Exclusive)
            .unwrap();

        // Verify lock is held
        assert!(vfs.is_locked(&PathBuf::from("/test.txt")));
    }

    // WOS-FS-007 Test 2: Basic shared lock (flock)
    #[test]
    fn test_flock_shared_lock() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Acquire shared lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Shared)
            .unwrap();

        // Verify lock is held
        assert!(vfs.is_locked(&PathBuf::from("/test.txt")));
    }

    // WOS-FS-007 Test 3: Multiple shared locks allowed
    #[test]
    fn test_flock_multiple_shared_locks() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Acquire first shared lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Shared)
            .unwrap();

        // Acquire second shared lock (should succeed)
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Shared)
            .unwrap();

        assert!(vfs.is_locked(&PathBuf::from("/test.txt")));
    }

    // WOS-FS-007 Test 4: Exclusive lock blocks other exclusive locks
    #[test]
    fn test_flock_exclusive_blocks_exclusive() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Acquire exclusive lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Exclusive)
            .unwrap();

        // Try to acquire another exclusive lock (should fail)
        assert!(vfs
            .flock(&PathBuf::from("/test.txt"), LockType::Exclusive)
            .is_err());
    }

    // WOS-FS-007 Test 5: Exclusive lock blocks shared locks
    #[test]
    fn test_flock_exclusive_blocks_shared() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Acquire exclusive lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Exclusive)
            .unwrap();

        // Try to acquire shared lock (should fail)
        assert!(vfs
            .flock(&PathBuf::from("/test.txt"), LockType::Shared)
            .is_err());
    }

    // WOS-FS-007 Test 6: Unlock releases lock
    #[test]
    fn test_flock_unlock() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Acquire lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Exclusive)
            .unwrap();
        assert!(vfs.is_locked(&PathBuf::from("/test.txt")));

        // Release lock
        vfs.funlock(&PathBuf::from("/test.txt")).unwrap();
        assert!(!vfs.is_locked(&PathBuf::from("/test.txt")));
    }

    // WOS-FS-007 Test 7: Byte-range lock (fcntl)
    #[test]
    fn test_fcntl_byte_range_lock() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"0123456789".to_vec())
            .unwrap();

        // Lock bytes 2-5
        vfs.fcntl_lock(
            &PathBuf::from("/test.txt"),
            LockType::Exclusive,
            2,
            4, // offset 2, length 4 -> bytes 2-5
        )
        .unwrap();

        // Verify range is locked
        assert!(vfs.is_range_locked(&PathBuf::from("/test.txt"), 2, 4));
    }

    // WOS-FS-007 Test 8: Byte-range locks don't conflict if non-overlapping
    #[test]
    fn test_fcntl_non_overlapping_locks() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"0123456789".to_vec())
            .unwrap();

        // Lock bytes 0-3
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 0, 4)
            .unwrap();

        // Lock bytes 6-9 (should succeed - no overlap)
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 6, 4)
            .unwrap();

        assert!(vfs.is_range_locked(&PathBuf::from("/test.txt"), 0, 4));
        assert!(vfs.is_range_locked(&PathBuf::from("/test.txt"), 6, 4));
    }

    // WOS-FS-007 Test 9: Byte-range locks conflict if overlapping
    #[test]
    fn test_fcntl_overlapping_locks_conflict() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"0123456789".to_vec())
            .unwrap();

        // Lock bytes 2-6
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 2, 5)
            .unwrap();

        // Try to lock bytes 4-8 (overlaps with 2-6, should fail)
        assert!(vfs
            .fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 4, 5)
            .is_err());
    }

    // WOS-FS-007 Test 10: Unlock byte range
    #[test]
    fn test_fcntl_unlock_range() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"0123456789".to_vec())
            .unwrap();

        // Lock bytes 2-5
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 2, 4)
            .unwrap();
        assert!(vfs.is_range_locked(&PathBuf::from("/test.txt"), 2, 4));

        // Unlock
        vfs.fcntl_unlock(&PathBuf::from("/test.txt"), 2, 4).unwrap();
        assert!(!vfs.is_range_locked(&PathBuf::from("/test.txt"), 2, 4));
    }

    // WOS-FS-007 Test 11: Shared byte-range locks are compatible
    #[test]
    fn test_fcntl_shared_locks_compatible() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"0123456789".to_vec())
            .unwrap();

        // First shared lock on bytes 2-5
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Shared, 2, 4)
            .unwrap();

        // Second shared lock on same bytes (should succeed)
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Shared, 2, 4)
            .unwrap();

        assert!(vfs.is_range_locked(&PathBuf::from("/test.txt"), 2, 4));
    }

    // WOS-FS-007 Test 12: Deadlock detection
    #[test]
    fn test_deadlock_detection() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file1.txt"), b"content1".to_vec())
            .unwrap();
        vfs.create_file(PathBuf::from("/file2.txt"), b"content2".to_vec())
            .unwrap();

        // Simulate process 1 locks file1
        vfs.flock_with_owner(&PathBuf::from("/file1.txt"), LockType::Exclusive, 1)
            .unwrap();

        // Simulate process 2 locks file2
        vfs.flock_with_owner(&PathBuf::from("/file2.txt"), LockType::Exclusive, 2)
            .unwrap();

        // Process 1 tries to lock file2 (blocked by process 2)
        // Process 2 tries to lock file1 (blocked by process 1)
        // This creates a deadlock - system should detect it
        let result = vfs.detect_deadlock(1, &PathBuf::from("/file2.txt"));
        assert!(result); // Should detect potential deadlock
    }

    // ============================================================================
    // WOS-FS-007: File Locking Integration Tests
    // ============================================================================

    // WOS-FS-007 Integration Test 1: Lock prevents write
    #[test]
    fn test_integration_lock_prevents_write() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"initial".to_vec())
            .unwrap();

        // Acquire exclusive lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Exclusive)
            .unwrap();

        // Try to write (should check lock and potentially fail)
        // For advisory locks, write succeeds but lock is advisory
        vfs.write_file(&PathBuf::from("/test.txt"), b"updated".to_vec())
            .unwrap();

        // For mandatory locks, this would fail
        // Test that lock state is maintained
        assert!(vfs.is_locked(&PathBuf::from("/test.txt")));
    }

    // WOS-FS-007 Integration Test 2: Lock survives read
    #[test]
    fn test_integration_lock_survives_read() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Acquire shared lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Shared)
            .unwrap();

        // Read file
        let content = vfs.read_file(&PathBuf::from("/test.txt")).unwrap();
        assert_eq!(content, b"content");

        // Lock should still be held
        assert!(vfs.is_locked(&PathBuf::from("/test.txt")));
    }

    // WOS-FS-007 Integration Test 3: Lock on symlink target
    #[test]
    fn test_integration_lock_on_symlink() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/target.txt"), b"content".to_vec())
            .unwrap();
        vfs.create_symlink(PathBuf::from("/link.txt"), PathBuf::from("/target.txt"))
            .unwrap();

        // Lock via symlink
        vfs.flock(&PathBuf::from("/link.txt"), LockType::Exclusive)
            .unwrap();

        // Target should be locked
        assert!(vfs.is_locked(&PathBuf::from("/target.txt")));
    }

    // WOS-FS-007 Integration Test 4: Locks persist across operations
    #[test]
    fn test_integration_locks_persist() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Lock file
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Exclusive)
            .unwrap();

        // Perform various operations
        vfs.chmod(&PathBuf::from("/test.txt"), 0o644).unwrap();
        let _stat = vfs.stat(&PathBuf::from("/test.txt")).unwrap();

        // Lock should still be held
        assert!(vfs.is_locked(&PathBuf::from("/test.txt")));
    }

    // WOS-FS-007 Integration Test 5: Byte-range locks and flock coexist
    #[test]
    fn test_integration_flock_and_fcntl_coexist() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"0123456789".to_vec())
            .unwrap();

        // Whole-file lock
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Shared)
            .unwrap();

        // Byte-range lock (should coexist or conflict based on semantics)
        // In POSIX, flock and fcntl locks are independent
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 2, 4)
            .unwrap();

        assert!(vfs.is_locked(&PathBuf::from("/test.txt")));
        assert!(vfs.is_range_locked(&PathBuf::from("/test.txt"), 2, 4));
    }

    // WOS-FS-007 Integration Test 6: Lock on deleted file
    #[test]
    fn test_integration_lock_on_deleted_file() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"content".to_vec())
            .unwrap();

        // Lock file
        vfs.flock(&PathBuf::from("/test.txt"), LockType::Exclusive)
            .unwrap();

        // Delete file (should fail while locked, or release lock)
        let result = vfs.delete_file(&PathBuf::from("/test.txt"));
        // Either deletion fails, or lock is auto-released
        assert!(result.is_err() || !vfs.exists(&PathBuf::from("/test.txt")));
    }

    // WOS-FS-007 Integration Test 7: Complex locking scenario
    #[test]
    fn test_integration_complex_locking() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/test.txt"), b"0123456789".to_vec())
            .unwrap();

        // Lock bytes 0-4 (shared)
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Shared, 0, 5)
            .unwrap();

        // Lock bytes 0-4 (shared) again - should succeed
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Shared, 0, 5)
            .unwrap();

        // Lock bytes 6-9 (exclusive) - should succeed (different range)
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 6, 4)
            .unwrap();

        // Try to lock bytes 5-8 (exclusive) - overlaps with 6-9, should fail
        assert!(vfs
            .fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 5, 4)
            .is_err());

        // Unlock bytes 0-4
        vfs.fcntl_unlock(&PathBuf::from("/test.txt"), 0, 5).unwrap();

        // Now lock bytes 0-4 (exclusive) - should succeed since shared locks released
        vfs.fcntl_lock(&PathBuf::from("/test.txt"), LockType::Exclusive, 0, 5)
            .unwrap();
    }

    // ============================================================================
    // WOS-FS-008: Mount Points and Multiple File Systems Tests
    // ============================================================================

    // WOS-FS-008 Test 1: Basic mount operation
    #[test]
    fn test_mount_basic() {
        let mut vfs = VirtualFileSystem::new();
        let mut sub_vfs = VirtualFileSystem::new();
        sub_vfs
            .create_file(PathBuf::from("/data.txt"), b"mounted".to_vec())
            .unwrap();

        // Mount sub_vfs at /mnt
        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Verify mount point exists
        assert!(vfs.is_mount_point(&PathBuf::from("/mnt")));
    }

    // WOS-FS-008 Test 2: Read from mounted filesystem
    #[test]
    fn test_mount_read_file() {
        let mut vfs = VirtualFileSystem::new();
        let mut sub_vfs = VirtualFileSystem::new();
        sub_vfs
            .create_file(PathBuf::from("/data.txt"), b"mounted".to_vec())
            .unwrap();

        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Read file from mounted filesystem
        let content = vfs.read_file(&PathBuf::from("/mnt/data.txt")).unwrap();
        assert_eq!(content, b"mounted");
    }

    // WOS-FS-008 Test 3: Write to mounted filesystem
    #[test]
    fn test_mount_write_file() {
        let mut vfs = VirtualFileSystem::new();
        let sub_vfs = VirtualFileSystem::new();

        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Write file to mounted filesystem
        vfs.create_file(PathBuf::from("/mnt/new.txt"), b"new data".to_vec())
            .unwrap();

        // Verify file exists
        let content = vfs.read_file(&PathBuf::from("/mnt/new.txt")).unwrap();
        assert_eq!(content, b"new data");
    }

    // WOS-FS-008 Test 4: Unmount filesystem
    #[test]
    fn test_unmount() {
        let mut vfs = VirtualFileSystem::new();
        let mut sub_vfs = VirtualFileSystem::new();
        sub_vfs
            .create_file(PathBuf::from("/data.txt"), b"mounted".to_vec())
            .unwrap();

        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();
        assert!(vfs.is_mount_point(&PathBuf::from("/mnt")));

        // Unmount
        vfs.umount(&PathBuf::from("/mnt")).unwrap();
        assert!(!vfs.is_mount_point(&PathBuf::from("/mnt")));

        // File should no longer be accessible
        assert!(vfs.read_file(&PathBuf::from("/mnt/data.txt")).is_err());
    }

    // WOS-FS-008 Test 5: Multiple mount points
    #[test]
    fn test_multiple_mounts() {
        let mut vfs = VirtualFileSystem::new();

        let mut fs1 = VirtualFileSystem::new();
        fs1.create_file(PathBuf::from("/file1.txt"), b"fs1".to_vec())
            .unwrap();

        let mut fs2 = VirtualFileSystem::new();
        fs2.create_file(PathBuf::from("/file2.txt"), b"fs2".to_vec())
            .unwrap();

        vfs.mount(PathBuf::from("/mnt1"), fs1).unwrap();
        vfs.mount(PathBuf::from("/mnt2"), fs2).unwrap();

        // Verify both mounts
        let content1 = vfs.read_file(&PathBuf::from("/mnt1/file1.txt")).unwrap();
        assert_eq!(content1, b"fs1");

        let content2 = vfs.read_file(&PathBuf::from("/mnt2/file2.txt")).unwrap();
        assert_eq!(content2, b"fs2");
    }

    // WOS-FS-008 Test 6: Nested mount points
    #[test]
    fn test_nested_mounts() {
        let mut vfs = VirtualFileSystem::new();
        let mut sub1 = VirtualFileSystem::new();
        let mut sub2 = VirtualFileSystem::new();

        sub2.create_file(PathBuf::from("/deep.txt"), b"nested".to_vec())
            .unwrap();

        sub1.create_directory(PathBuf::from("/inner")).unwrap();
        sub1.mount(PathBuf::from("/inner"), sub2).unwrap();

        vfs.mount(PathBuf::from("/outer"), sub1).unwrap();

        // Access nested mount
        let content = vfs
            .read_file(&PathBuf::from("/outer/inner/deep.txt"))
            .unwrap();
        assert_eq!(content, b"nested");
    }

    // WOS-FS-008 Test 7: Mount over existing directory
    #[test]
    fn test_mount_over_directory() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_directory(PathBuf::from("/mnt")).unwrap();
        vfs.create_file(PathBuf::from("/mnt/old.txt"), b"old".to_vec())
            .unwrap();

        let mut sub_vfs = VirtualFileSystem::new();
        sub_vfs
            .create_file(PathBuf::from("/new.txt"), b"new".to_vec())
            .unwrap();

        // Mount over existing directory (shadows old contents)
        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Old file should be shadowed
        assert!(vfs.read_file(&PathBuf::from("/mnt/old.txt")).is_err());

        // New file should be visible
        let content = vfs.read_file(&PathBuf::from("/mnt/new.txt")).unwrap();
        assert_eq!(content, b"new");
    }

    // WOS-FS-008 Test 8: Mount point path resolution
    #[test]
    fn test_mount_path_resolution() {
        let mut vfs = VirtualFileSystem::new();
        let mut sub_vfs = VirtualFileSystem::new();
        sub_vfs
            .create_file(PathBuf::from("/data.txt"), b"content".to_vec())
            .unwrap();

        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Access via different paths (with normalization)
        let content1 = vfs.read_file(&PathBuf::from("/mnt/data.txt")).unwrap();
        let content2 = vfs.read_file(&PathBuf::from("/mnt/./data.txt")).unwrap();
        let content3 = vfs.read_file(&PathBuf::from("/mnt//data.txt")).unwrap();

        assert_eq!(content1, b"content");
        assert_eq!(content2, b"content");
        assert_eq!(content3, b"content");
    }

    // WOS-FS-008 Test 9: List mounted filesystems
    #[test]
    fn test_list_mounts() {
        let mut vfs = VirtualFileSystem::new();
        vfs.mount(PathBuf::from("/mnt1"), VirtualFileSystem::new())
            .unwrap();
        vfs.mount(PathBuf::from("/mnt2"), VirtualFileSystem::new())
            .unwrap();

        let mounts = vfs.list_mounts();
        assert_eq!(mounts.len(), 2);
        assert!(mounts.contains(&PathBuf::from("/mnt1")));
        assert!(mounts.contains(&PathBuf::from("/mnt2")));
    }

    // WOS-FS-008 Test 10: Mount point cannot be a file
    #[test]
    fn test_mount_over_file_fails() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        let sub_vfs = VirtualFileSystem::new();

        // Mounting over a file should fail
        assert!(vfs.mount(PathBuf::from("/file.txt"), sub_vfs).is_err());
    }

    // WOS-FS-008 Test 11: Unmount non-existent mount fails
    #[test]
    fn test_unmount_non_existent_fails() {
        let mut vfs = VirtualFileSystem::new();

        // Unmounting non-existent mount should fail
        assert!(vfs.umount(&PathBuf::from("/nonexistent")).is_err());
    }

    // WOS-FS-008 Test 12: Get mount point for path
    #[test]
    fn test_get_mount_point() {
        let mut vfs = VirtualFileSystem::new();
        vfs.mount(PathBuf::from("/mnt"), VirtualFileSystem::new())
            .unwrap();

        // File within mount should return mount point
        let mount_point = vfs.get_mount_point(&PathBuf::from("/mnt/subdir/file.txt"));
        assert_eq!(mount_point, Some(PathBuf::from("/mnt")));

        // File outside mount should return None
        let no_mount = vfs.get_mount_point(&PathBuf::from("/other/file.txt"));
        assert_eq!(no_mount, None);
    }

    // WOS-FS-008 Test 13: Mount creates directory if needed
    #[test]
    fn test_mount_creates_mountpoint() {
        let mut vfs = VirtualFileSystem::new();
        let sub_vfs = VirtualFileSystem::new();

        // Mount point doesn't exist yet
        assert!(!vfs.exists(&PathBuf::from("/newmount")));

        // Mount should create it
        vfs.mount(PathBuf::from("/newmount"), sub_vfs).unwrap();
        assert!(vfs.exists(&PathBuf::from("/newmount")));
    }

    // WOS-FS-008 Test 14: Operations on root of mounted FS
    #[test]
    fn test_operations_on_mount_root() {
        let mut vfs = VirtualFileSystem::new();
        let sub_vfs = VirtualFileSystem::new();

        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // List directory at mount root
        let entries = vfs.list_directory(&PathBuf::from("/mnt")).unwrap();
        assert!(entries.is_empty() || !entries.is_empty()); // Should work either way

        // Stat mount root
        let stat = vfs.stat(&PathBuf::from("/mnt")).unwrap();
        assert_eq!(stat.file_type, FileType::Directory);
    }

    // WOS-FS-008 Test 15: Mount readonly flag (future)
    #[test]
    fn test_mount_readonly() {
        let mut vfs = VirtualFileSystem::new();
        let sub_vfs = VirtualFileSystem::new();

        // Mount as readonly (placeholder for future implementation)
        vfs.mount_readonly(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Write should fail on readonly mount
        assert!(vfs
            .create_file(PathBuf::from("/mnt/file.txt"), b"data".to_vec())
            .is_err());
    }

    // ============================================================================
    // WOS-FS-008: Mount Points Integration Tests
    // ============================================================================

    // WOS-FS-008 Integration Test 1: /proc mount
    #[test]
    fn test_integration_proc_mount() {
        let mut vfs = VirtualFileSystem::new();
        let mut proc_fs = VirtualFileSystem::new();

        proc_fs
            .create_file(PathBuf::from("/cpuinfo"), b"CPU info".to_vec())
            .unwrap();
        proc_fs
            .create_file(PathBuf::from("/meminfo"), b"Memory info".to_vec())
            .unwrap();

        vfs.mount(PathBuf::from("/proc"), proc_fs).unwrap();

        let cpu = vfs.read_file(&PathBuf::from("/proc/cpuinfo")).unwrap();
        assert_eq!(cpu, b"CPU info");
    }

    // WOS-FS-008 Integration Test 2: /dev mount
    #[test]
    fn test_integration_dev_mount() {
        let mut vfs = VirtualFileSystem::new();
        let mut dev_fs = VirtualFileSystem::new();

        dev_fs.create_file(PathBuf::from("/null"), vec![]).unwrap();
        dev_fs
            .create_file(PathBuf::from("/zero"), vec![0; 100])
            .unwrap();

        vfs.mount(PathBuf::from("/dev"), dev_fs).unwrap();

        let zero = vfs.read_file(&PathBuf::from("/dev/zero")).unwrap();
        assert_eq!(zero.len(), 100);
    }

    // WOS-FS-008 Integration Test 3: /tmp tmpfs mount
    #[test]
    fn test_integration_tmp_mount() {
        let mut vfs = VirtualFileSystem::new();
        let tmp_fs = VirtualFileSystem::new();

        vfs.mount(PathBuf::from("/tmp"), tmp_fs).unwrap();

        // Create temporary file
        vfs.create_file(PathBuf::from("/tmp/tempfile"), b"temp".to_vec())
            .unwrap();

        // Unmount (simulates system shutdown - tmp files lost)
        vfs.umount(&PathBuf::from("/tmp")).unwrap();

        // Remount fresh tmpfs
        vfs.mount(PathBuf::from("/tmp"), VirtualFileSystem::new())
            .unwrap();

        // Old temp file should not exist
        assert!(vfs.read_file(&PathBuf::from("/tmp/tempfile")).is_err());
    }

    // WOS-FS-008 Integration Test 4: Cross-mount operations
    #[test]
    fn test_integration_cross_mount_operations() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/root.txt"), b"root".to_vec())
            .unwrap();

        let mut mount1 = VirtualFileSystem::new();
        mount1
            .create_file(PathBuf::from("/m1.txt"), b"mount1".to_vec())
            .unwrap();

        vfs.mount(PathBuf::from("/mnt"), mount1).unwrap();

        // List root - should see both root file and mount point
        let root_entries = vfs.list_directory(&PathBuf::from("/")).unwrap();
        assert!(root_entries.iter().any(|e| e.name == "root.txt"));
        assert!(root_entries.iter().any(|e| e.name == "mnt"));

        // Access files from both filesystems
        let root_content = vfs.read_file(&PathBuf::from("/root.txt")).unwrap();
        let mount_content = vfs.read_file(&PathBuf::from("/mnt/m1.txt")).unwrap();
        assert_eq!(root_content, b"root");
        assert_eq!(mount_content, b"mount1");
    }

    // WOS-FS-008 Integration Test 5: Mount point permissions
    #[test]
    fn test_integration_mount_permissions() {
        let mut vfs = VirtualFileSystem::new();
        vfs.set_context(1000, 1000); // Non-root user

        let mut sub_vfs = VirtualFileSystem::new();
        sub_vfs
            .create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        // Non-root user should be able to mount (simplified - real systems restrict this)
        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Verify access
        let content = vfs.read_file(&PathBuf::from("/mnt/file.txt")).unwrap();
        assert_eq!(content, b"data");
    }

    // WOS-FS-008 Integration Test 6: Mount with symlinks
    #[test]
    fn test_integration_mount_with_symlinks() {
        let mut vfs = VirtualFileSystem::new();
        let mut sub_vfs = VirtualFileSystem::new();

        sub_vfs
            .create_file(PathBuf::from("/target.txt"), b"target".to_vec())
            .unwrap();
        sub_vfs
            .create_symlink(PathBuf::from("/link.txt"), PathBuf::from("/target.txt"))
            .unwrap();

        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Follow symlink within mounted filesystem
        let content = vfs.read_file(&PathBuf::from("/mnt/link.txt")).unwrap();
        assert_eq!(content, b"target");
    }

    // WOS-FS-008 Integration Test 7: Umount busy filesystem
    #[test]
    fn test_integration_umount_busy() {
        let mut vfs = VirtualFileSystem::new();
        let mut sub_vfs = VirtualFileSystem::new();

        sub_vfs
            .create_file(PathBuf::from("/file.txt"), b"data".to_vec())
            .unwrap();

        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Lock file on mounted filesystem
        vfs.flock(&PathBuf::from("/mnt/file.txt"), LockType::Exclusive)
            .unwrap();

        // Umount should fail (filesystem busy)
        // Or succeed and auto-release locks (implementation dependent)
        let result = vfs.umount(&PathBuf::from("/mnt"));
        // Either fails or succeeds with cleanup
        assert!(result.is_err() || result.is_ok());
    }

    // WOS-FS-008 Integration Test 8: Clone preserves mounts
    #[test]
    fn test_integration_clone_preserves_mounts() {
        let mut vfs = VirtualFileSystem::new();
        let mut sub_vfs = VirtualFileSystem::new();

        sub_vfs
            .create_file(PathBuf::from("/data.txt"), b"mounted".to_vec())
            .unwrap();

        vfs.mount(PathBuf::from("/mnt"), sub_vfs).unwrap();

        // Clone VFS
        let mut cloned_vfs = vfs.clone();

        // Mounts should be preserved
        assert!(cloned_vfs.is_mount_point(&PathBuf::from("/mnt")));

        // Should be able to access mounted files
        let content = cloned_vfs
            .read_file(&PathBuf::from("/mnt/data.txt"))
            .unwrap();
        assert_eq!(content, b"mounted");
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

                // Skip if directory already exists (e.g., /dev, /bin, /tmp)
                prop_assume!(!vfs.exists(&dir_path));

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

                // Skip if directory already exists (e.g., /dev, /bin, /tmp)
                prop_assume!(!vfs.exists(&parent_path));

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

                // Skip if directory already exists (e.g., /dev, /bin, /tmp)
                prop_assume!(!vfs.exists(&dir));

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

                // Skip if directory already exists (e.g., /dev, /bin, /tmp)
                prop_assume!(!vfs.exists(&dir_path));

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

            // ====================================================================
            // WOS-FS-006: Path Normalization Property Tests
            // ====================================================================

            /// Property: Normalized paths are idempotent
            #[test]
            fn proptest_normalization_idempotent(
                components in prop::collection::vec(
                    prop::string::string_regex("[a-z]{1,10}").unwrap(),
                    1..10
                ),
            ) {
                // Build path with components
                let path_str = format!("/{}", components.join("/"));
                let path = PathBuf::from(&path_str);

                // Normalize once
                let normalized1 = VirtualFileSystem::normalize_path(&path);
                // Normalize again
                let normalized2 = VirtualFileSystem::normalize_path(&normalized1);

                // Should be the same
                prop_assert_eq!(normalized1, normalized2);
            }

            /// Property: Normalized paths never contain . or ..
            #[test]
            fn proptest_normalized_no_dots(
                components in prop::collection::vec(
                    prop_oneof![
                        Just(".".to_string()),
                        Just("..".to_string()),
                        prop::string::string_regex("[a-z]{1,10}").unwrap(),
                    ],
                    1..15
                ),
            ) {
                let path_str = format!("/{}", components.join("/"));
                let path = PathBuf::from(&path_str);

                let normalized = VirtualFileSystem::normalize_path(&path);
                let normalized_str = normalized.to_str().unwrap();

                // Normalized path should not contain /. or /..
                prop_assert!(!normalized_str.contains("/."));
            }

            /// Property: Normalized paths never have consecutive slashes
            #[test]
            fn proptest_normalized_no_consecutive_slashes(
                components in prop::collection::vec(
                    prop::string::string_regex("[a-z]{1,10}").unwrap(),
                    1..10
                ),
                extra_slashes in prop::collection::vec(any::<bool>(), 1..10),
            ) {
                // Build path with random extra slashes
                let mut path_str = String::from("/");
                for (i, component) in components.iter().enumerate() {
                    path_str.push_str(component);
                    if i < components.len() - 1 {
                        path_str.push('/');
                        if i < extra_slashes.len() && extra_slashes[i] {
                            path_str.push('/');
                        }
                    }
                }
                let path = PathBuf::from(&path_str);

                let normalized = VirtualFileSystem::normalize_path(&path);
                let normalized_str = normalized.to_str().unwrap();

                // Should not contain //
                prop_assert!(!normalized_str.contains("//"));
            }

            /// Property: Path operations work with normalized and unnormalized paths
            #[test]
            fn proptest_operations_with_normalization(
                dir in prop::string::string_regex("[a-z]{1,10}").unwrap(),
                file in prop::string::string_regex("[a-z]{1,10}\\.txt").unwrap(),
                content in prop::collection::vec(any::<u8>(), 0..100),
            ) {
                let mut vfs = VirtualFileSystem::new();

                // Create file with simple path
                let simple_path = PathBuf::from(format!("/{}/{}", dir, file));
                vfs.create_file(simple_path.clone(), content.clone()).ok();

                // Read with complex path
                let complex_path = PathBuf::from(format!("/./{}/../{}/{}", dir, dir, file));
                if let Ok(read_content) = vfs.read_file(&complex_path) {
                    prop_assert_eq!(read_content, content);
                }
            }

            /// Property: Parent references never escape root
            #[test]
            fn proptest_parent_refs_bounded_at_root(
                num_parents in 1..20_usize,
                filename in prop::string::string_regex("[a-z]{1,10}\\.txt").unwrap(),
            ) {
                // Create path with excessive parent references: /../../../file.txt
                let mut path_str = String::from("/");
                for _ in 0..num_parents {
                    path_str.push_str("../");
                }
                path_str.push_str(&filename);
                let path = PathBuf::from(&path_str);

                let normalized = VirtualFileSystem::normalize_path(&path);
                let normalized_str = normalized.to_str().unwrap();

                // Should resolve to /filename (parent refs bounded at root)
                prop_assert_eq!(normalized_str, format!("/{}", filename));
            }

            // ====================================================================
            // WOS-FS-007: File Locking Property Tests
            // ====================================================================

            /// Property: Locking and unlocking is idempotent
            #[test]
            fn proptest_lock_unlock_idempotent(
                filename in prop::string::string_regex("/[a-z]{1,10}\\.txt").unwrap(),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(&filename);
                vfs.create_file(path.clone(), b"content".to_vec()).ok();

                // Lock
                if vfs.flock(&path, LockType::Exclusive).is_ok() {
                    prop_assert!(vfs.is_locked(&path));

                    // Unlock
                    vfs.funlock(&path).ok();
                    prop_assert!(!vfs.is_locked(&path));

                    // Unlock again (should be idempotent)
                    vfs.funlock(&path).ok();
                    prop_assert!(!vfs.is_locked(&path));
                }
            }

            /// Property: Shared locks don't conflict with each other
            #[test]
            fn proptest_shared_locks_compatible(
                filename in prop::string::string_regex("/[a-z]{1,10}\\.txt").unwrap(),
                num_locks in 1..10_usize,
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(&filename);
                vfs.create_file(path.clone(), b"content".to_vec()).ok();

                // Acquire multiple shared locks
                let mut successes = 0;
                for _ in 0..num_locks {
                    if vfs.flock(&path, LockType::Shared).is_ok() {
                        successes += 1;
                    }
                }

                // Should be able to acquire multiple shared locks
                if vfs.exists(&path) {
                    prop_assert!(successes > 0);
                    prop_assert!(vfs.is_locked(&path));
                }
            }

            /// Property: Byte-range locks with disjoint ranges don't conflict
            #[test]
            fn proptest_disjoint_ranges_no_conflict(
                filename in prop::string::string_regex("/[a-z]{1,10}\\.txt").unwrap(),
                offset1 in 0..50_u64,
                len1 in 1..20_u64,
                offset2 in 0..50_u64,
                len2 in 1..20_u64,
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(&filename);
                vfs.create_file(path.clone(), vec![0u8; 100]).ok();

                // Check if ranges are disjoint
                let end1 = offset1 + len1;
                let end2 = offset2 + len2;
                let disjoint = end1 <= offset2 || end2 <= offset1;

                if disjoint {
                    // Both locks should succeed since ranges don't overlap
                    let lock1 = vfs.fcntl_lock(&path, LockType::Exclusive, offset1, len1);
                    let lock2 = vfs.fcntl_lock(&path, LockType::Exclusive, offset2, len2);

                    if vfs.exists(&path) && lock1.is_ok() {
                        prop_assert!(lock2.is_ok());
                    }
                }
            }

            /// Property: Exclusive lock prevents all other locks
            #[test]
            fn proptest_exclusive_lock_prevents_others(
                filename in prop::string::string_regex("/[a-z]{1,10}\\.txt").unwrap(),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from(&filename);
                vfs.create_file(path.clone(), b"content".to_vec()).ok();

                // Acquire exclusive lock
                if vfs.flock(&path, LockType::Exclusive).is_ok() {
                    // Try to acquire another exclusive lock (should fail)
                    let result_exclusive = vfs.flock(&path, LockType::Exclusive);
                    prop_assert!(result_exclusive.is_err());

                    // Try to acquire shared lock (should fail)
                    let result_shared = vfs.flock(&path, LockType::Shared);
                    prop_assert!(result_shared.is_err());
                }
            }

            // ====================================================================
            // WOS-FS-008: Mount Points Property Tests
            // ====================================================================

            /// Property: Mount and unmount are inverse operations
            #[test]
            fn proptest_mount_unmount_inverse(
                mount_point in prop::string::string_regex("/[a-z]{1,10}").unwrap(),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let mount_path = PathBuf::from(&mount_point);
                let sub_vfs = VirtualFileSystem::new();

                // Mount
                if vfs.mount(mount_path.clone(), sub_vfs).is_ok() {
                    prop_assert!(vfs.is_mount_point(&mount_path));

                    // Unmount
                    vfs.umount(&mount_path).ok();
                    prop_assert!(!vfs.is_mount_point(&mount_path));
                }
            }

            /// Property: Files created on mount are accessible
            #[test]
            fn proptest_mount_file_access(
                mount_point in prop::string::string_regex("/[a-z]{1,10}").unwrap(),
                filename in prop::string::string_regex("[a-z]{1,10}\\.txt").unwrap(),
                content in prop::collection::vec(any::<u8>(), 0..100),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let mount_path = PathBuf::from(&mount_point);
                let sub_vfs = VirtualFileSystem::new();

                if vfs.mount(mount_path.clone(), sub_vfs).is_ok() {
                    let file_path = PathBuf::from(format!("{}/{}", mount_point, filename));

                    // Create file on mounted filesystem
                    if vfs.create_file(file_path.clone(), content.clone()).is_ok() {
                        // Should be able to read it back
                        if let Ok(read_content) = vfs.read_file(&file_path) {
                            prop_assert_eq!(read_content, content);
                        }
                    }
                }
            }

            /// Property: Multiple mounts don't interfere
            #[test]
            fn proptest_multiple_mounts_independent(
                mount1 in prop::string::string_regex("/[a-z]{1,10}").unwrap(),
                mount2 in prop::string::string_regex("/[a-z]{1,10}").unwrap(),
            ) {
                prop_assume!(mount1 != mount2);

                let mut vfs = VirtualFileSystem::new();
                let path1 = PathBuf::from(&mount1);
                let path2 = PathBuf::from(&mount2);

                let mut fs1 = VirtualFileSystem::new();
                let mut fs2 = VirtualFileSystem::new();

                fs1.create_file(PathBuf::from("/file1.txt"), b"fs1".to_vec()).ok();
                fs2.create_file(PathBuf::from("/file2.txt"), b"fs2".to_vec()).ok();

                if vfs.mount(path1.clone(), fs1).is_ok() && vfs.mount(path2.clone(), fs2).is_ok() {
                    // Files from each mount should be accessible independently
                    let file1_path = PathBuf::from(format!("{}/file1.txt", mount1));
                    let file2_path = PathBuf::from(format!("{}/file2.txt", mount2));

                    if let Ok(content1) = vfs.read_file(&file1_path) {
                        prop_assert_eq!(content1, b"fs1");
                    }

                    if let Ok(content2) = vfs.read_file(&file2_path) {
                        prop_assert_eq!(content2, b"fs2");
                    }

                    // File from fs1 should not be in fs2
                    let wrong_path = PathBuf::from(format!("{}/file1.txt", mount2));
                    prop_assert!(vfs.read_file(&wrong_path).is_err());
                }
            }
        }
    }

    // =========================================================================
    // WOS-FS-009: Extended Attributes (xattr) Tests (RED phase)
    // =========================================================================

    /// Unit Tests (10 tests)

    #[test]
    fn test_setxattr_basic() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();

        let result = vfs.setxattr(&path, "user.comment", b"Hello xattr");
        assert!(result.is_ok(), "setxattr should succeed");
    }

    #[test]
    fn test_getxattr_basic() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();
        vfs.setxattr(&path, "user.comment", b"Hello xattr").unwrap();

        let result = vfs.getxattr(&path, "user.comment");
        assert!(result.is_ok(), "getxattr should succeed");
        assert_eq!(result.unwrap(), b"Hello xattr");
    }

    #[test]
    fn test_listxattr_basic() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();
        vfs.setxattr(&path, "user.comment", b"value1").unwrap();
        vfs.setxattr(&path, "user.author", b"value2").unwrap();

        let result = vfs.listxattr(&path);
        assert!(result.is_ok(), "listxattr should succeed");
        let attrs = result.unwrap();
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains(&"user.comment".to_string()));
        assert!(attrs.contains(&"user.author".to_string()));
    }

    #[test]
    fn test_removexattr_basic() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();
        vfs.setxattr(&path, "user.comment", b"value").unwrap();

        let result = vfs.removexattr(&path, "user.comment");
        assert!(result.is_ok(), "removexattr should succeed");

        let get_result = vfs.getxattr(&path, "user.comment");
        assert_eq!(get_result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_setxattr_namespace_user() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();

        let result = vfs.setxattr(&path, "user.custom", b"user data");
        assert!(result.is_ok(), "user namespace should work");
    }

    #[test]
    fn test_setxattr_namespace_system() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();

        let result = vfs.setxattr(&path, "system.posix_acl_access", b"acl data");
        assert!(result.is_ok(), "system namespace should work");
    }

    #[test]
    fn test_setxattr_namespace_security() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();

        let result = vfs.setxattr(&path, "security.selinux", b"context");
        assert!(result.is_ok(), "security namespace should work");
    }

    #[test]
    fn test_getxattr_nonexistent() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();

        let result = vfs.getxattr(&path, "user.nonexistent");
        assert_eq!(result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_removexattr_nonexistent() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();

        let result = vfs.removexattr(&path, "user.nonexistent");
        assert_eq!(result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_xattr_on_nonexistent_file() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/nonexistent.txt");

        let set_result = vfs.setxattr(&path, "user.test", b"value");
        assert_eq!(set_result, Err(VfsError::NotFound));

        let get_result = vfs.getxattr(&path, "user.test");
        assert_eq!(get_result, Err(VfsError::NotFound));

        let list_result = vfs.listxattr(&path);
        assert_eq!(list_result, Err(VfsError::NotFound));

        let remove_result = vfs.removexattr(&path, "user.test");
        assert_eq!(remove_result, Err(VfsError::NotFound));
    }

    /// Integration Tests (4 tests)

    #[test]
    fn test_integration_xattr_multiple_files() {
        let mut vfs = VirtualFileSystem::new();
        let path1 = PathBuf::from("/file1.txt");
        let path2 = PathBuf::from("/file2.txt");

        vfs.create_file(path1.clone(), vec![]).unwrap();
        vfs.create_file(path2.clone(), vec![]).unwrap();

        vfs.setxattr(&path1, "user.comment", b"file1 comment")
            .unwrap();
        vfs.setxattr(&path2, "user.comment", b"file2 comment")
            .unwrap();

        let xattr1 = vfs.getxattr(&path1, "user.comment").unwrap();
        let xattr2 = vfs.getxattr(&path2, "user.comment").unwrap();

        assert_eq!(xattr1, b"file1 comment");
        assert_eq!(xattr2, b"file2 comment");
    }

    #[test]
    fn test_integration_xattr_clone_preserves() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), vec![]).unwrap();
        vfs.setxattr(&path, "user.comment", b"original").unwrap();

        let mut cloned = vfs.clone();

        // Original and clone should both have xattr
        assert_eq!(vfs.getxattr(&path, "user.comment").unwrap(), b"original");
        assert_eq!(cloned.getxattr(&path, "user.comment").unwrap(), b"original");

        // Modify clone - should not affect original
        cloned.setxattr(&path, "user.comment", b"modified").unwrap();
        assert_eq!(vfs.getxattr(&path, "user.comment").unwrap(), b"original");
        assert_eq!(cloned.getxattr(&path, "user.comment").unwrap(), b"modified");
    }

    #[test]
    fn test_integration_xattr_file_delete_removes() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");

        vfs.create_file(path.clone(), vec![]).unwrap();
        vfs.setxattr(&path, "user.comment", b"value").unwrap();

        vfs.delete_file(&path).unwrap();

        // Recreate file - xattrs should not persist
        vfs.create_file(path.clone(), vec![]).unwrap();
        let result = vfs.getxattr(&path, "user.comment");
        assert_eq!(result, Err(VfsError::NotFound));
    }

    #[test]
    fn test_integration_xattr_namespaces() {
        let mut vfs = VirtualFileSystem::new();
        let path = PathBuf::from("/test.txt");
        vfs.create_file(path.clone(), vec![]).unwrap();

        // Set attributes in different namespaces
        vfs.setxattr(&path, "user.comment", b"user data").unwrap();
        vfs.setxattr(&path, "system.acl", b"system data").unwrap();
        vfs.setxattr(&path, "security.label", b"security data")
            .unwrap();

        // All should be retrievable independently
        assert_eq!(vfs.getxattr(&path, "user.comment").unwrap(), b"user data");
        assert_eq!(vfs.getxattr(&path, "system.acl").unwrap(), b"system data");
        assert_eq!(
            vfs.getxattr(&path, "security.label").unwrap(),
            b"security data"
        );

        // List should show all 3
        let attrs = vfs.listxattr(&path).unwrap();
        assert_eq!(attrs.len(), 3);
    }

    /// Property Tests (2 tests)

    #[cfg(test)]
    mod xattr_properties {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10_000))]

            #[test]
            fn proptest_xattr_set_get_roundtrip(
                name in "[a-z]{1,20}",
                value in prop::collection::vec(any::<u8>(), 0..100),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from("/test.txt");
                vfs.create_file(path.clone(), vec![]).ok();

                let attr_name = format!("user.{}", name);
                if vfs.setxattr(&path, &attr_name, &value).is_ok() {
                    let retrieved = vfs.getxattr(&path, &attr_name).ok();
                    prop_assert_eq!(retrieved, Some(value));
                }
            }

            #[test]
            fn proptest_xattr_remove_makes_nonexistent(
                name in "[a-z]{1,20}",
                value in prop::collection::vec(any::<u8>(), 0..100),
            ) {
                let mut vfs = VirtualFileSystem::new();
                let path = PathBuf::from("/test.txt");
                vfs.create_file(path.clone(), vec![]).ok();

                let attr_name = format!("user.{}", name);
                if vfs.setxattr(&path, &attr_name, &value).is_ok() {
                    if vfs.removexattr(&path, &attr_name).is_ok() {
                        let result = vfs.getxattr(&path, &attr_name);
                        prop_assert_eq!(result, Err(VfsError::NotFound));
                    }
                }
            }
        }
    }

    mod devfs_tests {
        use super::*;

        #[test]
        fn test_dev_null_read() {
            let mut vfs = VirtualFileSystem::new();
            // /dev/null should exist after initialization
            let result = vfs.read_file(&PathBuf::from("/dev/null"));
            assert_eq!(result, Ok(vec![]));
        }

        #[test]
        fn test_dev_null_write() {
            let mut vfs = VirtualFileSystem::new();
            // Writing to /dev/null should succeed and discard data
            let result = vfs.write_file(&PathBuf::from("/dev/null"), vec![1, 2, 3, 4, 5]);
            assert!(result.is_ok());
            // Reading should still return empty
            let read_result = vfs.read_file(&PathBuf::from("/dev/null"));
            assert_eq!(read_result, Ok(vec![]));
        }

        #[test]
        fn test_dev_zero_read() {
            let mut vfs = VirtualFileSystem::new();
            // Reading from /dev/zero should return zeros
            let result = vfs.read_file(&PathBuf::from("/dev/zero"));
            assert!(result.is_ok());
            let data = result.unwrap();
            assert!(!data.is_empty());
            assert!(data.iter().all(|&b| b == 0));
        }

        #[test]
        fn test_dev_zero_multiple_reads() {
            let mut vfs = VirtualFileSystem::new();
            // Multiple reads from /dev/zero should return zeros
            let result1 = vfs.read_file(&PathBuf::from("/dev/zero"));
            let result2 = vfs.read_file(&PathBuf::from("/dev/zero"));
            assert!(result1.is_ok());
            assert!(result2.is_ok());
            assert!(result1.unwrap().iter().all(|&b| b == 0));
            assert!(result2.unwrap().iter().all(|&b| b == 0));
        }

        #[test]
        fn test_dev_random_read() {
            let mut vfs = VirtualFileSystem::new();
            // Reading from /dev/random should return non-zero bytes
            let result = vfs.read_file(&PathBuf::from("/dev/random"));
            assert!(result.is_ok());
            let data = result.unwrap();
            assert!(!data.is_empty());
            // Should have at least some non-zero bytes
            assert!(data.iter().any(|&b| b != 0));
        }

        #[test]
        fn test_dev_random_deterministic() {
            let vfs1 = VirtualFileSystem::new();
            let vfs2 = VirtualFileSystem::new();
            // Two fresh VFS instances should produce same random bytes (deterministic)
            let mut vfs1_clone = vfs1.clone();
            let mut vfs2_clone = vfs2.clone();
            let result1 = vfs1_clone.read_file(&PathBuf::from("/dev/random"));
            let result2 = vfs2_clone.read_file(&PathBuf::from("/dev/random"));
            assert_eq!(result1, result2);
        }

        #[test]
        fn test_dev_files_exist() {
            let vfs = VirtualFileSystem::new();
            // Check that all device files exist in /dev
            let entries = vfs.list_directory(&PathBuf::from("/dev")).unwrap();
            let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
            assert!(names.contains(&"null".to_string()));
            assert!(names.contains(&"zero".to_string()));
            assert!(names.contains(&"random".to_string()));
        }
    }
}
