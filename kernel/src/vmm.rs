//! MicroVM Abstraction Layer
//!
//! Educational implementation of virtualization concepts for WOS.
//! Provides a simulated MicroVM environment with guest memory isolation,
//! virtual CPUs, and VirtIO device emulation.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │           Host Kernel (WOS)          │
//! ├─────────────────────────────────────┤
//! │  MicroVM Manager                     │
//! │  ├── VM Creation/Destruction         │
//! │  ├── Guest Memory Management         │
//! │  └── VirtIO Device Coordination      │
//! ├─────────────────────────────────────┤
//! │  MicroVM Instance                    │
//! │  ├── vCPU State                      │
//! │  ├── Guest Physical Memory           │
//! │  └── VirtIO Devices                  │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Educational Purpose
//!
//! This is NOT a production hypervisor. It demonstrates:
//! - VM lifecycle management
//! - Guest memory isolation
//! - Device emulation concepts
//! - APR model execution in isolated environments

#![forbid(unsafe_code)]

use im::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

use wos_shared::primitives::{PhysAddr, VmId};
use wos_shared::AprModel;

/// Maximum guest memory (64 MB for educational purposes)
pub const MAX_GUEST_MEMORY: usize = 64 * 1024 * 1024;

/// Address where kernel/APR model is loaded in guest memory
pub const KERNEL_LOAD_ADDR: u64 = 0x10000;

/// Default guest memory (16 MB)
pub const DEFAULT_GUEST_MEMORY: usize = 16 * 1024 * 1024;

/// Page size for guest memory
pub const GUEST_PAGE_SIZE: usize = 4096;

/// MicroVM error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum VmError {
    /// VM not found
    #[error("VM not found: {0}")]
    NotFound(VmId),

    /// VM already exists
    #[error("VM already exists: {0}")]
    AlreadyExists(VmId),

    /// Invalid VM state for operation
    #[error("Invalid VM state: expected {expected:?}, got {actual:?}")]
    InvalidState {
        /// Expected VM state
        expected: VmState,
        /// Actual VM state encountered
        actual: VmState,
    },

    /// Memory allocation failed
    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),

    /// Guest memory access out of bounds
    #[error("Guest memory access out of bounds: addr={addr}, size={size}")]
    MemoryOutOfBounds {
        /// Guest address that was accessed
        addr: u64,
        /// Size of the access attempt
        size: usize,
    },

    /// VirtIO device error
    #[error("VirtIO device error: {0}")]
    VirtioError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// APR kernel loading error
    #[error("APR kernel load error: {0}")]
    AprKernelLoadError(String),
}

/// VM state enumeration
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VmState {
    /// VM created but not started
    #[default]
    Created,
    /// VM is starting up
    Starting,
    /// VM is running
    Running,
    /// VM is paused
    Paused,
    /// VM is stopping
    Stopping,
    /// VM has stopped
    Stopped,
    /// VM encountered a fatal error
    Failed,
}

/// VM exit reasons
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VmExitReason {
    /// VM halted normally
    Halt,
    /// VM requested I/O
    IoRequest(IoRequest),
    /// VM triggered an interrupt
    Interrupt(u8),
    /// VM encountered an exception
    Exception(ExceptionInfo),
    /// VM needs to handle a timer
    Timer,
    /// Guest requested shutdown
    Shutdown,
    /// Guest requested reboot
    Reboot,
    /// Continue execution
    Continue,
}

/// I/O request from guest
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IoRequest {
    /// Port number (for port I/O) or address (for MMIO)
    pub address: u64,
    /// Data size in bytes
    pub size: usize,
    /// Whether this is a write operation
    pub is_write: bool,
    /// Data for writes
    pub data: Option<Vec<u8>>,
}

/// Exception information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExceptionInfo {
    /// Exception vector number
    pub vector: u8,
    /// Error code if applicable
    pub error_code: Option<u32>,
    /// Faulting address if applicable
    pub fault_address: Option<u64>,
}

/// Virtual CPU state
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VcpuState {
    /// vCPU ID
    pub id: u8,
    /// Instruction pointer
    pub rip: u64,
    /// Stack pointer
    pub rsp: u64,
    /// General purpose registers (simplified)
    pub regs: [u64; 8],
    /// Flags register
    pub rflags: u64,
    /// Whether vCPU is running
    pub running: bool,
    /// Pending interrupt
    pub pending_interrupt: Option<u8>,
}

impl VcpuState {
    /// Create a new vCPU state
    pub fn new(id: u8) -> Self {
        Self {
            id,
            rip: 0,
            rsp: 0,
            regs: [0; 8],
            rflags: 0x2, // Reserved bit set
            running: false,
            pending_interrupt: None,
        }
    }

    /// Set instruction pointer
    pub fn set_rip(&mut self, rip: u64) {
        self.rip = rip;
    }

    /// Set stack pointer
    pub fn set_rsp(&mut self, rsp: u64) {
        self.rsp = rsp;
    }

    /// Inject an interrupt
    pub fn inject_interrupt(&mut self, vector: u8) {
        self.pending_interrupt = Some(vector);
    }

    /// Check if there's a pending exit
    pub fn pending_exit(&self) -> Option<VmExitReason> {
        self.pending_interrupt.map(VmExitReason::Interrupt)
    }
}

/// VM configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VmConfig {
    /// Guest memory size in bytes
    pub memory_bytes: usize,
    /// Number of virtual CPUs
    pub vcpu_count: u8,
    /// Kernel/APR model path
    pub kernel_path: Option<String>,
    /// Kernel command line
    pub cmdline: String,
    /// VirtIO devices to attach
    pub devices: Vec<VirtioDeviceConfig>,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            memory_bytes: DEFAULT_GUEST_MEMORY,
            vcpu_count: 1,
            kernel_path: None,
            cmdline: String::new(),
            devices: vec![VirtioDeviceConfig::Console],
        }
    }
}

impl VmConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<(), VmError> {
        if self.memory_bytes == 0 {
            return Err(VmError::ConfigError("Memory size cannot be zero".into()));
        }
        if self.memory_bytes > MAX_GUEST_MEMORY {
            return Err(VmError::ConfigError(format!(
                "Memory size {} exceeds maximum {}",
                self.memory_bytes, MAX_GUEST_MEMORY
            )));
        }
        if self.vcpu_count == 0 {
            return Err(VmError::ConfigError("vCPU count cannot be zero".into()));
        }
        Ok(())
    }
}

