//! Shared Memory IPC
//!
//! POSIX-like shared memory for fast inter-process communication.
//! Allows multiple processes to map the same physical memory into their
//! virtual address spaces.

use crate::memory::{PagePermissions, VirtualAddress};
use crate::state::ProcessId;
use im::{HashMap, HashSet, Vector};
use serde::{Deserialize, Serialize};

/// Shared memory segment identifier
pub type SharedMemoryId = u32;

/// Shared memory segment
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SharedMemorySegment {
    /// Unique identifier
    pub id: SharedMemoryId,
    /// Size in bytes
    pub size: usize,
    /// Permissions (read, write, execute)
    pub permissions: PagePermissions,
    /// Actual data (simulated physical memory)
    pub data: Vector<u8>,
    /// Processes that have mapped this segment
    pub mapped_processes: HashSet<ProcessId>,
    /// Reference count (for cleanup)
    pub ref_count: usize,
}

impl SharedMemorySegment {
    /// Create a new shared memory segment
    pub fn new(id: SharedMemoryId, size: usize, permissions: PagePermissions) -> Self {
        Self {
            id,
            size,
            permissions,
            data: Vector::from_iter(vec![0u8; size]),
            mapped_processes: HashSet::new(),
            ref_count: 0,
        }
    }

    /// Map this segment to a process
    pub fn map_to_process(&mut self, pid: ProcessId) -> Result<(), &'static str> {
        if self.mapped_processes.contains(&pid) {
            return Err("Process has already mapped this segment");
        }
        self.mapped_processes.insert(pid);
        self.ref_count += 1;
        Ok(())
    }

    /// Unmap this segment from a process
    pub fn unmap_from_process(&mut self, pid: ProcessId) -> Result<(), &'static str> {
        if !self.mapped_processes.contains(&pid) {
            return Err("Process has not mapped this segment");
        }
        self.mapped_processes.remove(&pid);
        self.ref_count = self.ref_count.saturating_sub(1);
        Ok(())
    }

    /// Read data from segment
    pub fn read(&self, offset: usize, length: usize) -> Result<Vector<u8>, &'static str> {
        if offset + length > self.size {
            return Err("Read beyond segment bounds");
        }
        if !self.permissions.allows_read() {
            return Err("Segment is not readable");
        }

        let end = offset + length;
        Ok(self.data.clone().slice(offset..end))
    }

    /// Write data to segment
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), &'static str> {
        if offset + data.len() > self.size {
            return Err("Write beyond segment bounds");
        }
        if !self.permissions.allows_write() {
            return Err("Segment is not writable");
        }

        // Update data (im::Vector is persistent, so we need to rebuild)
        let mut new_data = self.data.clone();
        for (i, &byte) in data.iter().enumerate() {
            new_data.set(offset + i, byte);
        }
        self.data = new_data;
        Ok(())
    }
}

/// Process mapping information
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessMapping {
    /// Shared memory segment ID
    pub shm_id: SharedMemoryId,
    /// Virtual address where segment is mapped
    pub virtual_addr: VirtualAddress,
    /// Size of mapping
    pub size: usize,
    /// Permissions for this mapping
    pub permissions: PagePermissions,
}

/// Shared memory manager
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SharedMemoryManager {
    /// All shared memory segments
    pub segments: HashMap<SharedMemoryId, SharedMemorySegment>,
    /// Next available segment ID
    pub next_id: SharedMemoryId,
    /// Process mappings (ProcessId -> list of mappings)
    pub process_mappings: HashMap<ProcessId, Vector<ProcessMapping>>,
}

impl SharedMemoryManager {
    /// Create a new shared memory manager
    pub fn new() -> Self {
        Self {
            segments: HashMap::new(),
            next_id: 1,
            process_mappings: HashMap::new(),
        }
    }

    /// Create a new shared memory segment
    pub fn create_segment(&mut self, size: usize, permissions: PagePermissions) -> SharedMemoryId {
        let id = self.next_id;
        self.next_id += 1;
        let segment = SharedMemorySegment::new(id, size, permissions);
        self.segments.insert(id, segment);
        id
    }

