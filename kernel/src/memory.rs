//! Virtual Memory Management
//!
//! Virtual memory with page tables and address translation.

use im::HashMap;
use serde::{Deserialize, Serialize};

/// Page size in bytes (4KB)
pub const PAGE_SIZE: usize = 4096;

/// Virtual page number
pub type VirtualPage = u32;

/// Physical page number
pub type PhysicalPage = u32;

/// Virtual address
pub type VirtualAddress = u64;

/// Physical address
pub type PhysicalAddress = u64;

/// Page permissions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PagePermissions {
    /// Page is readable
    pub read: bool,
    /// Page is writable
    pub write: bool,
    /// Page is executable
    pub execute: bool,
}

impl PagePermissions {
    /// Create read-only permissions
    pub const fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
        }
    }

    /// Create read-write permissions
    pub const fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
        }
    }

    /// Create read-execute permissions
    pub const fn read_execute() -> Self {
        Self {
            read: true,
            write: false,
            execute: true,
        }
    }

    /// Create read-write-execute permissions
    pub const fn read_write_execute() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }

    /// Create no permissions
    pub const fn none() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
        }
    }

    /// Check if permission allows read access
    pub fn allows_read(&self) -> bool {
        self.read
    }

    /// Check if permission allows write access
    pub fn allows_write(&self) -> bool {
        self.write
    }

    /// Check if permission allows execute access
    pub fn allows_execute(&self) -> bool {
        self.execute
    }
}

impl Default for PagePermissions {
    fn default() -> Self {
        Self::read_write()
    }
}

/// Page table entry with permissions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageTableEntry {
    /// Physical page number
    pub physical_page: PhysicalPage,
    /// Page permissions
    pub permissions: PagePermissions,
}

impl PageTableEntry {
    /// Create new page table entry
    pub fn new(physical_page: PhysicalPage, permissions: PagePermissions) -> Self {
        Self {
            physical_page,
            permissions,
        }
    }
}

/// Memory region type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryRegion {
    /// Code segment (executable)
    Code,
    /// Data segment (read/write)
    Data,
    /// Heap (dynamically allocated)
    Heap,
    /// Stack (grows downward)
    Stack,
}

/// Memory layout configuration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryLayout {
    /// Code region start address
    pub code_start: VirtualAddress,
    /// Code region size in bytes
    pub code_size: usize,
    /// Data region start address
    pub data_start: VirtualAddress,
    /// Data region size in bytes
    pub data_size: usize,
    /// Heap region start address
    pub heap_start: VirtualAddress,
    /// Heap region size in bytes
    pub heap_size: usize,
    /// Stack region start address
    pub stack_start: VirtualAddress,
    /// Stack region size in bytes
    pub stack_size: usize,
}

impl Default for MemoryLayout {
    fn default() -> Self {
        Self {
            code_start: 0x0000_0000_1000_0000,  // 256MB
            code_size: 16 * 1024 * 1024,        // 16MB
            data_start: 0x0000_0000_2000_0000,  // 512MB
            data_size: 16 * 1024 * 1024,        // 16MB
            heap_start: 0x0000_0000_3000_0000,  // 768MB
            heap_size: 256 * 1024 * 1024,       // 256MB
            stack_start: 0x0000_0000_7FFF_F000, // ~2GB (grows down)
            stack_size: 8 * 1024 * 1024,        // 8MB
        }
    }
}

impl MemoryLayout {
    /// Get region type for virtual address
    pub fn region_for_address(&self, addr: VirtualAddress) -> Option<MemoryRegion> {
        if addr >= self.code_start && addr < self.code_start + self.code_size as u64 {
            Some(MemoryRegion::Code)
        } else if addr >= self.data_start && addr < self.data_start + self.data_size as u64 {
            Some(MemoryRegion::Data)
        } else if addr >= self.heap_start && addr < self.heap_start + self.heap_size as u64 {
            Some(MemoryRegion::Heap)
        } else if addr >= (self.stack_start - self.stack_size as u64) && addr <= self.stack_start {
            Some(MemoryRegion::Stack)
        } else {
            None
        }
    }
}

/// Memory access type for permission checking
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryAccess {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Execute access
    Execute,
}

/// Virtual memory management with page tables
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualMemory {
    /// Page table (virtual page -> page table entry with permissions)
    page_table: HashMap<VirtualPage, PageTableEntry>,
    /// Memory layout
    layout: MemoryLayout,
    /// Next physical page to allocate
    next_physical_page: PhysicalPage,
    /// Next virtual address for heap allocations
    next_heap_addr: VirtualAddress,
}

impl VirtualMemory {
    /// Create new virtual memory with default layout
    pub fn new() -> Self {
        let layout = MemoryLayout::default();
        Self {
            page_table: HashMap::new(),
            next_heap_addr: layout.heap_start,
            layout,
            next_physical_page: 0,
        }
    }

    /// Create with custom layout
    pub fn with_layout(layout: MemoryLayout) -> Self {
        let next_heap_addr = layout.heap_start;
        Self {
            page_table: HashMap::new(),
            next_heap_addr,
            layout,
            next_physical_page: 0,
        }
    }

    /// Get memory layout
    pub fn layout(&self) -> &MemoryLayout {
        &self.layout
    }

    /// Map virtual page to physical page with permissions
    pub fn map_page(
        &mut self,
        vpage: VirtualPage,
        ppage: PhysicalPage,
        permissions: PagePermissions,
    ) {
        let entry = PageTableEntry::new(ppage, permissions);
        self.page_table.insert(vpage, entry);
    }

    /// Unmap virtual page
    pub fn unmap_page(&mut self, vpage: VirtualPage) -> Option<PageTableEntry> {
        self.page_table.remove(&vpage)
    }