/// VirtIO device configuration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VirtioDeviceConfig {
    /// Console device
    Console,
    /// Block device with backing file path
    Block {
        /// Path to the backing file
        path: String,
        /// Whether the device is read-only
        readonly: bool,
    },
    /// Network device
    Net {
        /// MAC address for the network interface
        mac: [u8; 6],
    },
    /// Vsock device
    Vsock {
        /// Context ID for the vsock device
        cid: u32,
    },
}

/// Guest physical memory
#[derive(Clone, Debug)]
pub struct GuestMemory {
    /// Memory pages (sparse representation using im::HashMap)
    pages: HashMap<u64, GuestPage>,
    /// Total allocated bytes
    total_bytes: usize,
    /// Maximum bytes
    max_bytes: usize,
}

/// A single guest memory page
#[derive(Clone, Debug)]
pub struct GuestPage {
    /// Page data
    data: Vec<u8>,
    /// Whether page is dirty
    dirty: bool,
}

impl Default for GuestPage {
    fn default() -> Self {
        Self::zero()
    }
}

impl GuestPage {
    /// Create a zero-filled page
    pub fn zero() -> Self {
        Self {
            data: vec![0u8; GUEST_PAGE_SIZE],
            dirty: false,
        }
    }

    /// Mark page as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl GuestMemory {
    /// Create new guest memory
    pub fn new(max_bytes: usize) -> Result<Self, VmError> {
        if max_bytes == 0 || max_bytes > MAX_GUEST_MEMORY {
            return Err(VmError::ConfigError(format!(
                "Invalid memory size: {}",
                max_bytes
            )));
        }

        Ok(Self {
            pages: HashMap::new(),
            total_bytes: 0,
            max_bytes,
        })
    }

    /// Read from guest memory
    pub fn read(&self, addr: PhysAddr, buf: &mut [u8]) -> Result<(), VmError> {
        let addr = addr.as_u64();
        let end = addr
            .checked_add(buf.len() as u64)
            .ok_or(VmError::MemoryOutOfBounds {
                addr,
                size: buf.len(),
            })?;

        if end as usize > self.max_bytes {
            return Err(VmError::MemoryOutOfBounds {
                addr,
                size: buf.len(),
            });
        }

        // Read page by page
        let mut offset = 0;
        while offset < buf.len() {
            let page_addr = (addr + offset as u64) & !(GUEST_PAGE_SIZE as u64 - 1);
            let page_offset = ((addr + offset as u64) & (GUEST_PAGE_SIZE as u64 - 1)) as usize;
            let page_remaining = GUEST_PAGE_SIZE - page_offset;
            let to_read = (buf.len() - offset).min(page_remaining);

            if let Some(page) = self.pages.get(&page_addr) {
                buf[offset..offset + to_read]
                    .copy_from_slice(&page.data[page_offset..page_offset + to_read]);
            } else {
                // Unallocated pages read as zero
                buf[offset..offset + to_read].fill(0);
            }

            offset += to_read;
        }

        Ok(())
    }

    /// Write to guest memory
    pub fn write(&mut self, addr: PhysAddr, data: &[u8]) -> Result<(), VmError> {
        let addr = addr.as_u64();
        let end = addr
            .checked_add(data.len() as u64)
            .ok_or(VmError::MemoryOutOfBounds {
                addr,
                size: data.len(),
            })?;

        if end as usize > self.max_bytes {
            return Err(VmError::MemoryOutOfBounds {
                addr,
                size: data.len(),
            });
        }

        // Write page by page
        let mut offset = 0;
        while offset < data.len() {
            let page_addr = (addr + offset as u64) & !(GUEST_PAGE_SIZE as u64 - 1);
            let page_offset = ((addr + offset as u64) & (GUEST_PAGE_SIZE as u64 - 1)) as usize;
            let page_remaining = GUEST_PAGE_SIZE - page_offset;
            let to_write = (data.len() - offset).min(page_remaining);

            let page = self.pages.entry(page_addr).or_insert_with(|| {
                self.total_bytes += GUEST_PAGE_SIZE;
                GuestPage::zero()
            });

            page.data[page_offset..page_offset + to_write]
                .copy_from_slice(&data[offset..offset + to_write]);
            page.mark_dirty();

            offset += to_write;
        }

        Ok(())
    }

    /// Get total memory used
    pub fn used_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Get maximum memory
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    /// Check if memory is within bounds
    pub fn within_bounds(&self) -> bool {
        self.total_bytes <= self.max_bytes
    }

    /// Clear all memory
    pub fn clear(&mut self) {
        self.pages.clear();
        self.total_bytes = 0;
    }
}

/// VirtIO console device
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VirtioConsole {
    /// Receive buffer (guest reads from here)
    rx_buffer: VecDeque<u8>,
    /// Transmit buffer (guest writes here)
    tx_buffer: VecDeque<u8>,
    /// Maximum buffer size
    max_buffer: usize,
}

impl VirtioConsole {
    /// Create a new console device
    pub fn new(max_buffer: usize) -> Self {
        Self {
            rx_buffer: VecDeque::new(),
            tx_buffer: VecDeque::new(),
            max_buffer,
        }
    }

    /// Write data from host to guest (guest will receive)
    pub fn host_write(&mut self, data: &[u8]) -> usize {
        let available = self.max_buffer - self.rx_buffer.len();
        let to_write = data.len().min(available);
        self.rx_buffer.extend(&data[..to_write]);
        to_write
    }