    /// Map a shared memory segment into a process's address space
    pub fn map_segment(
        &mut self,
        shm_id: SharedMemoryId,
        pid: ProcessId,
        virtual_addr: VirtualAddress,
        permissions: PagePermissions,
    ) -> Result<(), &'static str> {
        // Get the segment
        let segment = self
            .segments
            .get_mut(&shm_id)
            .ok_or("Shared memory segment not found")?;

        // Check permissions compatibility
        if permissions.allows_read() && !segment.permissions.allows_read() {
            return Err("Requested read permission not available on segment");
        }
        if permissions.allows_write() && !segment.permissions.allows_write() {
            return Err("Requested write permission not available on segment");
        }

        // Map to process
        segment.map_to_process(pid)?;

        // Record mapping
        let mapping = ProcessMapping {
            shm_id,
            virtual_addr,
            size: segment.size,
            permissions,
        };

        let mut mappings = self
            .process_mappings
            .get(&pid)
            .cloned()
            .unwrap_or_else(Vector::new);
        mappings.push_back(mapping);
        self.process_mappings.insert(pid, mappings);

        Ok(())
    }

    /// Unmap a shared memory segment from a process
    pub fn unmap_segment(
        &mut self,
        shm_id: SharedMemoryId,
        pid: ProcessId,
    ) -> Result<(), &'static str> {
        // Get the segment
        let segment = self
            .segments
            .get_mut(&shm_id)
            .ok_or("Shared memory segment not found")?;

        // Unmap from process
        segment.unmap_from_process(pid)?;

        // Remove mapping
        if let Some(mappings) = self.process_mappings.get(&pid) {
            let new_mappings: Vector<_> = mappings
                .iter()
                .filter(|m| m.shm_id != shm_id)
                .cloned()
                .collect();
            if new_mappings.is_empty() {
                self.process_mappings.remove(&pid);
            } else {
                self.process_mappings.insert(pid, new_mappings);
            }
        }

        Ok(())
    }

    /// Destroy a shared memory segment (if ref count is zero)
    pub fn destroy_segment(&mut self, shm_id: SharedMemoryId) -> Result<(), &'static str> {
        let segment = self
            .segments
            .get(&shm_id)
            .ok_or("Shared memory segment not found")?;

        if segment.ref_count > 0 {
            return Err("Cannot destroy segment with active mappings");
        }

        self.segments.remove(&shm_id);
        Ok(())
    }

    /// Read from a shared memory segment
    pub fn read(
        &self,
        shm_id: SharedMemoryId,
        pid: ProcessId,
        offset: usize,
        length: usize,
    ) -> Result<Vector<u8>, &'static str> {
        let segment = self
            .segments
            .get(&shm_id)
            .ok_or("Shared memory segment not found")?;

        // Check if process has mapped this segment
        if !segment.mapped_processes.contains(&pid) {
            return Err("Process has not mapped this segment");
        }

        segment.read(offset, length)
    }

    /// Write to a shared memory segment
    pub fn write(
        &mut self,
        shm_id: SharedMemoryId,
        pid: ProcessId,
        offset: usize,
        data: &[u8],
    ) -> Result<(), &'static str> {
        // Check if process has mapped this segment
        let segment = self
            .segments
            .get(&shm_id)
            .ok_or("Shared memory segment not found")?;

        if !segment.mapped_processes.contains(&pid) {
            return Err("Process has not mapped this segment");
        }

        // Write the data
        let segment = self.segments.get_mut(&shm_id).unwrap();
        segment.write(offset, data)
    }

    /// Get all mappings for a process
    pub fn get_process_mappings(&self, pid: ProcessId) -> Vector<ProcessMapping> {
        self.process_mappings
            .get(&pid)
            .cloned()
            .unwrap_or_else(Vector::new)
    }

    /// Cleanup all mappings for a process (called when process terminates)
    pub fn cleanup_process(&mut self, pid: ProcessId) {
        if let Some(mappings) = self.process_mappings.get(&pid) {
            let shm_ids: Vec<_> = mappings.iter().map(|m| m.shm_id).collect();
            for shm_id in shm_ids {
                let _ = self.unmap_segment(shm_id, pid);
            }
        }
        self.process_mappings.remove(&pid);
    }
}

