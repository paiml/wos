//! Type-Safe Kernel Primitives
//!
//! This module provides type-safe wrappers for kernel identifiers and addresses.
//! Following the poka-yoke (mistake-proofing) principle, these types prevent
//! common errors like using a ProcessId where a FileDescriptor is expected.
//!
//! # Design Philosophy
//!
//! - **Type Safety**: Distinct types for distinct concepts
//! - **Zero Cost**: Newtypes compile to bare values
//! - **Serializable**: All types implement Serialize/Deserialize
//! - **Hashable**: All types can be used as HashMap keys

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Page size constant (4 KiB)
pub const PAGE_SIZE: u64 = 4096;

/// Maximum PID value
pub const MAX_PID: u32 = 65535;

/// Maximum file descriptor value
pub const MAX_FD: u32 = 1024;

/// Maximum VM ID value
pub const MAX_VM_ID: u32 = 256;

// ============================================================================
// Address Types
// ============================================================================

/// Virtual address in process address space
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Create a new virtual address
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Get the raw address value
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Get the page offset (lower 12 bits)
    #[inline]
    pub const fn page_offset(&self) -> u64 {
        self.0 & (PAGE_SIZE - 1)
    }

    /// Get the page number
    #[inline]
    pub const fn page_number(&self) -> Pfn {
        Pfn(self.0 / PAGE_SIZE)
    }

    /// Check if address is page-aligned
    #[inline]
    pub const fn is_page_aligned(&self) -> bool {
        self.page_offset() == 0
    }

    /// Align address down to page boundary
    #[inline]
    pub const fn page_align_down(&self) -> Self {
        Self(self.0 & !(PAGE_SIZE - 1))
    }

    /// Align address up to page boundary
    #[inline]
    pub const fn page_align_up(&self) -> Self {
        Self((self.0 + PAGE_SIZE - 1) & !(PAGE_SIZE - 1))
    }

    /// Add offset to address
    #[inline]
    pub const fn offset(&self, offset: u64) -> Self {
        Self(self.0.wrapping_add(offset))
    }

    /// Subtract offset from address
    #[inline]
    pub const fn sub_offset(&self, offset: u64) -> Self {
        Self(self.0.wrapping_sub(offset))
    }

    /// Calculate distance between addresses
    #[inline]
    pub fn distance(&self, other: &Self) -> u64 {
        self.0.abs_diff(other.0)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl From<u64> for VirtAddr {
    fn from(addr: u64) -> Self {
        Self(addr)
    }
}

impl From<VirtAddr> for u64 {
    fn from(addr: VirtAddr) -> Self {
        addr.0
    }
}

/// Physical address in system memory
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Create a new physical address
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Get the raw address value
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Get the page offset (lower 12 bits)
    #[inline]
    pub const fn page_offset(&self) -> u64 {
        self.0 & (PAGE_SIZE - 1)
    }

    /// Get the page frame number
    #[inline]
    pub const fn page_frame_number(&self) -> Pfn {
        Pfn(self.0 / PAGE_SIZE)
    }

    /// Check if address is page-aligned
    #[inline]
    pub const fn is_page_aligned(&self) -> bool {
        self.page_offset() == 0
    }

    /// Align address down to page boundary
    #[inline]
    pub const fn page_align_down(&self) -> Self {
        Self(self.0 & !(PAGE_SIZE - 1))
    }

    /// Align address up to page boundary
    #[inline]
    pub const fn page_align_up(&self) -> Self {
        Self((self.0 + PAGE_SIZE - 1) & !(PAGE_SIZE - 1))
    }

    /// Add offset to address
    #[inline]
    pub const fn offset(&self, offset: u64) -> Self {
        Self(self.0.wrapping_add(offset))
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl From<u64> for PhysAddr {
    fn from(addr: u64) -> Self {
        Self(addr)
    }
}

impl From<PhysAddr> for u64 {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

/// Page Frame Number (physical page identifier)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Pfn(u64);

impl Pfn {
    /// Create a new page frame number
    #[inline]
    pub const fn new(pfn: u64) -> Self {
        Self(pfn)
    }

    /// Get the raw PFN value
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Convert to physical address (start of page)
    #[inline]
    pub const fn to_phys_addr(&self) -> PhysAddr {
        PhysAddr(self.0 * PAGE_SIZE)
    }

    /// Get next page frame number
    #[inline]
    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    /// Get previous page frame number
    #[inline]
    pub const fn prev(&self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self(self.0 - 1))
        } else {
            None
        }
    }
}

impl fmt::Display for Pfn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PFN:{}", self.0)
    }
}