    /// Read data from guest (guest transmitted)
    pub fn host_read(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.tx_buffer.len());
        for byte in buf.iter_mut().take(to_read) {
            *byte = self.tx_buffer.pop_front().unwrap_or(0);
        }
        to_read
    }

    /// Guest writes data (transmit)
    pub fn guest_write(&mut self, data: &[u8]) -> usize {
        let available = self.max_buffer - self.tx_buffer.len();
        let to_write = data.len().min(available);
        self.tx_buffer.extend(&data[..to_write]);
        to_write
    }

    /// Guest reads data (receive)
    pub fn guest_read(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.rx_buffer.len());
        for byte in buf.iter_mut().take(to_read) {
            *byte = self.rx_buffer.pop_front().unwrap_or(0);
        }
        to_read
    }

    /// Check if there's data available for guest to read
    pub fn rx_available(&self) -> usize {
        self.rx_buffer.len()
    }

    /// Check if there's data available for host to read
    pub fn tx_available(&self) -> usize {
        self.tx_buffer.len()
    }
}

/// VirtIO block device
///
/// Simulated block device with sector-based read/write operations.
/// Provides educational implementation of storage device emulation.
#[derive(Clone, Debug)]
pub struct VirtioBlock {
    /// Backing store data
    data: Vec<u8>,
    /// Sector size in bytes (typically 512)
    sector_size: usize,
    /// Whether the device is read-only
    readonly: bool,
    /// Device path (for display/identification)
    path: String,
    /// Number of read operations
    read_ops: u64,
    /// Number of write operations
    write_ops: u64,
}

/// Block device sector size
pub const BLOCK_SECTOR_SIZE: usize = 512;

/// Maximum block device size (16 MB for educational purposes)
pub const MAX_BLOCK_SIZE: usize = 16 * 1024 * 1024;

impl VirtioBlock {
    /// Create a new block device with specified size
    pub fn new(path: &str, size_bytes: usize, readonly: bool) -> Result<Self, VmError> {
        if size_bytes == 0 {
            return Err(VmError::ConfigError(
                "Block device size cannot be zero".into(),
            ));
        }
        if size_bytes > MAX_BLOCK_SIZE {
            return Err(VmError::ConfigError(format!(
                "Block device size {} exceeds maximum {}",
                size_bytes, MAX_BLOCK_SIZE
            )));
        }
        // Round up to sector boundary
        let aligned_size = size_bytes.div_ceil(BLOCK_SECTOR_SIZE) * BLOCK_SECTOR_SIZE;

        Ok(Self {
            data: vec![0u8; aligned_size],
            sector_size: BLOCK_SECTOR_SIZE,
            readonly,
            path: path.to_string(),
            read_ops: 0,
            write_ops: 0,
        })
    }

    /// Create from existing data (useful for loading disk images)
    pub fn from_data(path: &str, data: Vec<u8>, readonly: bool) -> Result<Self, VmError> {
        if data.is_empty() {
            return Err(VmError::ConfigError(
                "Block device data cannot be empty".into(),
            ));
        }
        if data.len() > MAX_BLOCK_SIZE {
            return Err(VmError::ConfigError(format!(
                "Block device size {} exceeds maximum {}",
                data.len(),
                MAX_BLOCK_SIZE
            )));
        }

        Ok(Self {
            data,
            sector_size: BLOCK_SECTOR_SIZE,
            readonly,
            path: path.to_string(),
            read_ops: 0,
            write_ops: 0,
        })
    }

    /// Get device capacity in bytes
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    /// Get number of sectors
    pub fn sector_count(&self) -> usize {
        self.data.len() / self.sector_size
    }

    /// Get sector size
    pub fn sector_size(&self) -> usize {
        self.sector_size
    }

    /// Check if device is read-only
    pub fn is_readonly(&self) -> bool {
        self.readonly
    }

    /// Get device path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Read a sector
    pub fn read_sector(&mut self, sector: u64, buf: &mut [u8]) -> Result<usize, VmError> {
        let offset = (sector as usize) * self.sector_size;
        let to_read = buf.len().min(self.sector_size);

        if offset + to_read > self.data.len() {
            return Err(VmError::VirtioError(format!(
                "Block read out of bounds: sector {} (offset {})",
                sector, offset
            )));
        }

        buf[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
        self.read_ops += 1;
        Ok(to_read)
    }

    /// Write a sector
    pub fn write_sector(&mut self, sector: u64, data: &[u8]) -> Result<usize, VmError> {
        if self.readonly {
            return Err(VmError::VirtioError("Block device is read-only".into()));
        }

        let offset = (sector as usize) * self.sector_size;
        let to_write = data.len().min(self.sector_size);

        if offset + to_write > self.data.len() {
            return Err(VmError::VirtioError(format!(
                "Block write out of bounds: sector {} (offset {})",
                sector, offset
            )));
        }

        self.data[offset..offset + to_write].copy_from_slice(&data[..to_write]);
        self.write_ops += 1;
        Ok(to_write)
    }

    /// Read multiple sectors
    pub fn read_sectors(
        &mut self,
        start_sector: u64,
        count: usize,
        buf: &mut [u8],
    ) -> Result<usize, VmError> {
        let total_bytes = count * self.sector_size;
        if buf.len() < total_bytes {
            return Err(VmError::VirtioError("Buffer too small for read".into()));
        }

        let mut total_read = 0;
        for i in 0..count {
            let sector = start_sector + i as u64;
            let offset = i * self.sector_size;
            total_read += self.read_sector(sector, &mut buf[offset..offset + self.sector_size])?;
        }
        Ok(total_read)
    }

    /// Write multiple sectors
    pub fn write_sectors(
        &mut self,
        start_sector: u64,
        count: usize,
        data: &[u8],
    ) -> Result<usize, VmError> {
        if self.readonly {
            return Err(VmError::VirtioError("Block device is read-only".into()));
        }

        let total_bytes = count * self.sector_size;
        if data.len() < total_bytes {
            return Err(VmError::VirtioError("Data too small for write".into()));
        }

        let mut total_written = 0;
        for i in 0..count {
            let sector = start_sector + i as u64;
            let offset = i * self.sector_size;
            total_written += self.write_sector(sector, &data[offset..offset + self.sector_size])?;
        }
        Ok(total_written)
    }

    /// Get read operation count
    pub fn read_ops(&self) -> u64 {
        self.read_ops
    }

    /// Get write operation count
    pub fn write_ops(&self) -> u64 {
        self.write_ops
    }

    /// Get block device status
    pub fn status(&self) -> BlockDeviceStatus {
        BlockDeviceStatus {
            path: self.path.clone(),
            capacity: self.capacity(),
            sector_count: self.sector_count(),
            sector_size: self.sector_size,
            readonly: self.readonly,
            read_ops: self.read_ops,
            write_ops: self.write_ops,
        }
    }
}

/// Block device status information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDeviceStatus {
    /// Device path
    pub path: String,
    /// Capacity in bytes
    pub capacity: usize,
    /// Number of sectors
    pub sector_count: usize,
    /// Sector size in bytes
    pub sector_size: usize,
    /// Whether device is read-only
    pub readonly: bool,
    /// Number of read operations
    pub read_ops: u64,
    /// Number of write operations
    pub write_ops: u64,
}