impl Default for SharedMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // WOS-013: Shared Memory Tests
    mod shm_tests {
        use super::*;

        #[test]
        fn test_segment_creation() {
            let perms = PagePermissions::read_write();
            let segment = SharedMemorySegment::new(1, 4096, perms);

            assert_eq!(segment.id, 1);
            assert_eq!(segment.size, 4096);
            assert_eq!(segment.permissions, perms);
            assert_eq!(segment.data.len(), 4096);
            assert_eq!(segment.ref_count, 0);
            assert!(segment.mapped_processes.is_empty());
        }

        #[test]
        fn test_segment_map_unmap() {
            let mut segment = SharedMemorySegment::new(1, 4096, PagePermissions::read_write());

            // Map to process 100
            assert!(segment.map_to_process(100).is_ok());
            assert_eq!(segment.ref_count, 1);
            assert!(segment.mapped_processes.contains(&100));

            // Cannot map same process twice
            assert!(segment.map_to_process(100).is_err());

            // Map to process 200
            assert!(segment.map_to_process(200).is_ok());
            assert_eq!(segment.ref_count, 2);

            // Unmap process 100
            assert!(segment.unmap_from_process(100).is_ok());
            assert_eq!(segment.ref_count, 1);
            assert!(!segment.mapped_processes.contains(&100));

            // Cannot unmap process that hasn't mapped
            assert!(segment.unmap_from_process(300).is_err());
        }

        #[test]
        fn test_segment_read_write() {
            let mut segment = SharedMemorySegment::new(1, 100, PagePermissions::read_write());

            // Write data
            let data = b"Hello, WOS!";
            assert!(segment.write(0, data).is_ok());

            // Read data back
            let read_data = segment.read(0, data.len()).unwrap();
            assert_eq!(read_data.len(), data.len());
            for (i, &byte) in data.iter().enumerate() {
                assert_eq!(read_data[i], byte);
            }
        }

        #[test]
        fn test_segment_read_write_bounds() {
            let mut segment = SharedMemorySegment::new(1, 100, PagePermissions::read_write());

            // Write beyond bounds should fail
            assert!(segment.write(95, b"Too long!").is_err());

            // Read beyond bounds should fail
            assert!(segment.read(90, 20).is_err());
        }

        #[test]
        fn test_segment_permissions() {
            // Read-only segment
            let mut ro_segment = SharedMemorySegment::new(1, 100, PagePermissions::read_only());
            assert!(ro_segment.write(0, b"test").is_err());
            assert!(ro_segment.read(0, 4).is_ok());

            // Write-only would need custom permissions
            let wo_perms = PagePermissions {
                read: false,
                write: true,
                execute: false,
            };
            let mut wo_segment = SharedMemorySegment::new(2, 100, wo_perms);
            assert!(wo_segment.write(0, b"test").is_ok());
            assert!(wo_segment.read(0, 4).is_err());
        }

        #[test]
        fn test_manager_creation() {
            let manager = SharedMemoryManager::new();
            assert!(manager.segments.is_empty());
            assert_eq!(manager.next_id, 1);
            assert!(manager.process_mappings.is_empty());
        }

        #[test]
        fn test_manager_create_segment() {
            let mut manager = SharedMemoryManager::new();

            let id1 = manager.create_segment(4096, PagePermissions::read_write());
            assert_eq!(id1, 1);
            assert!(manager.segments.contains_key(&1));

            let id2 = manager.create_segment(8192, PagePermissions::read_only());
            assert_eq!(id2, 2);
            assert_eq!(manager.segments.get(&2).unwrap().size, 8192);
        }

        #[test]
        fn test_manager_map_unmap() {
            let mut manager = SharedMemoryManager::new();
            let shm_id = manager.create_segment(4096, PagePermissions::read_write());

            // Map to process 100
            assert!(manager
                .map_segment(shm_id, 100, 0x1000, PagePermissions::read_write())
                .is_ok());

            let mappings = manager.get_process_mappings(100);
            assert_eq!(mappings.len(), 1);
            assert_eq!(mappings[0].shm_id, shm_id);
            assert_eq!(mappings[0].virtual_addr, 0x1000);

            // Unmap
            assert!(manager.unmap_segment(shm_id, 100).is_ok());
            assert!(manager.get_process_mappings(100).is_empty());
        }

        #[test]
        fn test_manager_permission_checks() {
            let mut manager = SharedMemoryManager::new();
            let shm_id = manager.create_segment(4096, PagePermissions::read_only());

            // Cannot map with write permission if segment is read-only
            let result = manager.map_segment(shm_id, 100, 0x1000, PagePermissions::read_write());
            assert!(result.is_err());

            // Can map with read-only permission
            assert!(manager
                .map_segment(shm_id, 100, 0x1000, PagePermissions::read_only())
                .is_ok());
        }

        #[test]
        fn test_manager_shared_access() {
            let mut manager = SharedMemoryManager::new();
            let shm_id = manager.create_segment(100, PagePermissions::read_write());

            // Map to two processes
            assert!(manager
                .map_segment(shm_id, 100, 0x1000, PagePermissions::read_write())
                .is_ok());
            assert!(manager
                .map_segment(shm_id, 200, 0x2000, PagePermissions::read_write())
                .is_ok());

            // Process 100 writes data
            assert!(manager.write(shm_id, 100, 0, b"Shared").is_ok());

            // Process 200 reads the same data
            let data = manager.read(shm_id, 200, 0, 6).unwrap();
            assert_eq!(data.len(), 6);
            let data_bytes: Vec<u8> = data.iter().copied().collect();
            assert_eq!(&data_bytes[..], b"Shared");
        }

        #[test]
        fn test_manager_destroy_segment() {
            let mut manager = SharedMemoryManager::new();
            let shm_id = manager.create_segment(4096, PagePermissions::read_write());

            // Map to process
            manager
                .map_segment(shm_id, 100, 0x1000, PagePermissions::read_write())
                .unwrap();

            // Cannot destroy while mapped
            assert!(manager.destroy_segment(shm_id).is_err());

            // Unmap and destroy
            manager.unmap_segment(shm_id, 100).unwrap();
            assert!(manager.destroy_segment(shm_id).is_ok());
            assert!(!manager.segments.contains_key(&shm_id));
        }

        #[test]
        fn test_manager_cleanup_process() {
            let mut manager = SharedMemoryManager::new();
            let shm1 = manager.create_segment(4096, PagePermissions::read_write());
            let shm2 = manager.create_segment(4096, PagePermissions::read_write());

            // Process 100 maps both segments
            manager
                .map_segment(shm1, 100, 0x1000, PagePermissions::read_write())
                .unwrap();
            manager
                .map_segment(shm2, 100, 0x2000, PagePermissions::read_write())
                .unwrap();

            assert_eq!(manager.get_process_mappings(100).len(), 2);
            assert_eq!(manager.segments.get(&shm1).unwrap().ref_count, 1);
            assert_eq!(manager.segments.get(&shm2).unwrap().ref_count, 1);

            // Cleanup process
            manager.cleanup_process(100);

            assert!(manager.get_process_mappings(100).is_empty());
            assert_eq!(manager.segments.get(&shm1).unwrap().ref_count, 0);
            assert_eq!(manager.segments.get(&shm2).unwrap().ref_count, 0);
        }

        #[test]
        fn test_manager_multi_process_ref_counting() {
            let mut manager = SharedMemoryManager::new();
            let shm_id = manager.create_segment(4096, PagePermissions::read_write());

            // Three processes map the same segment
            manager
                .map_segment(shm_id, 100, 0x1000, PagePermissions::read_write())
                .unwrap();
            manager
                .map_segment(shm_id, 200, 0x2000, PagePermissions::read_write())
                .unwrap();
            manager
                .map_segment(shm_id, 300, 0x3000, PagePermissions::read_write())
                .unwrap();

            assert_eq!(manager.segments.get(&shm_id).unwrap().ref_count, 3);

            // Unmap from one process
            manager.unmap_segment(shm_id, 200).unwrap();
            assert_eq!(manager.segments.get(&shm_id).unwrap().ref_count, 2);

            // Cannot destroy yet
            assert!(manager.destroy_segment(shm_id).is_err());

            // Unmap from remaining processes
            manager.unmap_segment(shm_id, 100).unwrap();
            manager.unmap_segment(shm_id, 300).unwrap();
            assert_eq!(manager.segments.get(&shm_id).unwrap().ref_count, 0);

            // Now can destroy
            assert!(manager.destroy_segment(shm_id).is_ok());
        }
    }
}