impl From<u64> for Pfn {
    fn from(pfn: u64) -> Self {
        Self(pfn)
    }
}

// ============================================================================
// Identifier Types
// ============================================================================

/// Process identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ProcessId(u32);

impl ProcessId {
    /// Invalid PID constant
    pub const INVALID: Self = Self(0);

    /// Init process PID
    pub const INIT: Self = Self(1);

    /// Shell process PID (conventional)
    pub const SHELL: Self = Self(2);

    /// Create a new process ID
    #[inline]
    pub const fn new(pid: u32) -> Self {
        Self(pid)
    }

    /// Get the raw PID value
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    /// Check if this is a valid PID
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 > 0 && self.0 <= MAX_PID
    }

    /// Check if this is the init process
    #[inline]
    pub const fn is_init(&self) -> bool {
        self.0 == 1
    }

    /// Get next PID
    #[inline]
    pub const fn next(&self) -> Option<Self> {
        if self.0 < MAX_PID {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }
}

impl Default for ProcessId {
    fn default() -> Self {
        Self::INVALID
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PID:{}", self.0)
    }
}

impl From<u32> for ProcessId {
    fn from(pid: u32) -> Self {
        Self(pid)
    }
}

impl From<ProcessId> for u32 {
    fn from(pid: ProcessId) -> Self {
        pid.0
    }
}

/// File descriptor
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct FileDescriptor(u32);

impl FileDescriptor {
    /// Standard input
    pub const STDIN: Self = Self(0);

    /// Standard output
    pub const STDOUT: Self = Self(1);

    /// Standard error
    pub const STDERR: Self = Self(2);

    /// Create a new file descriptor
    #[inline]
    pub const fn new(fd: u32) -> Self {
        Self(fd)
    }

    /// Get the raw FD value
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    /// Check if this is a standard stream
    #[inline]
    pub const fn is_standard(&self) -> bool {
        self.0 <= 2
    }

    /// Check if FD is valid
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 < MAX_FD
    }

    /// Get next FD
    #[inline]
    pub const fn next(&self) -> Option<Self> {
        if self.0 < MAX_FD - 1 {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }
}

impl fmt::Display for FileDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => write!(f, "stdin"),
            1 => write!(f, "stdout"),
            2 => write!(f, "stderr"),
            fd => write!(f, "fd:{}", fd),
        }
    }
}

impl From<u32> for FileDescriptor {
    fn from(fd: u32) -> Self {
        Self(fd)
    }
}

impl From<FileDescriptor> for u32 {
    fn from(fd: FileDescriptor) -> Self {
        fd.0
    }
}

/// Virtual Machine identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct VmId(u32);

impl VmId {
    /// Invalid VM ID
    pub const INVALID: Self = Self(0);

    /// Create a new VM ID
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    /// Check if VM ID is valid
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 > 0 && self.0 <= MAX_VM_ID
    }

    /// Get next VM ID
    #[inline]
    pub const fn next(&self) -> Option<Self> {
        if self.0 < MAX_VM_ID {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }
}

impl Default for VmId {
    fn default() -> Self {
        Self::INVALID
    }
}

impl fmt::Display for VmId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VM:{}", self.0)
    }
}

impl From<u32> for VmId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

// ============================================================================
// Signal Types
// ============================================================================

/// POSIX-like signal number
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Signal(i32);