/// MicroVM instance
#[derive(Clone, Debug)]
pub struct MicroVm {
    /// VM identifier
    pub id: VmId,
    /// Configuration
    pub config: VmConfig,
    /// Current state
    pub state: VmState,
    /// Guest memory
    pub memory: GuestMemory,
    /// vCPU states
    pub vcpus: Vec<VcpuState>,
    /// Console device
    pub console: VirtioConsole,
    /// Exit code if stopped
    pub exit_code: Option<i32>,
    /// Total executed instructions (for educational tracking)
    pub instruction_count: u64,
}

impl MicroVm {
    /// Create a new MicroVM
    pub fn create(id: VmId, config: VmConfig) -> Result<Self, VmError> {
        config.validate()?;

        let memory = GuestMemory::new(config.memory_bytes)?;
        let vcpus = (0..config.vcpu_count).map(VcpuState::new).collect();

        Ok(Self {
            id,
            config,
            state: VmState::Created,
            memory,
            vcpus,
            console: VirtioConsole::new(4096),
            exit_code: None,
            instruction_count: 0,
        })
    }

    /// Start the VM
    pub fn start(&mut self) -> Result<(), VmError> {
        if self.state != VmState::Created && self.state != VmState::Stopped {
            return Err(VmError::InvalidState {
                expected: VmState::Created,
                actual: self.state,
            });
        }

        self.state = VmState::Starting;

        // Mark first vCPU as running
        if let Some(vcpu) = self.vcpus.first_mut() {
            vcpu.running = true;
        }

        self.state = VmState::Running;
        Ok(())
    }

    /// Pause the VM
    pub fn pause(&mut self) -> Result<(), VmError> {
        if self.state != VmState::Running {
            return Err(VmError::InvalidState {
                expected: VmState::Running,
                actual: self.state,
            });
        }

        self.state = VmState::Paused;
        for vcpu in &mut self.vcpus {
            vcpu.running = false;
        }
        Ok(())
    }

    /// Resume the VM
    pub fn resume(&mut self) -> Result<(), VmError> {
        if self.state != VmState::Paused {
            return Err(VmError::InvalidState {
                expected: VmState::Paused,
                actual: self.state,
            });
        }

        self.state = VmState::Running;
        if let Some(vcpu) = self.vcpus.first_mut() {
            vcpu.running = true;
        }
        Ok(())
    }

    /// Stop the VM
    pub fn stop(&mut self, exit_code: i32) -> Result<(), VmError> {
        self.state = VmState::Stopping;

        for vcpu in &mut self.vcpus {
            vcpu.running = false;
        }

        self.exit_code = Some(exit_code);
        self.state = VmState::Stopped;
        Ok(())
    }

    /// Reset the VM
    pub fn reset(&mut self) -> Result<(), VmError> {
        self.memory.clear();
        self.vcpus = (0..self.config.vcpu_count).map(VcpuState::new).collect();
        self.console = VirtioConsole::new(4096);
        self.exit_code = None;
        self.instruction_count = 0;
        self.state = VmState::Created;
        Ok(())
    }

    /// Step execution (educational - one "instruction" at a time)
    pub fn step(&mut self) -> Result<VmExitReason, VmError> {
        if self.state != VmState::Running {
            return Err(VmError::InvalidState {
                expected: VmState::Running,
                actual: self.state,
            });
        }

        // Get active vCPU
        let vcpu = self
            .vcpus
            .first_mut()
            .ok_or_else(|| VmError::ConfigError("No vCPUs configured".into()))?;

        // Check for pending exits
        if let Some(exit) = vcpu.pending_exit() {
            vcpu.pending_interrupt = None;
            return Ok(exit);
        }

        // Simulate instruction execution (educational)
        vcpu.rip = vcpu.rip.wrapping_add(1);
        self.instruction_count += 1;

        Ok(VmExitReason::Continue)
    }

    /// Get VM status for display
    pub fn status(&self) -> VmStatus {
        VmStatus {
            id: self.id,
            state: self.state,
            memory_used: self.memory.used_bytes(),
            memory_max: self.memory.max_bytes(),
            vcpu_count: self.vcpus.len(),
            instruction_count: self.instruction_count,
            exit_code: self.exit_code,
        }
    }

    /// Load APR model as guest kernel
    ///
    /// Serializes the APR model and loads it into guest memory at KERNEL_LOAD_ADDR.
    /// This allows deterministic replay of kernel state in an isolated VM environment.
    ///
    /// # Arguments
    /// * `apr` - The APR model to load as kernel
    ///
    /// # Errors
    /// Returns `VmError::AprKernelLoadError` if serialization fails.
    /// Returns `VmError::MemoryOutOfBounds` if the model is too large for guest memory.
    pub fn load_apr_kernel(&mut self, apr: &AprModel) -> Result<(), VmError> {
        // Serialize APR model to bytes
        let kernel_data = apr
            .to_bytes()
            .map_err(|e| VmError::AprKernelLoadError(e.to_string()))?;

        // Load into guest memory at KERNEL_LOAD_ADDR
        let load_addr = PhysAddr::new(KERNEL_LOAD_ADDR);
        self.memory.write(load_addr, &kernel_data)?;

        // Set vCPU instruction pointer to kernel load address
        if let Some(vcpu) = self.vcpus.first_mut() {
            vcpu.set_rip(KERNEL_LOAD_ADDR);
        }

        Ok(())
    }