    /// Allocate a new physical page and map it to virtual page with permissions
    pub fn allocate_page(
        &mut self,
        vpage: VirtualPage,
        permissions: PagePermissions,
    ) -> PhysicalPage {
        let ppage = self.next_physical_page;
        self.next_physical_page += 1;
        let entry = PageTableEntry::new(ppage, permissions);
        self.page_table.insert(vpage, entry);
        ppage
    }

    /// Get physical page for virtual page
    pub fn get_physical_page(&self, vpage: VirtualPage) -> Option<PhysicalPage> {
        self.page_table.get(&vpage).map(|entry| entry.physical_page)
    }

    /// Get page table entry for virtual page
    pub fn get_page_entry(&self, vpage: VirtualPage) -> Option<&PageTableEntry> {
        self.page_table.get(&vpage)
    }

    /// Get page permissions for virtual page
    pub fn get_page_permissions(&self, vpage: VirtualPage) -> Option<PagePermissions> {
        self.page_table.get(&vpage).map(|entry| entry.permissions)
    }

    /// Set page permissions for virtual page
    pub fn set_page_permissions(
        &mut self,
        vpage: VirtualPage,
        permissions: PagePermissions,
    ) -> bool {
        if let Some(entry) = self.page_table.get(&vpage) {
            let new_entry = PageTableEntry::new(entry.physical_page, permissions);
            self.page_table.insert(vpage, new_entry);
            true
        } else {
            false
        }
    }

    /// Check if access to virtual page is allowed
    pub fn check_permission(&self, vpage: VirtualPage, access: MemoryAccess) -> bool {
        if let Some(entry) = self.page_table.get(&vpage) {
            match access {
                MemoryAccess::Read => entry.permissions.allows_read(),
                MemoryAccess::Write => entry.permissions.allows_write(),
                MemoryAccess::Execute => entry.permissions.allows_execute(),
            }
        } else {
            false // Unmapped page = no permission
        }
    }

    /// Check if access to virtual address is allowed
    pub fn check_address_permission(&self, vaddr: VirtualAddress, access: MemoryAccess) -> bool {
        let vpage = Self::vaddr_to_page(vaddr);
        self.check_permission(vpage, access)
    }

    /// Translate virtual address to physical address
    pub fn translate(&self, vaddr: VirtualAddress) -> Option<PhysicalAddress> {
        let vpage = (vaddr / PAGE_SIZE as u64) as VirtualPage;
        let offset = vaddr % PAGE_SIZE as u64;

        self.page_table
            .get(&vpage)
            .map(|entry| (entry.physical_page as u64) * PAGE_SIZE as u64 + offset)
    }

    /// Check if virtual page is mapped
    pub fn is_mapped(&self, vpage: VirtualPage) -> bool {
        self.page_table.contains_key(&vpage)
    }

    /// Get number of mapped pages
    pub fn mapped_page_count(&self) -> usize {
        self.page_table.len()
    }

    /// Virtual address to page number
    pub fn vaddr_to_page(vaddr: VirtualAddress) -> VirtualPage {
        (vaddr / PAGE_SIZE as u64) as VirtualPage
    }

    /// Page number to virtual address
    pub fn page_to_vaddr(vpage: VirtualPage) -> VirtualAddress {
        (vpage as u64) * PAGE_SIZE as u64
    }

    /// Allocate contiguous virtual memory region (mmap)
    ///
    /// Returns the starting virtual address of the allocated region.
    /// Allocates page-aligned memory in the heap region with read-write permissions.
    pub fn mmap(&mut self, size_bytes: usize) -> Option<VirtualAddress> {
        self.mmap_with_permissions(size_bytes, PagePermissions::read_write())
    }

    /// Allocate contiguous virtual memory region with specific permissions
    ///
    /// Returns the starting virtual address of the allocated region.
    /// Allocates page-aligned memory in the heap region.
    pub fn mmap_with_permissions(
        &mut self,
        size_bytes: usize,
        permissions: PagePermissions,
    ) -> Option<VirtualAddress> {
        if size_bytes == 0 {
            return None;
        }

        // Round up to page boundary
        let num_pages = size_bytes.div_ceil(PAGE_SIZE);
        let start_addr = self.next_heap_addr;

        // Check if allocation fits in heap region
        let end_addr = start_addr + (num_pages * PAGE_SIZE) as u64;
        let heap_end = self.layout.heap_start + self.layout.heap_size as u64;
        if end_addr > heap_end {
            return None; // Out of memory
        }

        // Allocate pages with permissions
        for i in 0..num_pages {
            let vaddr = start_addr + (i * PAGE_SIZE) as u64;
            let vpage = Self::vaddr_to_page(vaddr);
            let ppage = self.next_physical_page;
            self.next_physical_page += 1;
            let entry = PageTableEntry::new(ppage, permissions);
            self.page_table.insert(vpage, entry);
        }

        // Update next heap address
        self.next_heap_addr = end_addr;

        Some(start_addr)
    }

    /// Free virtual memory region (munmap)
    ///
    /// Unmaps all pages in the specified range.
    /// Returns true if successful, false if any page was not mapped.
    pub fn munmap(&mut self, addr: VirtualAddress, size_bytes: usize) -> bool {
        if size_bytes == 0 {
            return true;
        }

        // Round up to page boundary
        let num_pages = size_bytes.div_ceil(PAGE_SIZE);

        // Check if all pages are mapped before unmapping
        let mut pages_to_unmap = Vec::new();
        for i in 0..num_pages {
            let vaddr = addr + (i * PAGE_SIZE) as u64;
            let vpage = Self::vaddr_to_page(vaddr);
            if !self.is_mapped(vpage) {
                return false; // Can't unmap unmapped page
            }
            pages_to_unmap.push(vpage);
        }

        // Unmap all pages
        for vpage in pages_to_unmap {
            self.unmap_page(vpage);
        }

        true
    }