impl Signal {
    /// Hangup
    pub const SIGHUP: Self = Self(1);
    /// Interrupt (Ctrl+C)
    pub const SIGINT: Self = Self(2);
    /// Quit
    pub const SIGQUIT: Self = Self(3);
    /// Illegal instruction
    pub const SIGILL: Self = Self(4);
    /// Trap
    pub const SIGTRAP: Self = Self(5);
    /// Abort
    pub const SIGABRT: Self = Self(6);
    /// Bus error
    pub const SIGBUS: Self = Self(7);
    /// Floating point exception
    pub const SIGFPE: Self = Self(8);
    /// Kill (cannot be caught)
    pub const SIGKILL: Self = Self(9);
    /// User signal 1
    pub const SIGUSR1: Self = Self(10);
    /// Segmentation violation
    pub const SIGSEGV: Self = Self(11);
    /// User signal 2
    pub const SIGUSR2: Self = Self(12);
    /// Broken pipe
    pub const SIGPIPE: Self = Self(13);
    /// Alarm
    pub const SIGALRM: Self = Self(14);
    /// Termination
    pub const SIGTERM: Self = Self(15);
    /// Child status change
    pub const SIGCHLD: Self = Self(17);
    /// Continue
    pub const SIGCONT: Self = Self(18);
    /// Stop
    pub const SIGSTOP: Self = Self(19);
    /// Terminal stop
    pub const SIGTSTP: Self = Self(20);

    /// Create a new signal
    #[inline]
    pub const fn new(signum: i32) -> Self {
        Self(signum)
    }

    /// Get the signal number
    #[inline]
    pub const fn as_i32(&self) -> i32 {
        self.0
    }

    /// Check if signal can be caught
    #[inline]
    pub const fn is_catchable(&self) -> bool {
        self.0 != 9 && self.0 != 19 // SIGKILL and SIGSTOP
    }