    /// Check if an APR kernel is loaded
    pub fn has_kernel_loaded(&self) -> bool {
        // Check if there's data at KERNEL_LOAD_ADDR
        let mut buf = [0u8; 8];
        if self
            .memory
            .read(PhysAddr::new(KERNEL_LOAD_ADDR), &mut buf)
            .is_ok()
        {
            // APR models always start with '{'
            buf[0] == b'{'
        } else {
            false
        }
    }

    /// Get the loaded kernel size (for display/debugging)
    pub fn kernel_size(&self) -> Option<usize> {
        if !self.has_kernel_loaded() {
            return None;
        }
        // Approximate size based on memory used
        Some(self.memory.used_bytes())
    }
}

/// VM status information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VmStatus {
    /// VM ID
    pub id: VmId,
    /// Current state
    pub state: VmState,
    /// Memory used
    pub memory_used: usize,
    /// Maximum memory
    pub memory_max: usize,
    /// vCPU count
    pub vcpu_count: usize,
    /// Instruction count
    pub instruction_count: u64,
    /// Exit code if stopped
    pub exit_code: Option<i32>,
}

/// MicroVM Manager
#[derive(Clone, Debug, Default)]
pub struct VmManager {
    /// Active VMs
    vms: HashMap<VmId, MicroVm>,
    /// Next VM ID
    next_id: u32,
}

