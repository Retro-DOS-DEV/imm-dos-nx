use alloc::vec::Vec;
use crate::files::handle::LocalHandle;
use crate::memory::{
  address::{PhysicalAddress, VirtualAddress},
  heap::INITIAL_HEAP_SIZE,
  physical::{self, frame_range::FrameRange},
  virt::{
    page_directory::{AlternatePageDirectory, CurrentPageDirectory, self},
    page_table::{PageTable, PageTableReference},
    region::{
      ExpansionDirection,
      MemoryRegionType,
      Permissions,
      VirtualMemoryRegion,
    },
  },
};
use spin::RwLock;
use super::process_state::ProcessState;

/// The kernel stack extends from 0xffbf0000 to 0xffbfefff
pub const STACK_START: VirtualAddress = VirtualAddress::new(0xffbf0000);
pub const STACK_SIZE: usize = 0xffbff000 - STACK_START.as_usize();

static KERNEL_HEAP: RwLock<VirtualMemoryRegion> =
  RwLock::new(
    VirtualMemoryRegion::new(
      VirtualAddress::new(0),
      INITIAL_HEAP_SIZE * 0x1000,
      MemoryRegionType::Anonymous(ExpansionDirection::After),
      Permissions::ReadOnly,
    ),
  );

/// Store custom memmap regions shared between all processes in kernel space
static KERNEL_MEMMAP: RwLock<Vec<VirtualMemoryRegion>> = RwLock::new(Vec::new());

/// Increase the kernel heap range, returning the new range size
pub fn expand_kernel_heap(min_space_needed: usize) -> usize {
  let mut frames_needed = min_space_needed / 0x1000;
  if frames_needed < INITIAL_HEAP_SIZE {
    frames_needed = INITIAL_HEAP_SIZE;
  }
  let mut heap = KERNEL_HEAP.write();
  heap.expand(frames_needed)
}

pub struct MemoryRegions {
  pub kernel_stack_region: VirtualMemoryRegion,
  pub kernel_exec_region: VirtualMemoryRegion,
  pub heap_region: VirtualMemoryRegion,
  pub stack_region: VirtualMemoryRegion,
  pub execution_regions: Vec<VirtualMemoryRegion>,
}