    /// Check if signal is valid
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 > 0 && self.0 < 32
    }

    /// Get signal name
    pub fn name(&self) -> &'static str {
        match self.0 {
            1 => "SIGHUP",
            2 => "SIGINT",
            3 => "SIGQUIT",
            4 => "SIGILL",
            5 => "SIGTRAP",
            6 => "SIGABRT",
            7 => "SIGBUS",
            8 => "SIGFPE",
            9 => "SIGKILL",
            10 => "SIGUSR1",
            11 => "SIGSEGV",
            12 => "SIGUSR2",
            13 => "SIGPIPE",
            14 => "SIGALRM",
            15 => "SIGTERM",
            17 => "SIGCHLD",
            18 => "SIGCONT",
            19 => "SIGSTOP",
            20 => "SIGTSTP",
            _ => "UNKNOWN",
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<i32> for Signal {
    fn from(signum: i32) -> Self {
        Self(signum)
    }
}

// ============================================================================
// Memory Protection
// ============================================================================

/// Memory protection flags
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryProtection(u8);

impl MemoryProtection {
    /// No access
    pub const NONE: Self = Self(0);
    /// Read permission
    pub const READ: Self = Self(1);
    /// Write permission
    pub const WRITE: Self = Self(2);
    /// Execute permission
    pub const EXEC: Self = Self(4);
    /// Read + Write
    pub const READ_WRITE: Self = Self(3);
    /// Read + Execute
    pub const READ_EXEC: Self = Self(5);
    /// Read + Write + Execute
    pub const READ_WRITE_EXEC: Self = Self(7);

    /// Create new protection flags
    #[inline]
    pub const fn new(flags: u8) -> Self {
        Self(flags & 0x7)
    }

    /// Get raw flags
    #[inline]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    /// Check if readable
    #[inline]
    pub const fn is_readable(&self) -> bool {
        self.0 & 1 != 0
    }

    /// Check if writable
    #[inline]
    pub const fn is_writable(&self) -> bool {
        self.0 & 2 != 0
    }

    /// Check if executable
    #[inline]
    pub const fn is_executable(&self) -> bool {
        self.0 & 4 != 0
    }

    /// Add read permission
    #[inline]
    pub const fn with_read(self) -> Self {
        Self(self.0 | 1)
    }

    /// Add write permission
    #[inline]
    pub const fn with_write(self) -> Self {
        Self(self.0 | 2)
    }

    /// Add execute permission
    #[inline]
    pub const fn with_exec(self) -> Self {
        Self(self.0 | 4)
    }
}

impl Default for MemoryProtection {
    fn default() -> Self {
        Self::NONE
    }
}

impl fmt::Display for MemoryProtection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = if self.is_readable() { 'r' } else { '-' };
        let w = if self.is_writable() { 'w' } else { '-' };
        let x = if self.is_executable() { 'x' } else { '-' };
        write!(f, "{}{}{}", r, w, x)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virt_addr_creation() {
        let addr = VirtAddr::new(0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
    }

    #[test]
    fn test_virt_addr_page_operations() {
        let addr = VirtAddr::new(0x1234);
        assert_eq!(addr.page_offset(), 0x234);
        assert_eq!(addr.page_number().as_u64(), 1);
        assert!(!addr.is_page_aligned());

        let aligned = addr.page_align_down();
        assert!(aligned.is_page_aligned());
        assert_eq!(aligned.as_u64(), 0x1000);

        let aligned_up = addr.page_align_up();
        assert!(aligned_up.is_page_aligned());
        assert_eq!(aligned_up.as_u64(), 0x2000);
    }

    #[test]
    fn test_virt_addr_arithmetic() {
        let addr = VirtAddr::new(0x1000);
        assert_eq!(addr.offset(0x100).as_u64(), 0x1100);
        assert_eq!(addr.sub_offset(0x100).as_u64(), 0x0F00);
    }

    #[test]
    fn test_phys_addr_creation() {
        let addr = PhysAddr::new(0x2000);
        assert_eq!(addr.as_u64(), 0x2000);
        assert!(addr.is_page_aligned());
    }

    #[test]
    fn test_pfn_operations() {
        let pfn = Pfn::new(5);
        assert_eq!(pfn.as_u64(), 5);
        assert_eq!(pfn.to_phys_addr().as_u64(), 5 * PAGE_SIZE);
        assert_eq!(pfn.next().as_u64(), 6);
    }

    #[test]
    fn test_process_id() {
        let pid = ProcessId::new(42);
        assert_eq!(pid.as_u32(), 42);
        assert!(pid.is_valid());
        assert!(!pid.is_init());

        assert!(ProcessId::INIT.is_init());
        assert!(!ProcessId::INVALID.is_valid());
    }

    #[test]
    fn test_file_descriptor() {
        assert!(FileDescriptor::STDIN.is_standard());
        assert!(FileDescriptor::STDOUT.is_standard());
        assert!(FileDescriptor::STDERR.is_standard());

        let fd = FileDescriptor::new(10);
        assert!(!fd.is_standard());
        assert!(fd.is_valid());

        assert_eq!(FileDescriptor::STDIN.to_string(), "stdin");
        assert_eq!(FileDescriptor::STDOUT.to_string(), "stdout");
    }

    #[test]
    fn test_vm_id() {
        let vm = VmId::new(1);
        assert!(vm.is_valid());
        assert!(!VmId::INVALID.is_valid());
    }

    #[test]
    fn test_signal() {
        assert!(Signal::SIGTERM.is_catchable());
        assert!(!Signal::SIGKILL.is_catchable());
        assert!(!Signal::SIGSTOP.is_catchable());

        assert_eq!(Signal::SIGTERM.name(), "SIGTERM");
        assert_eq!(Signal::SIGKILL.as_i32(), 9);
    }

    #[test]
    fn test_memory_protection() {
        let prot = MemoryProtection::READ_WRITE;
        assert!(prot.is_readable());
        assert!(prot.is_writable());
        assert!(!prot.is_executable());

        let exec = prot.with_exec();
        assert!(exec.is_executable());

        assert_eq!(MemoryProtection::READ.to_string(), "r--");
        assert_eq!(MemoryProtection::READ_WRITE.to_string(), "rw-");
        assert_eq!(MemoryProtection::READ_WRITE_EXEC.to_string(), "rwx");
    }

    #[test]
    fn test_type_safety() {
        // These should be distinct types at compile time
        let pid: ProcessId = ProcessId::new(1);
        let fd: FileDescriptor = FileDescriptor::new(1);
        let vm: VmId = VmId::new(1);

        // All have value 1 but are different types
        assert_eq!(pid.as_u32(), 1);
        assert_eq!(fd.as_u32(), 1);
        assert_eq!(vm.as_u32(), 1);

        // This should NOT compile:
        // let _: ProcessId = fd; // Error!
    }

    #[test]
    fn test_serialization() {
        let pid = ProcessId::new(42);
        let json = serde_json::to_string(&pid).unwrap();
        let restored: ProcessId = serde_json::from_str(&json).unwrap();
        assert_eq!(pid, restored);

        let addr = VirtAddr::new(0x12345678);
        let json = serde_json::to_string(&addr).unwrap();
        let restored: VirtAddr = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, restored);
    }

    // Property-based tests using proptest
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        // VirtAddr properties
        proptest! {
            #[test]
            fn prop_virtaddr_roundtrip(addr: u64) {
                let v = VirtAddr::new(addr);
                prop_assert_eq!(v.as_u64(), addr);
            }

            #[test]
            fn prop_virtaddr_page_aligned_has_zero_offset(page_num: u32) {
                let addr = VirtAddr::new((page_num as u64) * PAGE_SIZE);
                prop_assert!(addr.is_page_aligned());
                prop_assert_eq!(addr.page_offset(), 0);
            }

            #[test]
            fn prop_virtaddr_page_align_down_idempotent(addr: u64) {
                let v = VirtAddr::new(addr);
                let aligned = v.page_align_down();
                prop_assert!(aligned.is_page_aligned());
                prop_assert_eq!(aligned.page_align_down(), aligned);
            }

            #[test]
            fn prop_virtaddr_page_align_up_idempotent(addr: u64) {
                let v = VirtAddr::new(addr);
                let aligned = v.page_align_up();
                // Handle overflow case
                if aligned.as_u64() > 0 {
                    prop_assert!(aligned.is_page_aligned());
                    prop_assert_eq!(aligned.page_align_up(), aligned);
                }
            }

            #[test]
            fn prop_virtaddr_offset_and_sub_offset_inverse(addr: u64, offset in 0u64..1_000_000) {
                let v = VirtAddr::new(addr);
                let after_offset = v.offset(offset);
                let restored = after_offset.sub_offset(offset);
                prop_assert_eq!(v, restored);
            }

            #[test]
            fn prop_virtaddr_page_number_and_offset_reconstruct(addr: u64) {
                let v = VirtAddr::new(addr);
                let page_num = v.page_number().as_u64();
                let offset = v.page_offset();
                let reconstructed = page_num * PAGE_SIZE + offset;
                prop_assert_eq!(v.as_u64(), reconstructed);
            }

            // PhysAddr properties
            #[test]
            fn prop_physaddr_roundtrip(addr: u64) {
                let p = PhysAddr::new(addr);
                prop_assert_eq!(p.as_u64(), addr);
            }

            #[test]
            fn prop_physaddr_page_aligned_has_zero_offset(page_num: u32) {
                let addr = PhysAddr::new((page_num as u64) * PAGE_SIZE);
                prop_assert!(addr.is_page_aligned());
                prop_assert_eq!(addr.page_offset(), 0);
            }

            // ProcessId properties
            #[test]
            fn prop_processid_roundtrip(id: u32) {
                let pid = ProcessId::new(id);
                prop_assert_eq!(pid.as_u32(), id);
            }

            #[test]
            fn prop_processid_valid_range(id in 1u32..=MAX_PID) {
                let pid = ProcessId::new(id);
                prop_assert!(pid.is_valid());
            }

            #[test]
            fn prop_processid_next_increments(id in 0u32..MAX_PID) {
                let pid = ProcessId::new(id);
                if let Some(next) = pid.next() {
                    prop_assert_eq!(next.as_u32(), id + 1);
                }
            }

            // FileDescriptor properties
            #[test]
            fn prop_fd_roundtrip(id: u32) {
                let fd = FileDescriptor::new(id);
                prop_assert_eq!(fd.as_u32(), id);
            }

            #[test]
            fn prop_fd_valid_range(id in 0u32..MAX_FD) {
                let fd = FileDescriptor::new(id);
                prop_assert!(fd.is_valid());
            }

            // VmId properties
            #[test]
            fn prop_vmid_roundtrip(id: u32) {
                let vm = VmId::new(id);
                prop_assert_eq!(vm.as_u32(), id);
            }

            #[test]
            fn prop_vmid_valid_range(id in 1u32..=MAX_VM_ID) {
                let vm = VmId::new(id);
                prop_assert!(vm.is_valid());
            }

            // Signal properties
            #[test]
            fn prop_signal_roundtrip(sig in 1i32..32) {
                let s = Signal::new(sig);
                prop_assert_eq!(s.as_i32(), sig);
            }

            #[test]
            fn prop_signal_valid_range(sig in 1i32..32) {
                let s = Signal::new(sig);
                prop_assert!(s.is_valid());
            }

            #[test]
            fn prop_signal_others_catchable(sig in 1i32..32) {
                let s = Signal::new(sig);
                // Only SIGKILL(9) and SIGSTOP(19) are not catchable
                if sig != 9 && sig != 19 {
                    prop_assert!(s.is_catchable());
                }
            }

            // Pfn properties
            #[test]
            fn prop_pfn_roundtrip(pfn: u64) {
                let p = Pfn::new(pfn);
                prop_assert_eq!(p.as_u64(), pfn);
            }

            #[test]
            fn prop_pfn_to_addr_and_back(pfn in 0u64..1_000_000) {
                let p = Pfn::new(pfn);
                let addr = p.to_phys_addr();
                let restored = addr.page_frame_number();
                prop_assert_eq!(p, restored);
            }

            // Serialization properties
            #[test]
            fn prop_processid_serialize_roundtrip(id: u32) {
                let pid = ProcessId::new(id);
                let json = serde_json::to_string(&pid).unwrap();
                let restored: ProcessId = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(pid, restored);
            }

            #[test]
            fn prop_virtaddr_serialize_roundtrip(addr: u64) {
                let v = VirtAddr::new(addr);
                let json = serde_json::to_string(&v).unwrap();
                let restored: VirtAddr = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(v, restored);
            }

            #[test]
            fn prop_physaddr_serialize_roundtrip(addr: u64) {
                let p = PhysAddr::new(addr);
                let json = serde_json::to_string(&p).unwrap();
                let restored: PhysAddr = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(p, restored);
            }

            #[test]
            fn prop_fd_serialize_roundtrip(id: u32) {
                let fd = FileDescriptor::new(id);
                let json = serde_json::to_string(&fd).unwrap();
                let restored: FileDescriptor = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(fd, restored);
            }

            #[test]
            fn prop_vmid_serialize_roundtrip(id: u32) {
                let vm = VmId::new(id);
                let json = serde_json::to_string(&vm).unwrap();
                let restored: VmId = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(vm, restored);
            }

            #[test]
            fn prop_signal_serialize_roundtrip(sig: i32) {
                let s = Signal::new(sig);
                let json = serde_json::to_string(&s).unwrap();
                let restored: Signal = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(s, restored);
            }
        }

        // Tests that don't need input generation
        #[test]
        fn prop_processid_invalid_zero() {
            let pid = ProcessId::new(0);
            assert!(!pid.is_valid());
        }

        #[test]
        fn prop_vmid_invalid_zero() {
            let vm = VmId::INVALID;
            assert!(!vm.is_valid());
        }

        #[test]
        fn prop_signal_kill_stop_not_catchable() {
            assert!(!Signal::SIGKILL.is_catchable());
            assert!(!Signal::SIGSTOP.is_catchable());
        }

        #[test]
        fn prop_fd_std_streams() {
            assert!(FileDescriptor::STDIN.is_valid());
            assert!(FileDescriptor::STDOUT.is_valid());
            assert!(FileDescriptor::STDERR.is_valid());
            assert_eq!(FileDescriptor::STDIN.as_u32(), 0);
            assert_eq!(FileDescriptor::STDOUT.as_u32(), 1);
            assert_eq!(FileDescriptor::STDERR.as_u32(), 2);
        }
    }

    // Additional coverage tests for untested code paths
    mod coverage_tests {
        use super::*;

        // VirtAddr coverage
        #[test]
        fn test_virtaddr_distance() {
            let addr1 = VirtAddr::new(0x1000);
            let addr2 = VirtAddr::new(0x2000);
            assert_eq!(addr1.distance(&addr2), 0x1000);
            assert_eq!(addr2.distance(&addr1), 0x1000);
            assert_eq!(addr1.distance(&addr1), 0);
        }

        #[test]
        fn test_virtaddr_display() {
            let addr = VirtAddr::new(0x123456789ABCDEF0);
            assert_eq!(format!("{}", addr), "0x123456789abcdef0");
            let zero = VirtAddr::new(0);
            assert_eq!(format!("{}", zero), "0x0000000000000000");
        }

        #[test]
        fn test_virtaddr_from_traits() {
            let addr: VirtAddr = 0x1000u64.into();
            assert_eq!(addr.as_u64(), 0x1000);
            let raw: u64 = addr.into();
            assert_eq!(raw, 0x1000);
        }

        // PhysAddr coverage
        #[test]
        fn test_physaddr_page_operations() {
            let addr = PhysAddr::new(0x1234);
            let aligned_down = addr.page_align_down();
            assert_eq!(aligned_down.as_u64(), 0x1000);
            assert!(aligned_down.is_page_aligned());

            let aligned_up = addr.page_align_up();
            assert_eq!(aligned_up.as_u64(), 0x2000);
            assert!(aligned_up.is_page_aligned());
        }

        #[test]
        fn test_physaddr_offset() {
            let addr = PhysAddr::new(0x1000);
            let offset_addr = addr.offset(0x500);
            assert_eq!(offset_addr.as_u64(), 0x1500);
        }

        #[test]
        fn test_physaddr_display() {
            let addr = PhysAddr::new(0xDEADBEEF);
            assert_eq!(format!("{}", addr), "0x00000000deadbeef");
        }

        #[test]
        fn test_physaddr_from_traits() {
            let addr: PhysAddr = 0x2000u64.into();
            assert_eq!(addr.as_u64(), 0x2000);
            let raw: u64 = addr.into();
            assert_eq!(raw, 0x2000);
        }

        // Pfn coverage
        #[test]
        fn test_pfn_prev() {
            let pfn = Pfn::new(5);
            let prev = pfn.prev();
            assert!(prev.is_some());
            assert_eq!(prev.unwrap().as_u64(), 4);

            let zero_pfn = Pfn::new(0);
            assert!(zero_pfn.prev().is_none());
        }

        #[test]
        fn test_pfn_display() {
            let pfn = Pfn::new(42);
            assert_eq!(format!("{}", pfn), "PFN:42");
        }

        #[test]
        fn test_pfn_from_trait() {
            let pfn: Pfn = 10u64.into();
            assert_eq!(pfn.as_u64(), 10);
        }

        // ProcessId coverage
        #[test]
        fn test_processid_next_at_max() {
            let max_pid = ProcessId::new(MAX_PID);
            assert!(max_pid.next().is_none());
        }

        #[test]
        fn test_processid_display() {
            let pid = ProcessId::new(123);
            assert_eq!(format!("{}", pid), "PID:123");
        }

        #[test]
        fn test_processid_from_traits() {
            let pid: ProcessId = 50u32.into();
            assert_eq!(pid.as_u32(), 50);
            let raw: u32 = pid.into();
            assert_eq!(raw, 50);
        }

        #[test]
        fn test_processid_default() {
            let pid = ProcessId::default();
            assert_eq!(pid, ProcessId::INVALID);
            assert!(!pid.is_valid());
        }

        // FileDescriptor coverage
        #[test]
        fn test_fd_next_at_max() {
            let max_fd = FileDescriptor::new(MAX_FD - 1);
            assert!(max_fd.next().is_none());
        }

        #[test]
        fn test_fd_display_numeric() {
            let fd = FileDescriptor::new(10);
            assert_eq!(format!("{}", fd), "fd:10");
            assert_eq!(format!("{}", FileDescriptor::STDERR), "stderr");
        }

        #[test]
        fn test_fd_from_traits() {
            let fd: FileDescriptor = 100u32.into();
            assert_eq!(fd.as_u32(), 100);
            let raw: u32 = fd.into();
            assert_eq!(raw, 100);
        }

        // VmId coverage
        #[test]
        fn test_vmid_next_at_max() {
            let max_vm = VmId::new(MAX_VM_ID);
            assert!(max_vm.next().is_none());
        }

        #[test]
        fn test_vmid_display() {
            let vm = VmId::new(5);
            assert_eq!(format!("{}", vm), "VM:5");
        }

        #[test]
        fn test_vmid_from_trait() {
            let vm: VmId = 10u32.into();
            assert_eq!(vm.as_u32(), 10);
        }

        #[test]
        fn test_vmid_default() {
            let vm = VmId::default();
            assert_eq!(vm, VmId::INVALID);
            assert!(!vm.is_valid());
        }

        // Signal coverage - test all signal names
        #[test]
        fn test_signal_all_names() {
            assert_eq!(Signal::SIGHUP.name(), "SIGHUP");
            assert_eq!(Signal::SIGINT.name(), "SIGINT");
            assert_eq!(Signal::SIGQUIT.name(), "SIGQUIT");
            assert_eq!(Signal::SIGILL.name(), "SIGILL");
            assert_eq!(Signal::SIGTRAP.name(), "SIGTRAP");
            assert_eq!(Signal::SIGABRT.name(), "SIGABRT");
            assert_eq!(Signal::SIGBUS.name(), "SIGBUS");
            assert_eq!(Signal::SIGFPE.name(), "SIGFPE");
            assert_eq!(Signal::SIGUSR1.name(), "SIGUSR1");
            assert_eq!(Signal::SIGSEGV.name(), "SIGSEGV");
            assert_eq!(Signal::SIGUSR2.name(), "SIGUSR2");
            assert_eq!(Signal::SIGPIPE.name(), "SIGPIPE");
            assert_eq!(Signal::SIGALRM.name(), "SIGALRM");
            assert_eq!(Signal::SIGCHLD.name(), "SIGCHLD");
            assert_eq!(Signal::SIGCONT.name(), "SIGCONT");
            assert_eq!(Signal::SIGTSTP.name(), "SIGTSTP");
        }

        #[test]
        fn test_signal_unknown_name() {
            let unknown = Signal::new(99);
            assert_eq!(unknown.name(), "UNKNOWN");
            assert!(!unknown.is_valid());
        }

        #[test]
        fn test_signal_display() {
            assert_eq!(format!("{}", Signal::SIGTERM), "SIGTERM");
            assert_eq!(format!("{}", Signal::new(99)), "UNKNOWN");
        }

        #[test]
        fn test_signal_from_trait() {
            let sig: Signal = 15i32.into();
            assert_eq!(sig.as_i32(), 15);
            assert_eq!(sig.name(), "SIGTERM");
        }

        // MemoryProtection coverage
        #[test]
        fn test_memprot_builders() {
            let prot = MemoryProtection::NONE.with_read().with_write().with_exec();
            assert_eq!(prot, MemoryProtection::READ_WRITE_EXEC);

            let read_only = MemoryProtection::NONE.with_read();
            assert!(read_only.is_readable());
            assert!(!read_only.is_writable());
            assert!(!read_only.is_executable());
        }

        #[test]
        fn test_memprot_display_all() {
            assert_eq!(format!("{}", MemoryProtection::NONE), "---");
            assert_eq!(format!("{}", MemoryProtection::EXEC), "--x");
            assert_eq!(format!("{}", MemoryProtection::WRITE), "-w-");
            assert_eq!(format!("{}", MemoryProtection::READ_EXEC), "r-x");
        }

        #[test]
        fn test_memprot_default() {
            let prot = MemoryProtection::default();
            assert_eq!(prot, MemoryProtection::NONE);
        }

        #[test]
        fn test_memprot_as_u8() {
            assert_eq!(MemoryProtection::NONE.as_u8(), 0);
            assert_eq!(MemoryProtection::READ.as_u8(), 1);
            assert_eq!(MemoryProtection::WRITE.as_u8(), 2);
            assert_eq!(MemoryProtection::EXEC.as_u8(), 4);
            assert_eq!(MemoryProtection::READ_WRITE_EXEC.as_u8(), 7);
        }

        #[test]
        fn test_memprot_new_masks_bits() {
            // New should mask to only 3 bits
            let prot = MemoryProtection::new(0xFF);
            assert_eq!(prot.as_u8(), 0x07);
        }

        // Edge cases for address arithmetic
        #[test]
        fn test_virtaddr_wrapping_arithmetic() {
            let addr = VirtAddr::new(u64::MAX);
            let offset = addr.offset(1);
            assert_eq!(offset.as_u64(), 0); // Wraps around

            let addr2 = VirtAddr::new(0);
            let sub = addr2.sub_offset(1);
            assert_eq!(sub.as_u64(), u64::MAX); // Wraps around
        }

        #[test]
        fn test_physaddr_page_frame_number() {
            let addr = PhysAddr::new(0x12345);
            let pfn = addr.page_frame_number();
            assert_eq!(pfn.as_u64(), 0x12345 / PAGE_SIZE);
        }
    }
}