impl VmManager {
    /// Create a new VM manager
    pub fn new() -> Self {
        Self {
            vms: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new VM
    pub fn create_vm(&mut self, config: VmConfig) -> Result<VmId, VmError> {
        let id = VmId::new(self.next_id);
        self.next_id += 1;

        let vm = MicroVm::create(id, config)?;
        self.vms.insert(id, vm);

        Ok(id)
    }

    /// Get a VM by ID
    pub fn get(&self, id: VmId) -> Option<&MicroVm> {
        self.vms.get(&id)
    }

    /// Get a mutable VM by ID
    pub fn get_mut(&mut self, id: VmId) -> Option<&mut MicroVm> {
        self.vms.get_mut(&id)
    }

    /// Start a VM
    pub fn start_vm(&mut self, id: VmId) -> Result<(), VmError> {
        let vm = self.vms.get_mut(&id).ok_or(VmError::NotFound(id))?;
        vm.start()
    }

    /// Stop a VM
    pub fn stop_vm(&mut self, id: VmId, exit_code: i32) -> Result<(), VmError> {
        let vm = self.vms.get_mut(&id).ok_or(VmError::NotFound(id))?;
        vm.stop(exit_code)
    }

    /// Destroy a VM
    pub fn destroy_vm(&mut self, id: VmId) -> Result<(), VmError> {
        if self.vms.remove(&id).is_some() {
            Ok(())
        } else {
            Err(VmError::NotFound(id))
        }
    }

    /// List all VMs
    pub fn list(&self) -> Vec<VmStatus> {
        self.vms.values().map(|vm| vm.status()).collect()
    }

    /// Get VM count
    pub fn count(&self) -> usize {
        self.vms.len()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let config = VmConfig::default();
        let vm = MicroVm::create(VmId::new(1), config).unwrap();
        assert_eq!(vm.state, VmState::Created);
        assert_eq!(vm.vcpus.len(), 1);
    }

    #[test]
    fn test_vm_lifecycle() {
        let config = VmConfig::default();
        let mut vm = MicroVm::create(VmId::new(1), config).unwrap();

        // Start
        vm.start().unwrap();
        assert_eq!(vm.state, VmState::Running);

        // Pause
        vm.pause().unwrap();
        assert_eq!(vm.state, VmState::Paused);

        // Resume
        vm.resume().unwrap();
        assert_eq!(vm.state, VmState::Running);

        // Stop
        vm.stop(0).unwrap();
        assert_eq!(vm.state, VmState::Stopped);
        assert_eq!(vm.exit_code, Some(0));
    }

    #[test]
    fn test_vm_invalid_state_transition() {
        let config = VmConfig::default();
        let mut vm = MicroVm::create(VmId::new(1), config).unwrap();

        // Cannot pause before starting
        let result = vm.pause();
        assert!(matches!(result, Err(VmError::InvalidState { .. })));
    }

    #[test]
    fn test_guest_memory() {
        let mut memory = GuestMemory::new(1024 * 1024).unwrap();

        // Write
        let data = b"Hello, VM!";
        memory.write(PhysAddr::new(0x1000), data).unwrap();

        // Read
        let mut buf = vec![0u8; data.len()];
        memory.read(PhysAddr::new(0x1000), &mut buf).unwrap();
        assert_eq!(&buf, data);
    }

    #[test]
    fn test_guest_memory_out_of_bounds() {
        let memory = GuestMemory::new(4096).unwrap();

        let mut buf = vec![0u8; 10];
        let result = memory.read(PhysAddr::new(4090), &mut buf);
        assert!(matches!(result, Err(VmError::MemoryOutOfBounds { .. })));
    }

    #[test]
    fn test_virtio_console() {
        let mut console = VirtioConsole::new(1024);

        // Host writes, guest reads
        console.host_write(b"Hello from host");
        let mut buf = vec![0u8; 20];
        let n = console.guest_read(&mut buf);
        assert_eq!(&buf[..n], b"Hello from host");

        // Guest writes, host reads
        console.guest_write(b"Hello from guest");
        let n = console.host_read(&mut buf);
        assert_eq!(&buf[..n], b"Hello from guest");
    }

    #[test]
    fn test_vm_manager() {
        let mut manager = VmManager::new();

        let id1 = manager.create_vm(VmConfig::default()).unwrap();
        let id2 = manager.create_vm(VmConfig::default()).unwrap();

        assert_eq!(manager.count(), 2);
        assert!(manager.get(id1).is_some());
        assert!(manager.get(id2).is_some());

        manager.start_vm(id1).unwrap();
        assert_eq!(manager.get(id1).unwrap().state, VmState::Running);

        manager.destroy_vm(id1).unwrap();
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_vm_step() {
        let config = VmConfig::default();
        let mut vm = MicroVm::create(VmId::new(1), config).unwrap();
        vm.start().unwrap();

        let exit = vm.step().unwrap();
        assert_eq!(exit, VmExitReason::Continue);
        assert_eq!(vm.instruction_count, 1);

        // Step more
        for _ in 0..99 {
            vm.step().unwrap();
        }
        assert_eq!(vm.instruction_count, 100);
    }

    #[test]
    fn test_config_validation() {
        let config = VmConfig {
            memory_bytes: 0,
            ..Default::default()
        };
        let result = MicroVm::create(VmId::new(1), config);
        assert!(matches!(result, Err(VmError::ConfigError(_))));

        let config = VmConfig {
            memory_bytes: MAX_GUEST_MEMORY + 1,
            ..Default::default()
        };
        let result = MicroVm::create(VmId::new(1), config);
        assert!(matches!(result, Err(VmError::ConfigError(_))));
    }

    #[test]
    fn test_load_apr_kernel() {
        use wos_shared::AprModel;

        let config = VmConfig::default();
        let mut vm = MicroVm::create(VmId::new(1), config).unwrap();

        // Create a simple APR model
        let apr = AprModel::new(42);

        // Load kernel should succeed
        vm.load_apr_kernel(&apr).unwrap();

        // Verify kernel is loaded
        assert!(vm.has_kernel_loaded());

        // Verify vCPU rip is set to kernel load address
        assert_eq!(vm.vcpus[0].rip, KERNEL_LOAD_ADDR);
    }

    #[test]
    fn test_apr_kernel_roundtrip() {
        use wos_shared::AprModel;

        let config = VmConfig::default();
        let mut vm = MicroVm::create(VmId::new(1), config).unwrap();

        // Create APR model with data
        let apr = AprModel::with_initial_state(123, serde_json::json!({"test": "data"}));

        // Load kernel
        vm.load_apr_kernel(&apr).unwrap();

        // Read back from memory and verify roundtrip
        let kernel_bytes = apr.to_bytes().unwrap();
        let mut buf = vec![0u8; kernel_bytes.len()];
        vm.memory
            .read(PhysAddr::new(KERNEL_LOAD_ADDR), &mut buf)
            .unwrap();

        let restored = AprModel::from_bytes(&buf).unwrap();
        assert_eq!(restored.seed, apr.seed);
    }

    #[test]
    fn test_kernel_not_loaded_initially() {
        let config = VmConfig::default();
        let vm = MicroVm::create(VmId::new(1), config).unwrap();

        assert!(!vm.has_kernel_loaded());
        assert!(vm.kernel_size().is_none());
    }

    #[test]
    fn test_kernel_load_addr_constant() {
        // Verify kernel load address is reasonable (compile-time checks)
        const _: () = assert!(KERNEL_LOAD_ADDR > 0);
        const _: () = assert!(KERNEL_LOAD_ADDR < DEFAULT_GUEST_MEMORY as u64);
        // Runtime sanity check that constants are accessible
        assert_ne!(KERNEL_LOAD_ADDR, DEFAULT_GUEST_MEMORY as u64);
    }

    // ========================================================================
    // VirtIO Block device tests
    // ========================================================================

    #[test]
    fn test_block_device_creation() {
        let block = VirtioBlock::new("/dev/vda", 4096, false).unwrap();
        assert_eq!(block.capacity(), 4096);
        assert_eq!(block.sector_count(), 8); // 4096 / 512
        assert!(!block.is_readonly());
        assert_eq!(block.path(), "/dev/vda");
    }

    #[test]
    fn test_block_device_read_write() {
        let mut block = VirtioBlock::new("/dev/vda", 4096, false).unwrap();

        // Write data
        let data = b"Hello, Block Device!";
        let mut sector_data = [0u8; BLOCK_SECTOR_SIZE];
        sector_data[..data.len()].copy_from_slice(data);

        block.write_sector(0, &sector_data).unwrap();
        assert_eq!(block.write_ops(), 1);

        // Read data
        let mut buf = [0u8; BLOCK_SECTOR_SIZE];
        block.read_sector(0, &mut buf).unwrap();
        assert_eq!(block.read_ops(), 1);
        assert_eq!(&buf[..data.len()], data);
    }

    #[test]
    fn test_block_device_readonly() {
        let mut block = VirtioBlock::new("/dev/vda", 4096, true).unwrap();
        assert!(block.is_readonly());

        let data = [0u8; BLOCK_SECTOR_SIZE];
        let result = block.write_sector(0, &data);
        assert!(matches!(result, Err(VmError::VirtioError(_))));
    }

    #[test]
    fn test_block_device_out_of_bounds() {
        let mut block = VirtioBlock::new("/dev/vda", 4096, false).unwrap();

        // Sector 8 is out of bounds (only 0-7 valid)
        let mut buf = [0u8; BLOCK_SECTOR_SIZE];
        let result = block.read_sector(8, &mut buf);
        assert!(matches!(result, Err(VmError::VirtioError(_))));
    }

    #[test]
    fn test_block_device_multi_sector() {
        let mut block = VirtioBlock::new("/dev/vda", 4096, false).unwrap();

        // Write 2 sectors
        let data = vec![0xAB; BLOCK_SECTOR_SIZE * 2];
        block.write_sectors(0, 2, &data).unwrap();
        assert_eq!(block.write_ops(), 2);

        // Read 2 sectors
        let mut buf = vec![0u8; BLOCK_SECTOR_SIZE * 2];
        block.read_sectors(0, 2, &mut buf).unwrap();
        assert_eq!(block.read_ops(), 2);
        assert_eq!(buf, data);
    }

    #[test]
    fn test_block_device_from_data() {
        let data = vec![0xCD; 1024];
        let mut block = VirtioBlock::from_data("/dev/vdb", data.clone(), false).unwrap();

        let mut buf = [0u8; BLOCK_SECTOR_SIZE];
        block.read_sector(0, &mut buf).unwrap();
        assert_eq!(&buf[..], &data[..BLOCK_SECTOR_SIZE]);
    }

    #[test]
    fn test_block_device_status() {
        let mut block = VirtioBlock::new("/dev/vda", 4096, false).unwrap();
        block.read_sector(0, &mut [0u8; 512]).unwrap();
        block.write_sector(1, &[0u8; 512]).unwrap();

        let status = block.status();
        assert_eq!(status.path, "/dev/vda");
        assert_eq!(status.capacity, 4096);
        assert_eq!(status.sector_count, 8);
        assert_eq!(status.read_ops, 1);
        assert_eq!(status.write_ops, 1);
    }

    #[test]
    fn test_block_device_zero_size_error() {
        let result = VirtioBlock::new("/dev/vda", 0, false);
        assert!(matches!(result, Err(VmError::ConfigError(_))));
    }

    #[test]
    fn test_block_device_max_size_error() {
        let result = VirtioBlock::new("/dev/vda", MAX_BLOCK_SIZE + 1, false);
        assert!(matches!(result, Err(VmError::ConfigError(_))));
    }

    // ========================================================================
    // Property-based tests
    // ========================================================================
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_console_host_write_guest_read(
                data in proptest::collection::vec(any::<u8>(), 1..100)
            ) {
                let mut console = VirtioConsole::new(1024);
                let written = console.host_write(&data);
                prop_assert_eq!(written, data.len());

                let mut buf = vec![0u8; data.len()];
                let read = console.guest_read(&mut buf);
                prop_assert_eq!(read, data.len());
                prop_assert_eq!(data, buf);
            }

            #[test]
            fn prop_console_guest_write_host_read(
                data in proptest::collection::vec(any::<u8>(), 1..100)
            ) {
                let mut console = VirtioConsole::new(1024);
                let written = console.guest_write(&data);
                prop_assert_eq!(written, data.len());

                let mut buf = vec![0u8; data.len()];
                let read = console.host_read(&mut buf);
                prop_assert_eq!(read, data.len());
                prop_assert_eq!(data, buf);
            }

            #[test]
            fn prop_console_buffer_limit_respected(
                buffer_size in 64usize..256,
                data_size in 256usize..512
            ) {
                prop_assume!(data_size > buffer_size);
                let mut console = VirtioConsole::new(buffer_size);
                let data = vec![0xAB; data_size];
                let written = console.host_write(&data);
                prop_assert!(written <= buffer_size);
            }

            #[test]
            fn prop_vcpu_rip_preserved(id: u8, rip: u64) {
                let mut vcpu = VcpuState::new(id);
                vcpu.set_rip(rip);
                prop_assert_eq!(vcpu.rip, rip);
            }

            #[test]
            fn prop_vcpu_rsp_preserved(id: u8, rsp: u64) {
                let mut vcpu = VcpuState::new(id);
                vcpu.set_rsp(rsp);
                prop_assert_eq!(vcpu.rsp, rsp);
            }

            #[test]
            fn prop_vcpu_interrupt_pending(id: u8, vector: u8) {
                let mut vcpu = VcpuState::new(id);
                vcpu.inject_interrupt(vector);
                let exit = vcpu.pending_exit();
                let matches_interrupt = matches!(exit, Some(VmExitReason::Interrupt(v)) if v == vector);
                prop_assert!(matches_interrupt);
            }

            #[test]
            fn prop_manager_create_unique_ids(count in 1usize..10) {
                let mut manager = VmManager::new();
                let mut ids = Vec::new();

                for _ in 0..count {
                    let id = manager.create_vm(VmConfig::default()).unwrap();
                    prop_assert!(!ids.contains(&id), "Duplicate VM ID generated");
                    ids.push(id);
                }
            }

            #[test]
            fn prop_manager_get_returns_none_for_unknown(id in 1u32..1000) {
                let manager = VmManager::new();
                let result = manager.get(VmId::new(id));
                prop_assert!(result.is_none());
            }

            #[test]
            fn prop_manager_count_accurate(count in 0usize..10) {
                let mut manager = VmManager::new();
                for _ in 0..count {
                    manager.create_vm(VmConfig::default()).unwrap();
                }
                prop_assert_eq!(manager.count(), count);
            }

            #[test]
            fn prop_manager_destroy_reduces_count(count in 2usize..5) {
                let mut manager = VmManager::new();
                let mut ids = Vec::new();
                for _ in 0..count {
                    ids.push(manager.create_vm(VmConfig::default()).unwrap());
                }

                manager.destroy_vm(ids[0]).unwrap();
                prop_assert_eq!(manager.count(), count - 1);
                prop_assert!(manager.get(ids[0]).is_none());
            }

            #[test]
            fn prop_memory_read_after_write(
                data in proptest::collection::vec(any::<u8>(), 1..256)
            ) {
                let mut memory = GuestMemory::new(DEFAULT_GUEST_MEMORY).unwrap();
                let addr = PhysAddr::new(0);

                memory.write(addr, &data).unwrap();

                let mut buf = vec![0u8; data.len()];
                memory.read(addr, &mut buf).unwrap();
                prop_assert_eq!(data, buf);
            }

            #[test]
            fn prop_memory_unallocated_reads_zero(size in 1usize..256) {
                let memory = GuestMemory::new(DEFAULT_GUEST_MEMORY).unwrap();
                let addr = PhysAddr::new(0);

                let mut buf = vec![0xFFu8; size];
                memory.read(addr, &mut buf).unwrap();

                for byte in &buf {
                    prop_assert_eq!(*byte, 0);
                }
            }

            #[test]
            fn prop_virtio_block_serialization(readonly: bool) {
                let config = VirtioDeviceConfig::Block {
                    path: "/dev/vda".into(),
                    readonly,
                };
                let json = serde_json::to_string(&config).unwrap();
                let restored: VirtioDeviceConfig = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(config, restored);
            }

            #[test]
            fn prop_virtio_net_serialization(mac: [u8; 6]) {
                let config = VirtioDeviceConfig::Net { mac };
                let json = serde_json::to_string(&config).unwrap();
                let restored: VirtioDeviceConfig = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(config, restored);
            }

            #[test]
            fn prop_virtio_vsock_serialization(cid: u32) {
                let config = VirtioDeviceConfig::Vsock { cid };
                let json = serde_json::to_string(&config).unwrap();
                let restored: VirtioDeviceConfig = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(config, restored);
            }

            #[test]
            fn prop_vm_stop_preserves_exit_code(exit_code: i32) {
                let config = VmConfig::default();
                let mut vm = MicroVm::create(VmId::new(1), config).unwrap();
                vm.start().unwrap();
                vm.stop(exit_code).unwrap();
                prop_assert_eq!(vm.exit_code, Some(exit_code));
                prop_assert_eq!(vm.state, VmState::Stopped);
            }

            #[test]
            fn prop_vm_step_increments_count(steps in 1u64..100) {
                let config = VmConfig::default();
                let mut vm = MicroVm::create(VmId::new(1), config).unwrap();
                vm.start().unwrap();

                let initial = vm.instruction_count;
                for _ in 0..steps {
                    let _ = vm.step();
                }
                prop_assert_eq!(vm.instruction_count, initial + steps);
            }
        }

        #[test]
        fn prop_vm_state_default_is_created() {
            assert_eq!(VmState::default(), VmState::Created);
        }

        #[test]
        fn prop_vcpu_new_not_running() {
            let vcpu = VcpuState::new(0);
            assert!(!vcpu.running);
            assert!(vcpu.pending_interrupt.is_none());
        }

        #[test]
        fn prop_guest_page_default_is_zero() {
            let page = GuestPage::default();
            for byte in &page.data {
                assert_eq!(*byte, 0);
            }
        }

        #[test]
        fn prop_vm_manager_default_empty() {
            let manager = VmManager::default();
            assert_eq!(manager.count(), 0);
        }

        #[test]
        fn prop_apr_kernel_load_sets_rip() {
            use wos_shared::AprModel;
            let config = VmConfig::default();
            let mut vm = MicroVm::create(VmId::new(1), config).unwrap();
            let apr = AprModel::new(42);
            vm.load_apr_kernel(&apr).unwrap();
            assert_eq!(vm.vcpus[0].rip, KERNEL_LOAD_ADDR);
        }

        #[test]
        fn prop_apr_kernel_load_marks_as_loaded() {
            use wos_shared::AprModel;
            let config = VmConfig::default();
            let mut vm = MicroVm::create(VmId::new(1), config).unwrap();
            assert!(!vm.has_kernel_loaded());
            let apr = AprModel::new(99);
            vm.load_apr_kernel(&apr).unwrap();
            assert!(vm.has_kernel_loaded());
        }

        #[test]
        fn prop_apr_kernel_load_allocates_memory() {
            use wos_shared::AprModel;
            let config = VmConfig::default();
            let mut vm = MicroVm::create(VmId::new(1), config).unwrap();
            let initial_used = vm.memory.used_bytes();
            let apr = AprModel::new(42);
            vm.load_apr_kernel(&apr).unwrap();
            assert!(vm.memory.used_bytes() > initial_used);
        }

        proptest! {
            #[test]
            fn prop_block_read_after_write(
                data in proptest::collection::vec(any::<u8>(), BLOCK_SECTOR_SIZE)
            ) {
                let mut block = VirtioBlock::new("/dev/vda", 4096, false).unwrap();
                block.write_sector(0, &data).unwrap();

                let mut buf = vec![0u8; BLOCK_SECTOR_SIZE];
                block.read_sector(0, &mut buf).unwrap();
                prop_assert_eq!(data, buf);
            }

            #[test]
            fn prop_block_size_aligned_to_sector(size in 1usize..8192) {
                if size > MAX_BLOCK_SIZE {
                    return Ok(());
                }
                let block = VirtioBlock::new("/dev/vda", size, false).unwrap();
                prop_assert_eq!(block.capacity() % BLOCK_SECTOR_SIZE, 0);
            }

            #[test]
            fn prop_block_readonly_rejects_writes(sector in 0u64..8) {
                let mut block = VirtioBlock::new("/dev/vda", 4096, true).unwrap();
                let data = vec![0u8; BLOCK_SECTOR_SIZE];
                let result = block.write_sector(sector, &data);
                let is_error = matches!(result, Err(VmError::VirtioError(_)));
                prop_assert!(is_error);
            }

            #[test]
            fn prop_block_ops_count_accurate(ops in 1usize..10) {
                let mut block = VirtioBlock::new("/dev/vda", 8192, false).unwrap();
                let buf = vec![0u8; BLOCK_SECTOR_SIZE];

                for i in 0..ops {
                    let sector = (i % 16) as u64;
                    block.read_sector(sector, &mut buf.clone()).unwrap();
                    block.write_sector(sector, &buf).unwrap();
                }

                prop_assert_eq!(block.read_ops() as usize, ops);
                prop_assert_eq!(block.write_ops() as usize, ops);
            }
        }

        #[test]
        fn prop_block_from_data_preserves_content() {
            let original = vec![0x42u8; BLOCK_SECTOR_SIZE];
            let mut block = VirtioBlock::from_data("/dev/vda", original.clone(), false).unwrap();

            let mut buf = vec![0u8; BLOCK_SECTOR_SIZE];
            block.read_sector(0, &mut buf).unwrap();
            assert_eq!(original, buf);
        }

        #[test]
        fn prop_vm_lifecycle_works() {
            let config = VmConfig::default();
            let mut vm = MicroVm::create(VmId::new(1), config).unwrap();
            assert_eq!(vm.state, VmState::Created);
            vm.start().unwrap();
            assert_eq!(vm.state, VmState::Running);
            vm.pause().unwrap();
            assert_eq!(vm.state, VmState::Paused);
            vm.resume().unwrap();
            assert_eq!(vm.state, VmState::Running);
            vm.stop(0).unwrap();
            assert_eq!(vm.state, VmState::Stopped);
        }

        #[test]
        fn prop_config_validation() {
            let good = VmConfig::default();
            assert!(good.validate().is_ok());

            let zero_mem = VmConfig {
                memory_bytes: 0,
                ..Default::default()
            };
            assert!(zero_mem.validate().is_err());

            let too_much = VmConfig {
                memory_bytes: MAX_GUEST_MEMORY + 1,
                ..Default::default()
            };
            assert!(too_much.validate().is_err());

            let zero_vcpu = VmConfig {
                vcpu_count: 0,
                ..Default::default()
            };
            assert!(zero_vcpu.validate().is_err());
        }

        #[test]
        fn prop_virtio_console_serialization() {
            let config = VirtioDeviceConfig::Console;
            let json = serde_json::to_string(&config).unwrap();
            let restored: VirtioDeviceConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config, restored);
        }
    }
}