impl MemoryRegions {
  pub fn initial(heap_start: VirtualAddress) -> MemoryRegions {
    {
      let mut heap = KERNEL_HEAP.write();
      heap.set_starting_address(heap_start);
    }
    let mut execution_regions = Vec::with_capacity(1);
    execution_regions.push(VirtualMemoryRegion::new(
      VirtualAddress::new(0),
      0x400000,
      MemoryRegionType::Anonymous(ExpansionDirection::None),
      Permissions::CopyOnWrite,
    ));

    MemoryRegions {
      kernel_stack_region: VirtualMemoryRegion::new(
        STACK_START,
        STACK_SIZE,
        MemoryRegionType::Anonymous(ExpansionDirection::None),
        Permissions::ReadWrite,
      ),

      kernel_exec_region: VirtualMemoryRegion::new(
        VirtualAddress::new(0xc0000000),
        0x400000,
        MemoryRegionType::Anonymous(ExpansionDirection::None),
        Permissions::CopyOnWrite,
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
    let kernel_stack_region = self.kernel_stack_region.copy_with_permissions(Permissions::ReadWrite);
    let kernel_exec_region = self.kernel_exec_region.copy_for_new_process();
    let heap_region = self.heap_region.copy_for_new_process();
    let stack_region = self.stack_region.copy_for_new_process();
    let execution_regions = self.execution_regions
      .iter()
      .map(|&range| range.copy_for_new_process())
      .collect();

    MemoryRegions {
      kernel_stack_region,
      kernel_exec_region,
      heap_region,
      stack_region,
      execution_regions,
    }
  }

  pub fn get_range_containing_address(&self, addr: VirtualAddress) -> Option<VirtualMemoryRegion> {
    {
      let kernel_heap = KERNEL_HEAP.read();
      if kernel_heap.contains_address(addr) {
        return Some(kernel_heap.clone());
      }
    }

    let kernel_stack = self.kernel_stack_region;
    if kernel_stack.contains_address(addr) {
      return Some(kernel_stack.clone());
    }

    let kernel_exec = self.kernel_exec_region;
    if kernel_exec.contains_address(addr) {
      return Some(kernel_exec.clone());
    }

    {
      let kernel_memmap = KERNEL_MEMMAP.read();
      for region in kernel_memmap.iter() {
        if region.contains_address(addr) {
          return Some(region.clone());
        }
      }
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
      let kernel_heap = *KERNEL_HEAP.read();
      new_page_dir.map_region(kernel_heap);
    }
    {
      let regions = self.get_memory_regions().write();
      new_page_dir.map_region(regions.kernel_stack_region);
      new_page_dir.map_region(regions.kernel_exec_region);
      new_page_dir.map_region(regions.stack_region);
      new_page_dir.map_region(regions.heap_region);

      for region in regions.execution_regions.iter() {
        new_page_dir.map_region(*region);
      }
    }

    PageTableReference::new(directory_frame.get_address())
  }

  pub fn unmap_all(&self) {
    let mut regions = self.get_memory_regions().write();
    let current_pagedir = CurrentPageDirectory::get();
    while regions.execution_regions.len() > 0 {
      if let Some(region) = regions.execution_regions.pop() {
        current_pagedir.unmap_region(region);
      }
    }
  }

  /**
   * Map a memory region to the contents of a file. When pages are accessed,
   * they'll be fetched from the filesystem, 0x1000 bytes at a time.
   */
  pub fn mmap(&self, start: VirtualAddress, length: usize, drive_number: usize, handle: LocalHandle) {
    let mut region_length = length;
    if length & 0xfff != 0 {
      region_length = (length + 0x1000) & 0xfffff000;
    }
    let region = VirtualMemoryRegion::new(
      start,
      region_length,
      MemoryRegionType::MemMapped(drive_number, handle, length),
      Permissions::ReadWrite,
    );
    self.get_memory_regions().write().execution_regions.push(region);
  }

  /**
   * Create a mapping to an "anonymous" range, which just pulls in free physical
   * memory pages as needed.
   */
  pub fn anonymous_map(&self, start: VirtualAddress, length: usize) {
    let mut region_length = length;
    if length & 0xfff != 0 {
      region_length = (length + 0x1000) & 0xfffff000;
    }
    let region = VirtualMemoryRegion::new(
      start,
      region_length,
      MemoryRegionType::Anonymous(ExpansionDirection::None),
      Permissions::ReadWrite,
    );
    self.get_memory_regions().write().execution_regions.push(region);
  }

  /// Map a virtual address to a contiguous region of memory suitable for DMA
  /// transfers
  fn mmap_dma_region(&self, virt: VirtualAddress, length: usize) -> (PhysicalAddress, VirtualMemoryRegion) {
    // Find an appropriately sized physical memory region first
    let mut frame_count = length >> 12;
    if length & 0xfff > 0 {
      frame_count += 1;
    }
    // This should be updated to ensure the memory is in the first 16MiB
    let range = physical::allocate_frames(frame_count).unwrap();
    let phys = range.get_starting_address();

    let mut region_length = length;
    if length & 0xfff != 0 {
      region_length = (length + 0x1000) & 0xfffff000;
    }
    let range = FrameRange::new(phys.as_usize(), length);
    let region = VirtualMemoryRegion::new(
      virt,
      region_length,
      MemoryRegionType::DMA(range),
      Permissions::ReadWrite,
    );
    (phys, region)
  }

  pub fn mmap_dma(&self, virt: VirtualAddress, length: usize) -> PhysicalAddress {
    let (phys, region) = self.mmap_dma_region(virt, length);
    self.get_memory_regions().write().execution_regions.push(region);
    phys
  }

  pub fn kernel_mmap_dma(&self, length: usize) -> (PhysicalAddress, VirtualAddress) {
    let mut kernel_memmap = KERNEL_MEMMAP.write();
    // Find a free space below the stack
    let mut last_occupied = STACK_START.as_usize();
    for region in kernel_memmap.iter() {
      let region_start = region.get_starting_address_as_usize();
      if region_start < last_occupied {
        last_occupied = region_start;
      }
    }
    let new_region_start = VirtualAddress::new((last_occupied - length) & 0xfffff000);
    let (phys, region) = self.mmap_dma_region(new_region_start, length);
    kernel_memmap.push(region);
    (phys, new_region_start)
  }
}
