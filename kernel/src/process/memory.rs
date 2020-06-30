use alloc::vec::Vec;
use crate::memory::{
  address::VirtualAddress,
  heap::INITIAL_HEAP_SIZE,
  physical,
  virt::{
    page_directory::{AlternatePageDirectory, self},
    page_table::{PageTable, PageTableReference},
    region::{
      ExpansionDirection,
      MemoryRegionType,
      Permissions,
      VirtualMemoryRegion,
    },
    STACK_START,
  },
};
use super::process_state::ProcessState;

pub struct MemoryRegions {
  pub kernel_heap_region: VirtualMemoryRegion,
  pub kernel_stack_region: VirtualMemoryRegion,
  pub heap_region: VirtualMemoryRegion,
  pub stack_region: VirtualMemoryRegion,
  pub execution_regions: Vec<VirtualMemoryRegion>,
}

impl MemoryRegions {
  pub fn initial(heap_start: VirtualAddress) -> MemoryRegions {
    let mut execution_regions = Vec::with_capacity(2);
    execution_regions.push(VirtualMemoryRegion::new(
      VirtualAddress::new(0),
      0x400000,
      MemoryRegionType::Anonymous(ExpansionDirection::None),
      Permissions::CopyOnWrite,
    ));
    execution_regions.push(VirtualMemoryRegion::new(
      VirtualAddress::new(0xc0000000),
      0x400000,
      MemoryRegionType::Anonymous(ExpansionDirection::None),
      Permissions::CopyOnWrite,
    ));

    MemoryRegions {
      kernel_heap_region: VirtualMemoryRegion::new(
        heap_start,
        INITIAL_HEAP_SIZE * 0x1000,
        MemoryRegionType::Anonymous(ExpansionDirection::After),
        Permissions::ReadOnly,
      ),

      kernel_stack_region: VirtualMemoryRegion::new(
        STACK_START,
        0x2000,
        MemoryRegionType::Anonymous(ExpansionDirection::Before),
        Permissions::ReadWrite,
      ),

      heap_region: VirtualMemoryRegion::empty(),

      stack_region: VirtualMemoryRegion::new(
        VirtualAddress::new(0xc0000000 - 0x2000),
        0x2000,
        MemoryRegionType::Anonymous(ExpansionDirection::Before),
        Permissions::ReadWrite,
      ),

      execution_regions,
    }
  }

  /**
   * Duplicate the memory range for a forked process.
   * The kernel uses a copy-on-write scheme
   */
  pub fn fork(&self) -> MemoryRegions {
    let kernel_heap_region = self.kernel_heap_region.clone();
    let kernel_stack_region = self.kernel_stack_region.copy_with_permissions(Permissions::ReadWrite);
    let heap_region = self.heap_region.copy_for_new_process();
    let stack_region = self.stack_region.copy_for_new_process();
    let execution_regions = self.execution_regions
      .iter()
      .map(|&range| range.copy_for_new_process())
      .collect();

    MemoryRegions {
      kernel_heap_region,
      kernel_stack_region,
      heap_region,
      stack_region,
      execution_regions,
    }
  }

  pub fn get_range_containing_address(&self, addr: VirtualAddress) -> Option<VirtualMemoryRegion> {
    let kernel_heap = self.kernel_heap_region;
    if kernel_heap.contains_address(addr) {
      return Some(kernel_heap.clone());
    }

    let kernel_stack = self.kernel_stack_region;
    if kernel_stack.contains_address(addr) {
      return Some(kernel_stack.clone());
    }

    let heap = self.heap_region;
    if heap.contains_address(addr) {
      return Some(heap.clone());
    }

    let stack = self.stack_region;
    if stack.contains_address(addr) {
      return Some(stack.clone());
    }

    for region in self.execution_regions.iter() {
      if region.contains_address(addr) {
        return Some(region.clone());
      }
    }

    None
  }
}

impl ProcessState {
  pub fn fork_page_directory(&self) -> PageTableReference {
    let temp_page_address = page_directory::get_temporary_page_address();

    // Create the top page, which will contain the temp page and kernel stack
    let top_page = physical::allocate_frame().unwrap();
    page_directory::map_frame_to_temporary_page(top_page);
    PageTable::at_address(temp_page_address).zero();

    // Create the new page directory
    let directory_frame = physical::allocate_frame().unwrap();
    page_directory::map_frame_to_temporary_page(directory_frame);
    let directory_table = PageTable::at_address(temp_page_address);
    directory_table.zero();

    // Map the directory table to itself
    directory_table.get_mut(1023).set_address(directory_frame.get_address());
    directory_table.get_mut(1023).set_present();
    // Map the top page
    directory_table.get_mut(1022).set_address(top_page.get_address());
    directory_table.get_mut(1022).set_present();

    // Map each of the ranges
    let new_page_dir = AlternatePageDirectory::new(directory_frame.get_address());
    {
      let regions = self.get_memory_regions().write();
      new_page_dir.map_region(regions.kernel_stack_region);
      new_page_dir.map_region(regions.kernel_heap_region);
      new_page_dir.map_region(regions.stack_region);
      new_page_dir.map_region(regions.heap_region);
      
      for region in regions.execution_regions.iter() {
        new_page_dir.map_region(*region);
      }
    }

    PageTableReference::new(directory_frame.get_address())
  }
}
