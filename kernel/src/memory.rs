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

/// Virtual memory management with page tables
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualMemory {
    /// Page table (virtual page -> physical page mapping)
    page_table: HashMap<VirtualPage, PhysicalPage>,
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

    /// Map virtual page to physical page
    pub fn map_page(&mut self, vpage: VirtualPage, ppage: PhysicalPage) {
        self.page_table.insert(vpage, ppage);
    }

    /// Unmap virtual page
    pub fn unmap_page(&mut self, vpage: VirtualPage) -> Option<PhysicalPage> {
        self.page_table.remove(&vpage)
    }

    /// Allocate a new physical page and map it to virtual page
    pub fn allocate_page(&mut self, vpage: VirtualPage) -> PhysicalPage {
        let ppage = self.next_physical_page;
        self.next_physical_page += 1;
        self.page_table.insert(vpage, ppage);
        ppage
    }

    /// Get physical page for virtual page
    pub fn get_physical_page(&self, vpage: VirtualPage) -> Option<PhysicalPage> {
        self.page_table.get(&vpage).copied()
    }

    /// Translate virtual address to physical address
    pub fn translate(&self, vaddr: VirtualAddress) -> Option<PhysicalAddress> {
        let vpage = (vaddr / PAGE_SIZE as u64) as VirtualPage;
        let offset = vaddr % PAGE_SIZE as u64;

        self.page_table
            .get(&vpage)
            .map(|&ppage| (ppage as u64) * PAGE_SIZE as u64 + offset)
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
    /// Allocates page-aligned memory in the heap region.
    pub fn mmap(&mut self, size_bytes: usize) -> Option<VirtualAddress> {
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

        // Allocate pages
        for i in 0..num_pages {
            let vaddr = start_addr + (i * PAGE_SIZE) as u64;
            let vpage = Self::vaddr_to_page(vaddr);
            let ppage = self.next_physical_page;
            self.next_physical_page += 1;
            self.page_table.insert(vpage, ppage);
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

        // Map pages
        vm.map_page(0, 100);
        vm.map_page(1, 101);
        vm.map_page(2, 102);

        assert_eq!(vm.mapped_page_count(), 3);

        // Lookup
        assert_eq!(vm.get_physical_page(0), Some(100));
        assert_eq!(vm.get_physical_page(1), Some(101));
        assert_eq!(vm.get_physical_page(2), Some(102));
        assert_eq!(vm.get_physical_page(3), None);

        // Unmap
        assert_eq!(vm.unmap_page(1), Some(101));
        assert_eq!(vm.mapped_page_count(), 2);
        assert_eq!(vm.get_physical_page(1), None);
    }

    #[test]
    fn test_address_translation() {
        let mut vm = VirtualMemory::new();

        // Map virtual page 0 to physical page 100
        vm.map_page(0, 100);

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

        let ppage0 = vm.allocate_page(0);
        assert_eq!(ppage0, 0);
        assert_eq!(vm.get_physical_page(0), Some(0));

        let ppage1 = vm.allocate_page(1);
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
        assert!(!vm.is_mapped(0));

        vm.map_page(0, 100);
        assert!(vm.is_mapped(0));
        assert!(!vm.is_mapped(1));
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
                vm.map_page(vpage, ppage);

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

                for i in 0..num_pages {
                    let ppage = vm.allocate_page(i as VirtualPage);
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

                for (op, vpage, ppage) in operations {
                    match op {
                        0 => {
                            // Map
                            vm.map_page(vpage, ppage);
                            expected.insert(vpage, ppage);
                        }
                        1 => {
                            // Unmap
                            let result = vm.unmap_page(vpage);
                            let expected_result = expected.remove(&vpage);
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
                vm.map_page(vpage, ppage);

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
        }
    }
}