    /// Check if address range is fully mapped
    pub fn is_range_mapped(&self, addr: VirtualAddress, size_bytes: usize) -> bool {
        if size_bytes == 0 {
            return true;
        }

        let num_pages = size_bytes.div_ceil(PAGE_SIZE);
        for i in 0..num_pages {
            let vaddr = addr + (i * PAGE_SIZE) as u64;
            let vpage = Self::vaddr_to_page(vaddr);
            if !self.is_mapped(vpage) {
                return false;
            }
        }
        true
    }
}

impl Default for VirtualMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_memory_creation() {
        let vm = VirtualMemory::new();
        assert_eq!(vm.mapped_page_count(), 0);
        assert_eq!(vm.next_physical_page, 0);

        let layout = vm.layout();
        assert_eq!(layout.code_start, 0x0000_0000_1000_0000);
        assert_eq!(layout.code_size, 16 * 1024 * 1024);
    }

    #[test]
    fn test_page_table_operations() {
        let mut vm = VirtualMemory::new();
        let perms = PagePermissions::read_write();

        // Map pages
        vm.map_page(0, 100, perms);
        vm.map_page(1, 101, perms);
        vm.map_page(2, 102, perms);

        assert_eq!(vm.mapped_page_count(), 3);

        // Lookup
        assert_eq!(vm.get_physical_page(0), Some(100));
        assert_eq!(vm.get_physical_page(1), Some(101));
        assert_eq!(vm.get_physical_page(2), Some(102));
        assert_eq!(vm.get_physical_page(3), None);

        // Unmap
        let entry = vm.unmap_page(1);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().physical_page, 101);
        assert_eq!(vm.mapped_page_count(), 2);
        assert_eq!(vm.get_physical_page(1), None);
    }

    #[test]
    fn test_address_translation() {
        let mut vm = VirtualMemory::new();
        let perms = PagePermissions::read_write();

        // Map virtual page 0 to physical page 100
        vm.map_page(0, 100, perms);

        // Translate address in page 0
        let vaddr = 0x0000; // Offset 0 in page 0
        let paddr = vm.translate(vaddr);
        assert_eq!(paddr, Some(100 * PAGE_SIZE as u64));

        // Translate with offset
        let vaddr = 0x0100; // Offset 256 in page 0
        let paddr = vm.translate(vaddr);
        assert_eq!(paddr, Some(100 * PAGE_SIZE as u64 + 256));

        // Unmapped page returns None
        let vaddr = PAGE_SIZE as u64; // Page 1, unmapped
        let paddr = vm.translate(vaddr);
        assert_eq!(paddr, None);
    }

    #[test]
    fn test_allocate_page() {
        let mut vm = VirtualMemory::new();
        let perms = PagePermissions::read_write();

        let ppage0 = vm.allocate_page(0, perms);
        assert_eq!(ppage0, 0);
        assert_eq!(vm.get_physical_page(0), Some(0));

        let ppage1 = vm.allocate_page(1, perms);
        assert_eq!(ppage1, 1);
        assert_eq!(vm.get_physical_page(1), Some(1));

        assert_eq!(vm.mapped_page_count(), 2);
    }

    #[test]
    fn test_memory_layout_regions() {
        let layout = MemoryLayout::default();

        // Code region
        let code_addr = layout.code_start;
        assert_eq!(
            layout.region_for_address(code_addr),
            Some(MemoryRegion::Code)
        );

        // Data region
        let data_addr = layout.data_start;
        assert_eq!(
            layout.region_for_address(data_addr),
            Some(MemoryRegion::Data)
        );

        // Heap region
        let heap_addr = layout.heap_start;
        assert_eq!(
            layout.region_for_address(heap_addr),
            Some(MemoryRegion::Heap)
        );

        // Stack region
        let stack_addr = layout.stack_start;
        assert_eq!(
            layout.region_for_address(stack_addr),
            Some(MemoryRegion::Stack)
        );

        // Invalid region
        let invalid_addr = 0x0;
        assert_eq!(layout.region_for_address(invalid_addr), None);
    }

    #[test]
    fn test_vaddr_page_conversion() {
        let vaddr = 0x5000; // 5 pages * 4096 bytes
        let vpage = VirtualMemory::vaddr_to_page(vaddr);
        assert_eq!(vpage, 5);

        let vaddr2 = VirtualMemory::page_to_vaddr(vpage);
        assert_eq!(vaddr2, 5 * PAGE_SIZE as u64);
    }

    #[test]
    fn test_is_mapped() {
        let mut vm = VirtualMemory::new();
        let perms = PagePermissions::read_write();
        assert!(!vm.is_mapped(0));

        vm.map_page(0, 100, perms);
        assert!(vm.is_mapped(0));
        assert!(!vm.is_mapped(1));
    }

    #[test]
    fn test_mmap_basic() {
        let mut vm = VirtualMemory::new();
        let layout = vm.layout().clone();

        // Allocate 1 page
        let addr = vm.mmap(PAGE_SIZE).unwrap();
        assert_eq!(addr, layout.heap_start);
        assert_eq!(vm.mapped_page_count(), 1);

        // Verify page is mapped
        let vpage = VirtualMemory::vaddr_to_page(addr);
        assert!(vm.is_mapped(vpage));
    }

    #[test]
    fn test_mmap_multiple_pages() {
        let mut vm = VirtualMemory::new();
        let layout = vm.layout().clone();

        // Allocate 3 pages
        let addr = vm.mmap(3 * PAGE_SIZE).unwrap();
        assert_eq!(addr, layout.heap_start);
        assert_eq!(vm.mapped_page_count(), 3);

        // Verify all pages are mapped
        for i in 0..3 {
            let vpage = VirtualMemory::vaddr_to_page(addr + (i * PAGE_SIZE) as u64);
            assert!(vm.is_mapped(vpage));
        }
    }

    #[test]
    fn test_mmap_rounds_up_to_page() {
        let mut vm = VirtualMemory::new();

        // Allocate 1 byte (should round up to 1 page)
        let addr = vm.mmap(1).unwrap();
        assert_eq!(vm.mapped_page_count(), 1);

        // Allocate PAGE_SIZE + 1 bytes (should round up to 2 pages)
        let addr2 = vm.mmap(PAGE_SIZE + 1).unwrap();
        assert_eq!(vm.mapped_page_count(), 3); // 1 + 2 pages
        assert!(addr2 > addr);
    }

    #[test]
    fn test_mmap_zero_size() {
        let mut vm = VirtualMemory::new();

        // Zero-size allocation should return None
        let result = vm.mmap(0);
        assert_eq!(result, None);
        assert_eq!(vm.mapped_page_count(), 0);
    }

    #[test]
    fn test_mmap_sequential_allocations() {
        let mut vm = VirtualMemory::new();
        let layout = vm.layout().clone();

        // First allocation
        let addr1 = vm.mmap(PAGE_SIZE).unwrap();
        assert_eq!(addr1, layout.heap_start);

        // Second allocation should be contiguous
        let addr2 = vm.mmap(PAGE_SIZE).unwrap();
        assert_eq!(addr2, layout.heap_start + PAGE_SIZE as u64);

        // Third allocation
        let addr3 = vm.mmap(2 * PAGE_SIZE).unwrap();
        assert_eq!(addr3, layout.heap_start + (2 * PAGE_SIZE) as u64);

        assert_eq!(vm.mapped_page_count(), 4); // 1 + 1 + 2 pages
    }

    #[test]
    fn test_mmap_out_of_memory() {
        let mut vm = VirtualMemory::new();
        let layout = vm.layout().clone();

        // Try to allocate more than heap size
        let result = vm.mmap(layout.heap_size + PAGE_SIZE);
        assert_eq!(result, None);

        // Allocate maximum heap size
        let addr = vm.mmap(layout.heap_size).unwrap();
        assert_eq!(addr, layout.heap_start);

        // Next allocation should fail
        let result2 = vm.mmap(PAGE_SIZE);
        assert_eq!(result2, None);
    }

    #[test]
    fn test_munmap_basic() {
        let mut vm = VirtualMemory::new();

        // Allocate and then free
        let addr = vm.mmap(PAGE_SIZE).unwrap();
        assert_eq!(vm.mapped_page_count(), 1);

        let result = vm.munmap(addr, PAGE_SIZE);
        assert!(result);
        assert_eq!(vm.mapped_page_count(), 0);

        // Verify page is unmapped
        let vpage = VirtualMemory::vaddr_to_page(addr);
        assert!(!vm.is_mapped(vpage));
    }

    #[test]
    fn test_munmap_multiple_pages() {
        let mut vm = VirtualMemory::new();

        // Allocate 5 pages
        let addr = vm.mmap(5 * PAGE_SIZE).unwrap();
        assert_eq!(vm.mapped_page_count(), 5);

        // Free all 5 pages
        let result = vm.munmap(addr, 5 * PAGE_SIZE);
        assert!(result);
        assert_eq!(vm.mapped_page_count(), 0);

        // Verify all pages are unmapped
        for i in 0..5 {
            let vpage = VirtualMemory::vaddr_to_page(addr + (i * PAGE_SIZE) as u64);
            assert!(!vm.is_mapped(vpage));
        }
    }

    #[test]
    fn test_munmap_partial_range() {
        let mut vm = VirtualMemory::new();

        // Allocate 5 pages
        let addr = vm.mmap(5 * PAGE_SIZE).unwrap();
        assert_eq!(vm.mapped_page_count(), 5);

        // Free middle 3 pages
        let middle_addr = addr + PAGE_SIZE as u64;
        let result = vm.munmap(middle_addr, 3 * PAGE_SIZE);
        assert!(result);
        assert_eq!(vm.mapped_page_count(), 2);

        // First and last pages should still be mapped
        let vpage0 = VirtualMemory::vaddr_to_page(addr);
        let vpage4 = VirtualMemory::vaddr_to_page(addr + (4 * PAGE_SIZE) as u64);
        assert!(vm.is_mapped(vpage0));
        assert!(vm.is_mapped(vpage4));

        // Middle pages should be unmapped
        for i in 1..4 {
            let vpage = VirtualMemory::vaddr_to_page(addr + (i * PAGE_SIZE) as u64);
            assert!(!vm.is_mapped(vpage));
        }
    }

    #[test]
    fn test_munmap_zero_size() {
        let mut vm = VirtualMemory::new();

        // Allocate a page
        let addr = vm.mmap(PAGE_SIZE).unwrap();

        // Zero-size munmap should succeed and do nothing
        let result = vm.munmap(addr, 0);
        assert!(result);
        assert_eq!(vm.mapped_page_count(), 1);
    }

    #[test]
    fn test_munmap_unmapped_fails() {
        let mut vm = VirtualMemory::new();
        let layout = vm.layout().clone();

        // Try to unmap unmapped pages
        let result = vm.munmap(layout.heap_start, PAGE_SIZE);
        assert!(!result); // Should fail
    }

    #[test]
    fn test_munmap_partially_mapped_fails() {
        let mut vm = VirtualMemory::new();

        // Allocate 2 pages
        let addr = vm.mmap(2 * PAGE_SIZE).unwrap();

        // Try to unmap 3 pages (one unmapped)
        let result = vm.munmap(addr, 3 * PAGE_SIZE);
        assert!(!result); // Should fail

        // Original pages should still be mapped (all-or-nothing)
        assert_eq!(vm.mapped_page_count(), 2);
    }

    #[test]
    fn test_is_range_mapped() {
        let mut vm = VirtualMemory::new();

        // Allocate 3 pages
        let addr = vm.mmap(3 * PAGE_SIZE).unwrap();

        // Range should be fully mapped
        assert!(vm.is_range_mapped(addr, 3 * PAGE_SIZE));
        assert!(vm.is_range_mapped(addr, PAGE_SIZE));
        assert!(vm.is_range_mapped(addr + PAGE_SIZE as u64, PAGE_SIZE));

        // Beyond allocation should not be mapped
        assert!(!vm.is_range_mapped(addr, 4 * PAGE_SIZE));
        assert!(!vm.is_range_mapped(addr + (3 * PAGE_SIZE) as u64, PAGE_SIZE));

        // Zero-size range is always "mapped"
        assert!(vm.is_range_mapped(addr, 0));
        assert!(vm.is_range_mapped(0, 0));
    }

    #[test]
    fn test_mmap_munmap_cycle() {
        let mut vm = VirtualMemory::new();

        // Allocate, free, allocate again
        let addr1 = vm.mmap(PAGE_SIZE).unwrap();
        vm.munmap(addr1, PAGE_SIZE);
        let addr2 = vm.mmap(PAGE_SIZE).unwrap();

        // Note: Sequential allocator doesn't reuse freed pages
        // Second allocation comes after first
        assert!(addr2 > addr1);
    }

    #[test]
    fn test_page_permissions() {
        let mut vm = VirtualMemory::new();

        // Map page with read-only permissions
        let vpage = 0;
        vm.map_page(vpage, 100, PagePermissions::read_only());

        // Check permissions
        let perms = vm.get_page_permissions(vpage).unwrap();
        assert!(perms.allows_read());
        assert!(!perms.allows_write());
        assert!(!perms.allows_execute());
    }

    #[test]
    fn test_permission_checking() {
        let mut vm = VirtualMemory::new();
        let vpage = 0;

        // Map with read-only permissions
        vm.map_page(vpage, 100, PagePermissions::read_only());

        // Read should succeed
        assert!(vm.check_permission(vpage, MemoryAccess::Read));
        // Write should fail
        assert!(!vm.check_permission(vpage, MemoryAccess::Write));
        // Execute should fail
        assert!(!vm.check_permission(vpage, MemoryAccess::Execute));
    }

    #[test]
    fn test_set_page_permissions() {
        let mut vm = VirtualMemory::new();
        let vpage = 0;

        // Map with read-write permissions
        vm.map_page(vpage, 100, PagePermissions::read_write());

        // Verify initial permissions
        assert!(vm.check_permission(vpage, MemoryAccess::Read));
        assert!(vm.check_permission(vpage, MemoryAccess::Write));
        assert!(!vm.check_permission(vpage, MemoryAccess::Execute));

        // Change to read-only
        assert!(vm.set_page_permissions(vpage, PagePermissions::read_only()));

        // Verify updated permissions
        assert!(vm.check_permission(vpage, MemoryAccess::Read));
        assert!(!vm.check_permission(vpage, MemoryAccess::Write));
        assert!(!vm.check_permission(vpage, MemoryAccess::Execute));
    }

    #[test]
    fn test_set_permissions_unmapped_fails() {
        let mut vm = VirtualMemory::new();
        let vpage = 0;

        // Try to set permissions on unmapped page
        assert!(!vm.set_page_permissions(vpage, PagePermissions::read_only()));
    }

    #[test]
    fn test_read_only_pages() {
        let mut vm = VirtualMemory::new();
        let vpage = 0;

        // Map with read-only permissions
        vm.map_page(vpage, 100, PagePermissions::read_only());

        // Read should be allowed
        assert!(vm.check_permission(vpage, MemoryAccess::Read));
        // Write should be denied
        assert!(!vm.check_permission(vpage, MemoryAccess::Write));
        // Execute should be denied
        assert!(!vm.check_permission(vpage, MemoryAccess::Execute));

        // Physical page should still be mapped
        assert_eq!(vm.get_physical_page(vpage), Some(100));
    }

    #[test]
    fn test_execute_only_pages() {
        let mut vm = VirtualMemory::new();
        let vpage = 0;

        // Map with read-execute permissions (code pages)
        vm.map_page(vpage, 100, PagePermissions::read_execute());

        // Read should be allowed
        assert!(vm.check_permission(vpage, MemoryAccess::Read));
        // Write should be denied
        assert!(!vm.check_permission(vpage, MemoryAccess::Write));
        // Execute should be allowed
        assert!(vm.check_permission(vpage, MemoryAccess::Execute));
    }

    #[test]
    fn test_no_permissions() {
        let mut vm = VirtualMemory::new();
        let vpage = 0;

        // Map with no permissions
        vm.map_page(vpage, 100, PagePermissions::none());

        // All accesses should be denied
        assert!(!vm.check_permission(vpage, MemoryAccess::Read));
        assert!(!vm.check_permission(vpage, MemoryAccess::Write));
        assert!(!vm.check_permission(vpage, MemoryAccess::Execute));

        // Page should still be mapped
        assert!(vm.is_mapped(vpage));
    }

    #[test]
    fn test_unmapped_page_permissions() {
        let vm = VirtualMemory::new();
        let vpage = 0;

        // Unmapped page should deny all accesses
        assert!(!vm.check_permission(vpage, MemoryAccess::Read));
        assert!(!vm.check_permission(vpage, MemoryAccess::Write));
        assert!(!vm.check_permission(vpage, MemoryAccess::Execute));
        assert_eq!(vm.get_page_permissions(vpage), None);
    }

    #[test]
    fn test_check_address_permission() {
        let mut vm = VirtualMemory::new();
        let layout = vm.layout().clone();

        // Map heap page with read-write
        let heap_addr = layout.heap_start;
        let vpage = VirtualMemory::vaddr_to_page(heap_addr);
        vm.map_page(vpage, 100, PagePermissions::read_write());

        // Check address permissions
        assert!(vm.check_address_permission(heap_addr, MemoryAccess::Read));
        assert!(vm.check_address_permission(heap_addr, MemoryAccess::Write));
        assert!(!vm.check_address_permission(heap_addr, MemoryAccess::Execute));

        // Check offset within page
        let offset_addr = heap_addr + 1024;
        assert!(vm.check_address_permission(offset_addr, MemoryAccess::Read));
        assert!(vm.check_address_permission(offset_addr, MemoryAccess::Write));
    }

    #[test]
    fn test_mmap_with_permissions() {
        let mut vm = VirtualMemory::new();

        // Allocate with read-only permissions
        let addr = vm
            .mmap_with_permissions(PAGE_SIZE, PagePermissions::read_only())
            .unwrap();
        let vpage = VirtualMemory::vaddr_to_page(addr);

        // Verify permissions
        assert!(vm.check_permission(vpage, MemoryAccess::Read));
        assert!(!vm.check_permission(vpage, MemoryAccess::Write));
        assert!(!vm.check_permission(vpage, MemoryAccess::Execute));
    }

    #[test]
    fn test_mmap_default_permissions() {
        let mut vm = VirtualMemory::new();

        // Default mmap should be read-write
        let addr = vm.mmap(PAGE_SIZE).unwrap();
        let vpage = VirtualMemory::vaddr_to_page(addr);

        // Verify read-write permissions
        assert!(vm.check_permission(vpage, MemoryAccess::Read));
        assert!(vm.check_permission(vpage, MemoryAccess::Write));
        assert!(!vm.check_permission(vpage, MemoryAccess::Execute));
    }

    #[test]
    fn test_page_permissions_constants() {
        // Test read-only
        let ro = PagePermissions::read_only();
        assert!(ro.allows_read());
        assert!(!ro.allows_write());
        assert!(!ro.allows_execute());

        // Test read-write
        let rw = PagePermissions::read_write();
        assert!(rw.allows_read());
        assert!(rw.allows_write());
        assert!(!rw.allows_execute());

        // Test read-execute
        let rx = PagePermissions::read_execute();
        assert!(rx.allows_read());
        assert!(!rx.allows_write());
        assert!(rx.allows_execute());

        // Test read-write-execute
        let rwx = PagePermissions::read_write_execute();
        assert!(rwx.allows_read());
        assert!(rwx.allows_write());
        assert!(rwx.allows_execute());

        // Test none
        let none = PagePermissions::none();
        assert!(!none.allows_read());
        assert!(!none.allows_write());
        assert!(!none.allows_execute());
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: Address translation is deterministic
            #[test]
            fn proptest_address_translation_deterministic(
                vpage in 0..1000u32,
                ppage in 0..10000u32,
                offset in 0..PAGE_SIZE as u64,
            ) {
                let mut vm = VirtualMemory::new();
                let perms = PagePermissions::read_write();
                vm.map_page(vpage, ppage, perms);

                let vaddr = (vpage as u64) * PAGE_SIZE as u64 + offset;
                let paddr1 = vm.translate(vaddr);
                let paddr2 = vm.translate(vaddr);

                prop_assert_eq!(paddr1, paddr2);
                prop_assert_eq!(paddr1, Some((ppage as u64) * PAGE_SIZE as u64 + offset));
            }

            /// Property: Page allocation is monotonic
            #[test]
            fn proptest_page_allocation_monotonic(
                num_pages in 1..1000usize,
            ) {
                let mut vm = VirtualMemory::new();
                let mut ppages = Vec::new();
                let perms = PagePermissions::read_write();

                for i in 0..num_pages {
                    let ppage = vm.allocate_page(i as VirtualPage, perms);
                    ppages.push(ppage);
                }

                // All physical pages should be unique and increasing
                for i in 1..ppages.len() {
                    prop_assert!(ppages[i] > ppages[i - 1]);
                }
            }

            /// Property: Map/unmap is consistent
            #[test]
            fn proptest_map_unmap_consistent(
                operations in prop::collection::vec((0..3usize, 0..100u32, 0..1000u32), 0..100),
            ) {
                let mut vm = VirtualMemory::new();
                let mut expected = std::collections::HashMap::new();
                let perms = PagePermissions::read_write();

                for (op, vpage, ppage) in operations {
                    match op {
                        0 => {
                            // Map
                            vm.map_page(vpage, ppage, perms);
                            expected.insert(vpage, ppage);
                        }
                        1 => {
                            // Unmap
                            let result = vm.unmap_page(vpage);
                            let expected_result = expected.remove(&vpage).map(|ppage| PageTableEntry::new(ppage, perms));
                            prop_assert_eq!(result, expected_result);
                        }
                        _ => {
                            // Lookup
                            let result = vm.get_physical_page(vpage);
                            let expected_result = expected.get(&vpage).copied();
                            prop_assert_eq!(result, expected_result);
                        }
                    }
                }
            }

            /// Property: Translation preserves page boundaries
            #[test]
            fn proptest_translation_preserves_page_boundaries(
                vpage in 0..1000u32,
                ppage in 0..10000u32,
            ) {
                let mut vm = VirtualMemory::new();
                let perms = PagePermissions::read_write();
                vm.map_page(vpage, ppage, perms);

                // First byte of page
                let vaddr_start = (vpage as u64) * PAGE_SIZE as u64;
                let paddr_start = vm.translate(vaddr_start).unwrap();
                prop_assert_eq!(paddr_start, (ppage as u64) * PAGE_SIZE as u64);

                // Last byte of page
                let vaddr_end = (vpage as u64) * PAGE_SIZE as u64 + (PAGE_SIZE - 1) as u64;
                let paddr_end = vm.translate(vaddr_end).unwrap();
                prop_assert_eq!(paddr_end, (ppage as u64) * PAGE_SIZE as u64 + (PAGE_SIZE - 1) as u64);
            }

            /// Property: Memory layout regions are disjoint
            #[test]
            fn proptest_memory_regions_disjoint(
                _seed in 0..100u64,
            ) {
                let layout = MemoryLayout::default();

                // Check all regions don't overlap
                let code_end = layout.code_start + layout.code_size as u64;
                let data_end = layout.data_start + layout.data_size as u64;
                let heap_end = layout.heap_start + layout.heap_size as u64;
                let stack_end = layout.stack_start - layout.stack_size as u64;

                prop_assert!(code_end <= layout.data_start);
                prop_assert!(data_end <= layout.heap_start);
                prop_assert!(heap_end <= stack_end);
            }

            /// Property: mmap always returns addresses in heap region
            #[test]
            fn proptest_mmap_returns_heap_addresses(
                size in 1..1024usize,
            ) {
                let mut vm = VirtualMemory::new();
                let layout = vm.layout().clone();

                if let Some(addr) = vm.mmap(size) {
                    prop_assert!(addr >= layout.heap_start);
                    prop_assert!(addr < layout.heap_start + layout.heap_size as u64);
                }
            }

            /// Property: mmap allocates correct number of pages
            #[test]
            fn proptest_mmap_allocates_correct_pages(
                size in 1..10000usize,
            ) {
                let mut vm = VirtualMemory::new();
                let initial_count = vm.mapped_page_count();

                if let Some(_addr) = vm.mmap(size) {
                    let expected_pages = size.div_ceil(PAGE_SIZE);
                    let actual_pages = vm.mapped_page_count() - initial_count;
                    prop_assert_eq!(actual_pages, expected_pages);
                }
            }

            /// Property: mmap returns sequential addresses
            #[test]
            fn proptest_mmap_sequential(
                sizes in prop::collection::vec(1..1000usize, 1..10),
            ) {
                let mut vm = VirtualMemory::new();
                let mut prev_addr: Option<u64> = None;
                let mut prev_size: usize = 0;

                for size in sizes {
                    if let Some(addr) = vm.mmap(size) {
                        if let Some(prev) = prev_addr {
                            let prev_pages = prev_size.div_ceil(PAGE_SIZE);
                            let expected_addr = prev + (prev_pages * PAGE_SIZE) as u64;
                            prop_assert_eq!(addr, expected_addr);
                        }
                        prev_addr = Some(addr);
                        prev_size = size;
                    } else {
                        // Out of memory - stop
                        break;
                    }
                }
            }

            /// Property: munmap all-or-nothing semantics
            #[test]
            fn proptest_munmap_all_or_nothing(
                alloc_size in 1..10000usize,
                unmap_size in 1..20000usize,
            ) {
                let mut vm = VirtualMemory::new();

                // Allocate
                if let Some(addr) = vm.mmap(alloc_size) {
                    let count_before = vm.mapped_page_count();

                    // Try to unmap
                    let result = vm.munmap(addr, unmap_size);

                    if result {
                        // If successful, pages should be unmapped
                        let unmap_pages = unmap_size.div_ceil(PAGE_SIZE);
                        prop_assert!(vm.mapped_page_count() <= count_before);
                        prop_assert!(vm.mapped_page_count() >= count_before - unmap_pages);
                    } else {
                        // If failed, count should be unchanged
                        prop_assert_eq!(vm.mapped_page_count(), count_before);
                    }
                }
            }

            /// Property: munmap of mapped range always succeeds
            #[test]
            fn proptest_munmap_mapped_succeeds(
                size in 1..10000usize,
            ) {
                let mut vm = VirtualMemory::new();

                // Allocate
                if let Some(addr) = vm.mmap(size) {
                    // Unmapping the exact range should always succeed
                    let result = vm.munmap(addr, size);
                    prop_assert!(result);

                    // Pages should be unmapped
                    let num_pages = size.div_ceil(PAGE_SIZE);
                    for i in 0..num_pages {
                        let vpage = VirtualMemory::vaddr_to_page(addr + (i * PAGE_SIZE) as u64);
                        prop_assert!(!vm.is_mapped(vpage));
                    }
                }
            }

            /// Property: is_range_mapped is consistent with individual page checks
            #[test]
            fn proptest_is_range_mapped_consistent(
                size in 1..10000usize,
            ) {
                let mut vm = VirtualMemory::new();

                if let Some(addr) = vm.mmap(size) {
                    // Range should be mapped
                    prop_assert!(vm.is_range_mapped(addr, size));

                    // Each individual page should be mapped
                    let num_pages = size.div_ceil(PAGE_SIZE);
                    for i in 0..num_pages {
                        let vpage = VirtualMemory::vaddr_to_page(addr + (i * PAGE_SIZE) as u64);
                        prop_assert!(vm.is_mapped(vpage));
                    }
                }
            }

            /// Property: mmap/munmap roundtrip preserves page count
            #[test]
            fn proptest_mmap_munmap_roundtrip(
                operations in prop::collection::vec((1..1000usize, 0..2usize), 1..20),
            ) {
                let mut vm = VirtualMemory::new();
                let mut allocated = Vec::new();

                for (size, op) in operations {
                    match op {
                        0 => {
                            // mmap
                            if let Some(addr) = vm.mmap(size) {
                                allocated.push((addr, size));
                            }
                        }
                        _ => {
                            // munmap
                            if let Some((addr, size)) = allocated.pop() {
                                vm.munmap(addr, size);
                            }
                        }
                    }
                }

                // Page count should match remaining allocations
                let expected_pages: usize = allocated
                    .iter()
                    .map(|(_, size)| size.div_ceil(PAGE_SIZE))
                    .sum();
                prop_assert_eq!(vm.mapped_page_count(), expected_pages);
            }

            /// Property: mmap never returns same address twice (without munmap)
            #[test]
            fn proptest_mmap_unique_addresses(
                sizes in prop::collection::vec(1..1000usize, 1..20),
            ) {
                let mut vm = VirtualMemory::new();
                let mut addresses = std::collections::HashSet::new();

                for size in sizes {
                    if let Some(addr) = vm.mmap(size) {
                        // Address should be unique
                        prop_assert!(!addresses.contains(&addr));
                        addresses.insert(addr);
                    } else {
                        // Out of memory - ok to stop
                        break;
                    }
                }
            }

            /// Property: Zero-size operations are safe
            #[test]
            fn proptest_zero_size_safe(
                _seed in 0..100u32,
            ) {
                let mut vm = VirtualMemory::new();

                // Zero-size mmap returns None
                prop_assert_eq!(vm.mmap(0), None);

                // Zero-size munmap succeeds
                prop_assert!(vm.munmap(0, 0));
                prop_assert!(vm.munmap(vm.layout().heap_start, 0));

                // Zero-size is_range_mapped returns true
                prop_assert!(vm.is_range_mapped(0, 0));
                prop_assert!(vm.is_range_mapped(vm.layout().heap_start, 0));
            }

            /// Property: Permission checking is consistent
            #[test]
            fn proptest_permission_checking(
                vpage in 0..1000u32,
                read in proptest::bool::ANY,
                write in proptest::bool::ANY,
                execute in proptest::bool::ANY,
            ) {
                let mut vm = VirtualMemory::new();
                let perms = PagePermissions { read, write, execute };
                vm.map_page(vpage, 100, perms);

                // Check consistency
                prop_assert_eq!(vm.check_permission(vpage, MemoryAccess::Read), read);
                prop_assert_eq!(vm.check_permission(vpage, MemoryAccess::Write), write);
                prop_assert_eq!(vm.check_permission(vpage, MemoryAccess::Execute), execute);

                // Verify permissions match
                let retrieved_perms = vm.get_page_permissions(vpage).unwrap();
                prop_assert_eq!(retrieved_perms.allows_read(), read);
                prop_assert_eq!(retrieved_perms.allows_write(), write);
                prop_assert_eq!(retrieved_perms.allows_execute(), execute);
            }

            /// Property: Set permissions always succeeds on mapped pages
            #[test]
            fn proptest_set_permissions_mapped(
                vpage in 0..1000u32,
                new_read in proptest::bool::ANY,
                new_write in proptest::bool::ANY,
                new_execute in proptest::bool::ANY,
            ) {
                let mut vm = VirtualMemory::new();
                // Map with initial permissions
                vm.map_page(vpage, 100, PagePermissions::read_write());

                // Set new permissions
                let new_perms = PagePermissions { read: new_read, write: new_write, execute: new_execute };
                let result = vm.set_page_permissions(vpage, new_perms);
                prop_assert!(result);

                // Verify new permissions
                let retrieved_perms = vm.get_page_permissions(vpage).unwrap();
                prop_assert_eq!(retrieved_perms, new_perms);
            }

            /// Property: Set permissions always fails on unmapped pages
            #[test]
            fn proptest_set_permissions_unmapped(
                vpage in 0..1000u32,
            ) {
                let vm = VirtualMemory::new();
                // Try to set permissions on unmapped page
                let result = vm.clone().set_page_permissions(vpage, PagePermissions::read_only());
                prop_assert!(!result);
            }

            /// Property: Unmapped pages deny all access
            #[test]
            fn proptest_unmapped_denies_access(
                vpage in 0..1000u32,
            ) {
                let vm = VirtualMemory::new();
                // All accesses to unmapped pages should be denied
                prop_assert!(!vm.check_permission(vpage, MemoryAccess::Read));
                prop_assert!(!vm.check_permission(vpage, MemoryAccess::Write));
                prop_assert!(!vm.check_permission(vpage, MemoryAccess::Execute));
            }

            /// Property: mmap_with_permissions sets correct permissions
            #[test]
            fn proptest_mmap_with_permissions(
                size in 1..10000usize,
                read in proptest::bool::ANY,
                write in proptest::bool::ANY,
                execute in proptest::bool::ANY,
            ) {
                let mut vm = VirtualMemory::new();
                let perms = PagePermissions { read, write, execute };

                if let Some(addr) = vm.mmap_with_permissions(size, perms) {
                    // All allocated pages should have the specified permissions
                    let num_pages = size.div_ceil(PAGE_SIZE);
                    for i in 0..num_pages {
                        let vpage = VirtualMemory::vaddr_to_page(addr + (i * PAGE_SIZE) as u64);
                        let page_perms = vm.get_page_permissions(vpage).unwrap();
                        prop_assert_eq!(page_perms, perms);
                    }
                }
            }

            /// Property: Address permission checking matches page checking
            #[test]
            fn proptest_address_permission_consistent(
                vpage in 0..1000u32,
                offset in 0..PAGE_SIZE as u64,
                read in proptest::bool::ANY,
                write in proptest::bool::ANY,
                execute in proptest::bool::ANY,
            ) {
                let mut vm = VirtualMemory::new();
                let perms = PagePermissions { read, write, execute };
                vm.map_page(vpage, 100, perms);

                let vaddr = (vpage as u64) * PAGE_SIZE as u64 + offset;

                // Address check should match page check
                prop_assert_eq!(
                    vm.check_address_permission(vaddr, MemoryAccess::Read),
                    vm.check_permission(vpage, MemoryAccess::Read)
                );
                prop_assert_eq!(
                    vm.check_address_permission(vaddr, MemoryAccess::Write),
                    vm.check_permission(vpage, MemoryAccess::Write)
                );
                prop_assert_eq!(
                    vm.check_address_permission(vaddr, MemoryAccess::Execute),
                    vm.check_permission(vpage, MemoryAccess::Execute)
                );
            }
        }
    }
}
