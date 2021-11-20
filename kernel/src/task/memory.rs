//! Processes are protected and separated by independent memory address spaces.
//! With the exception of shared kernel memory like the heap, each process has
//! a unique set of virtual address mappings.
//! 
//! This 32-bit kernel only supports up to 4GiB of virtual memory. The lower 3
//! are available to the process in userspace, with memory above 3GiB reserved
//! for kernel execution:
//! 
//! 0                           3GiB       4GiB
//! [ Userspace                   |  Kernel  ]
//! 
//! Each of these distinct privileged spaces has to support a similar set of
//! mappings: fixed execution mappings, arbitrary mmaps, an upward-growing heap,
//! a downward-growing stack.
//! The kernel space at the top starts with the actual kernel executable,
//! followed by an upward-growing heap space. At the very top of memory is a
//! fixed-size stack, as well as some scratch pages to allow editing memory not
//! found in the current page mapping. Below this is space that can be used for
//! kernel-mode mmap pages.
//! 
//! 3GiB                                                                    4GiB  
//! [ Kernel Code | Kernel Data | Kernel Heap->      <-mmap | Scratch | Stack ]
//! 
//! The layout of the lower 3GiB is not standardized. Typically, code and data
//! from the process are found at the bottom, followed by a Unix-style `brk`
//! point that can be moved upward with syscalls. This `brk` location allows a
//! contiguous region of memory to be allocated to the process. Calls to mmap
//! will allocated pages near the top, and will move downwards as space is
//! occupied.
//! 
//! 0                                            3GiB
//! [ User Code? | User Data? | brk->       <-mmap ]

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::ops::Range;
use crate::memory::address::{PAGE_SIZE_IN_BYTES, PhysicalAddress, VirtualAddress};
use spin::RwLock;

pub const USER_KERNEL_BARRIER: usize = 0xc0000000;

pub const KERNEL_MMAP_TOP: usize = super::stack::STACKS_TOP - super::stack::MAX_STACK_AREA_SIZE;

/// When executing a process, the kernel builds a mapping between sections of
/// the executable and locations where they should appear in virtual memory.
/// Formats like ELF formally define these mappings. The kernel is designed to
/// be flexible enough to support interpretation of multiple execution formats
/// if desired.
/// The ExecutionSection structure is the core of this mapping. When a frame of
/// virtual memory needs to be initialized, this mapping can determine which
/// bytes to load from the source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSection {
  /// Where in the memory segment does this get copied to?
  pub segment_offset: usize,
  /// Where is this section found in the executable file? If None, no data needs
  /// to be copied from the file, and the section should be backed by zeroes.
  pub executable_offset: Option<usize>,
  /// Section size, in bytes
  pub size: usize,
}

impl ExecutionSection {
  /// Constructs a `Range` based on the parent segment's starting address. This
  /// is useful for determining if the section contains a specific address.
  pub fn as_virtual_range(&self, segment_start: VirtualAddress) -> Range<VirtualAddress> {
    let start = segment_start + self.segment_offset;
    let end = start + self.size;
    start..end
  }

  pub fn clip_to(&self, range: Range<usize>) -> ExecutionSection {
    let (start, offset) = if self.segment_offset < range.start {
      let delta = range.start - self.segment_offset;
      (range.start, self.executable_offset.map(|off| off + delta))
    } else {
      (self.segment_offset, self.executable_offset)
    };
    let end = range.end.min(self.segment_offset + self.size);
    let size = if end > start { end - start } else { 0 };
    ExecutionSection {
      segment_offset: start,
      executable_offset: offset,
      size,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.size == 0
  }
}

/// An ExecutionSegment associates a series of virtual memory pages with data
/// stored in an executable file.
/// Each segment has a single set of read/write permissions, and must be page
/// aligned. These values directly determine how the page table entry is
/// constructed.
pub struct ExecutionSegment {
  /// Where the segment begins in virtual memory. This must be page-aligned.
  pub address: VirtualAddress,
  /// The size of the segment, in bytes. This must be a multiple of page size.
  pub size: usize,
  /// The full set of sections found in this segment. Because segments are sized
  /// to be a multiple of the page size, not all addresses in a segment will map
  /// to a section.
  pub sections: Vec<ExecutionSection>,
  /// Is the section user-writable?
  pub can_write: bool,
}

impl ExecutionSegment {
  /// Construct a new ExecutionSegment that begins at the specified virtual
  /// address, and contains the specified number of pages.
  pub fn at_address(address: VirtualAddress, pages: usize) -> Result<Self, ProcessMemoryError> {
    if !address.is_page_aligned() {
      return Err(ProcessMemoryError::SegmentWrongAlignment);
    }
    Ok(
      Self {
        address,
        size: pages * PAGE_SIZE_IN_BYTES,
        sections: Vec::new(),
        can_write: false,
      }
    )
  }

  pub fn add_section(&mut self, section: ExecutionSection) -> Result<(), ProcessMemoryError> {
    if section.segment_offset + section.size > self.size {
      return Err(ProcessMemoryError::SectionOutOfBounds);
    }
    self.sections.push(section);
    Ok(())
  }

  pub fn set_user_can_write(&mut self, flag: bool) {
    self.can_write = flag;
  }

  pub fn user_can_write(&self) -> bool {
    self.can_write
  }

  pub fn contains_address(&self, addr: &VirtualAddress) -> bool {
    for section in self.sections.iter() {
      if section.as_virtual_range(self.address).contains(addr) {
        return true;
      }
    }
    false
  }

  pub fn get_starting_address(&self) -> VirtualAddress {
    self.address
  }

  pub fn get_size(&self) -> usize {
    self.size
  }

  pub fn sections_iter(&self) -> impl Iterator<Item = &ExecutionSection> {
    self.sections.iter()
  }
}

impl Clone for ExecutionSegment {
  fn clone(&self) -> Self {
    Self {
      address: self.address,
      size: self.size,
      sections: self.sections.clone(),
      can_write: self.can_write,
    }
  }
}

/// MMapRegion represents a section of memory allocated by the `mmap` syscall.
/// `mmap` is used for allocating arbitrary memory, or mapping a region of
/// memory to a specific area of physical memory space or a file.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MMapRegion {
  pub address: VirtualAddress,
  pub size: usize,
  pub backed_by: MMapBacking,
}

impl MMapRegion {
  pub fn get_address_range(&self) -> Range<VirtualAddress> {
    let start = self.address;
    let end = start + self.size;
    start..end
  }

  pub fn contains_address(&self, addr: &VirtualAddress) -> bool {
    self.get_address_range().contains(addr)
  }

  pub fn is_empty(&self) -> bool {
    self.size == 0
  }
}

/// The backing type of a MMapRegion determines how it behaves when a page fault
/// occurs. It tells the kernel how to find the memory or data that this vmem
/// region points to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MMapBacking {
  /// This mmap region should directly point to a same-sized region of physical
  /// memory. This is necessary for interfacing with devices on the memory bus.
  Direct(PhysicalAddress),
  /// This region is backed by an arbitrary section of physical memory.
  Anonymous,
  /// Similar to Anonymous, but guarantees that the physical memory will be
  /// within the first 16MiB. This is necessary for old-school ISA DMA.
  DMA,
  /// This region is backed by the contents of a file. When a page fault occurs,
  /// the file will be read and the appropriate range will be copied to memory.
  DeviceFile,
}

pub struct MemoryRegions {
  /// A series of excution segments representing the program's code and data.
  execution_segments: Vec<ExecutionSegment>,
  /// Caches the first address following all execution segments
  heap_start: VirtualAddress,
  /// Determines the size of the heap found after the program's exeuction
  /// segments. This can be controlled with the `brk`/`sbrk` syscalls.
  heap_size: usize,
  /// The highest location in memory available to this process; only space below
  /// this will be allocated for mmap operations
  memory_top: usize,
  /// Collection of mmap regions. When a specific location is not requested,
  /// these will be allocated from the top of memory space downwards.
  mmap_regions: BTreeMap<VirtualAddress, MMapRegion>,
}

impl MemoryRegions {
  pub const fn new() -> Self {
    Self {
      execution_segments: Vec::new(),
      heap_start: VirtualAddress::new(0),
      heap_size: 0,
      memory_top: USER_KERNEL_BARRIER,
      mmap_regions: BTreeMap::new(),
    }
  }

  pub const fn with_memory_top(top: usize) -> Self {
    Self {
      execution_segments: Vec::new(),
      heap_start: VirtualAddress::new(0),
      heap_size: 0,
      memory_top: top,
      mmap_regions: BTreeMap::new(),
    }
  }

  pub fn get_execution_segments_start(&self) -> VirtualAddress {
    if self.execution_segments.is_empty() {
      return VirtualAddress::new(0);
    }
    let mut low = VirtualAddress::new(0xffffffff);
    for segment in self.execution_segments.iter() {
      let start = segment.get_starting_address();
      if start < low {
        low = start;
      }
    }
    low
  }

  pub fn get_execution_segments_end(&self) -> VirtualAddress {
    if self.execution_segments.is_empty() {
      return VirtualAddress::new(0);
    }
    let mut high = VirtualAddress::new(0);
    for segment in self.execution_segments.iter() {
      let end = segment.get_starting_address().offset(segment.get_size());
      if end > high {
        high = end;
      }
    }
    high
  }

  /// Return a reference to an execution segment if it contains the requested
  /// address. This is useful for handling a page fault.
  pub fn get_execution_segment_containing_address(&self, addr: &VirtualAddress) -> Option<&ExecutionSegment> {
    for segment in self.execution_segments.iter() {
      if segment.contains_address(addr) {
        return Some(segment);
      }
    }
    None
  }

  /// Replace any existing execution segments with a new set, resetting the heap
  /// in the process. Returns the previous set of segments so that it can be
  /// unmapped from the page table.
  pub fn reset_execution_segments(&mut self, segments: Vec<ExecutionSegment>) -> Vec<ExecutionSegment> {
    let old = core::mem::replace(&mut self.execution_segments, segments);
    let heap_start = self.get_execution_segments_end();
    self.heap_start = heap_start;
    self.heap_size = 0;
    old
  }

  pub fn get_heap_start(&self) -> VirtualAddress {
    self.heap_start
  }

  pub fn get_heap_size(&self) -> usize {
    self.heap_size
  }

  pub fn set_heap_size(&mut self, size: usize) {
    self.heap_size = size;
  }

  pub fn get_heap_address_range(&self) -> Range<VirtualAddress> {
    let start = self.heap_start;
    let end = start + self.heap_size;
    start..end
  }

  /// Similar to `get_heap_address_range`, but rounded to the next page boundary
  pub fn get_heap_page_range(&self) -> Range<VirtualAddress> {
    let start = self.heap_start;
    let mut end = (start + self.heap_size).as_usize();
    if end & 0xfff != 0 {
      end = (end + 0x1000) & 0xfffff000;
    }
    start..VirtualAddress::new(end)
  }

  pub fn can_fit_range(&self, range: Range<VirtualAddress>) -> bool {
    // Check if it fits before executable ranges
    if range.end <= self.get_execution_segments_start() {
      return true;
    }
    // If it intersects with the current heap, it doesn't fit
    let heap_range = self.get_heap_page_range();
    if ranges_overlap(&range, &heap_range) {
      return false;
    }
    // Check for intersection with each mmap'd range
    for (_, mmap) in self.mmap_regions.iter() {
      if ranges_overlap(&mmap.get_address_range(), &range) {
        return false;
      }
    }
    true
  }

  /// Find an appropriately sized space for the requested mmap region.
  /// Except... we don't actually use the hint for anything right now.
  pub fn find_free_mmap_space(&self, _hint: Option<VirtualAddress>, size: usize) -> Option<VirtualAddress> {
    let heap_end = self.get_heap_page_range().end;
    if VirtualAddress::new(self.memory_top - size) < heap_end {
      return None;
    }
    // Iterate backwards through the mmap set. If the space between the current
    // region and the previous one is large enough to fit the requested 
    let mut prev_start = VirtualAddress::new(self.memory_top);
    for (_, region) in self.mmap_regions.iter().rev() {
      let region_end = (region.address + region.size).next_page_barrier();
      let free_space = prev_start - region_end;
      if free_space >= size {
        return Some((prev_start - size).prev_page_barrier());
      }
      prev_start = region.address;
    }
    // TODO: Check this doesn't intersect with the heap
    let prev_page_barrier = (prev_start.as_usize() - size) & 0xfffff000;
    Some(VirtualAddress::new(prev_page_barrier))
  }

  /// Create a memory mapping for this process. This method backs the `mmap`
  /// syscall. It does not actually modify the page table -- the first time a
  /// process tries to access the newly mapped memory, the page fault handler
  /// will look up the mapping and fill the accessed page with data.
  /// On success, it returns the address of the requested block.
  pub fn mmap(&mut self, addr: Option<VirtualAddress>, size: usize, backing: MMapBacking) -> Result<VirtualAddress, ProcessMemoryError> {
    // Find an appropriate spot in virtual memory. If the caller specified a
    // location, we want to find the closest available space; otherwise, crawl
    // through the existing allocations until an appropriately sized space is
    // found.
    let location: Option<VirtualAddress> = match addr {
      Some(request_start) => {
        if !request_start.is_page_aligned() {
          return Err(ProcessMemoryError::MMapWrongAlignment);
        }
        let request_end = request_start + size;
        if self.can_fit_range(request_start..request_end) {
          Some(request_start)
        } else {
          self.find_free_mmap_space(Some(request_start), size)
        }
      },
      None => {
        self.find_free_mmap_space(None, size)
      },
    };

    match location {
      Some(free_space) => {
        // Add the mapping to the set
        let mapping = MMapRegion {
          address: free_space,
          size,
          backed_by: backing,          
        };
        self.mmap_regions.insert(free_space, mapping);
        Ok(free_space)
      },
      None => Err(ProcessMemoryError::NotEnoughMemory),
    }
  }

  /// Remove a memory mapping for this process.
  /// On success, it returns a range of addresses that are now freed up. This
  /// can be used elsewhere in the kernel to invalidate page table entries.
  pub fn munmap(&mut self, addr: VirtualAddress, length: usize) -> Result<Range<VirtualAddress>, ProcessMemoryError> {
    if length & 0xfff != 0 {
      return Err(ProcessMemoryError::MUnmapNotPageMultiple);
    }
    if addr.as_usize() >= USER_KERNEL_BARRIER || addr.as_usize() + length >= USER_KERNEL_BARRIER {
      return Err(ProcessMemoryError::MapOutOfBounds);
    }
    // We should really replace this BTree with an Interval Tree...
    // Iterate over all regions, and find the ones that need to be modified
    // Once that set has been computed, all intersected regions will be removed
    // from the map, and any remaining sub-regions will be put back.
    let mut unmap_start = addr;
    let mut unmap_length = length;
    let mut modified_regions: Vec<(VirtualAddress, Range<usize>)> = Vec::new();
    for (_, region) in self.mmap_regions.iter() {
      let region_range = region.get_address_range();
      if unmap_start < region_range.start && (unmap_start + unmap_length) > region_range.start {
        let delta = region_range.start - unmap_start;
        unmap_length -= delta;
        unmap_start = unmap_start + delta;
      }
      if region_range.contains(&unmap_start) {
        let can_unmap = region_range.end - unmap_start;
        let (to_remove, remaining) = if can_unmap > unmap_length {
          (unmap_length, 0)
        } else {
          (can_unmap, unmap_length - can_unmap)
        };
        unmap_length = remaining;
        let remove_start = unmap_start - region_range.start;
        let remove_end = remove_start + to_remove;
        modified_regions.push((region_range.start, remove_start..remove_end));
        unmap_start = unmap_start + to_remove;
        if remaining == 0 {
          break;
        }
      }
    }
    for modification in modified_regions {
      match self.mmap_regions.remove(&modification.0) {
        Some(region) => {
          if modification.1.start > 0 {
            let before = MMapRegion {
              address: region.address,
              size: modification.1.start,
              backed_by: region.backed_by,
            };
            self.mmap_regions.insert(region.address, before);
          }
          if modification.1.end < region.size {
            let new_size = region.size - modification.1.end;
            let new_address = region.address + (region.size - new_size);
            let after = MMapRegion {
              address: new_address,
              size: new_size,
              backed_by: region.backed_by,
            };
            self.mmap_regions.insert(new_address, after);
          }
        },
        None => (), // Unreachable
      }
    }
    Ok(addr..(addr + length))
  }

  /// Return a reference to a mmap region if it contains the requested
  /// address. This is useful for handling a page fault.
  pub fn get_mapping_containing_address(&self, addr: &VirtualAddress) -> Option<&MMapRegion> {
    for (_, region) in self.mmap_regions.iter() {
      if region.contains_address(addr) {
        return Some(region);
      }
    }
    None
  }
}

impl Clone for MemoryRegions {
  fn clone(&self) -> Self {
    Self {
      execution_segments: self.execution_segments.clone(),
      heap_start: self.heap_start,
      heap_size: self.heap_size,
      memory_top: self.memory_top,
      mmap_regions: self.mmap_regions.clone(),
    }
  }
}

#[derive(Debug)]
pub enum ProcessMemoryError {
  /// A segment was created outside of page alignment
  SegmentWrongAlignment,
  /// A section was constructed outside the bounds of its parent segment 
  SectionOutOfBounds,
  /// A mmap was attempted outside of page alignment
  MMapWrongAlignment,
  /// Not enough memory for the requested mapping
  NotEnoughMemory,
  /// A mapping or unmapping was requested outside of process memory
  MapOutOfBounds,
  /// Attempted to unmap a region of memory that wasn't a multiple of page size
  MUnmapNotPageMultiple,
}

pub fn ranges_overlap(a: &Range<VirtualAddress>, b: &Range<VirtualAddress>) -> bool {
  let min = a.start.min(b.start);
  let max = a.end.max(b.end);
  let a_length = a.end - a.start;
  let b_length = b.end - b.start;
  (a_length + b_length) > (max - min)
}

/// Specifies a type of relocation to be performed when executable memory is
/// paged in from the file.
#[derive(Copy, Clone)]
pub enum Relocation {
  /// Only type of relocation supported in a DOS EXE: add a 16-bit offset to a
  /// word at a specific address
  DosExe(VirtualAddress, u16)
}

impl Relocation {
  pub fn get_address(&self) -> VirtualAddress {
    match self {
      Self::DosExe(addr, _) => *addr,
    }
  }

  pub unsafe fn apply(&self) {
    match self {
      Self::DosExe(addr, offset) => {
        let ptr = addr.as_usize() as *mut u16;
        *ptr += *offset;
      },
    }
  }
}

/// Kernel-space memory is shared between all processes. Each process has its
/// own stack, although they all exist in the same address space. Beyond this,
/// all other memory (executable, heap, mmaps) are shared between all processes.
pub static KERNEL_MEMORY: RwLock<MemoryRegions> = RwLock::new(MemoryRegions::with_memory_top(KERNEL_MMAP_TOP));

pub fn kernel_mmap(addr: Option<VirtualAddress>, size: usize, backing: MMapBacking) -> Result<VirtualAddress, ProcessMemoryError> {
  let mut mem = KERNEL_MEMORY.write();
  let location = mem.mmap(addr, size, backing)?;

  Ok(location)
}

pub fn get_kernel_mapping(addr: VirtualAddress) -> Option<MMapRegion> {
  let kernel_mem = KERNEL_MEMORY.read();
  kernel_mem.get_mapping_containing_address(&addr).map(|m| m.clone())
}

#[cfg(test)]
mod tests {
  use super::{
    ranges_overlap,
    ExecutionSection,
    ExecutionSegment,
    MemoryRegions,
    MMapBacking,
    MMapRegion,
    VirtualAddress,
  };

  #[test]
  fn overlapping_ranges() {
    assert!(!ranges_overlap(
      &(VirtualAddress::new(0x100)..VirtualAddress::new(0x200)),
      &(VirtualAddress::new(0x300)..VirtualAddress::new(0x400)),
    ));

    assert!(!ranges_overlap(
      &(VirtualAddress::new(0x100)..VirtualAddress::new(0x200)),
      &(VirtualAddress::new(0x200)..VirtualAddress::new(0x400)),
    ));

    assert!(ranges_overlap(
      &(VirtualAddress::new(0x100)..VirtualAddress::new(0x200)),
      &(VirtualAddress::new(0x1ff)..VirtualAddress::new(0x400)),
    ));

    assert!(ranges_overlap(
      &(VirtualAddress::new(0x100)..VirtualAddress::new(0x300)),
      &(VirtualAddress::new(0x200)..VirtualAddress::new(0x220)),
    ));

    assert!(ranges_overlap(
      &(VirtualAddress::new(0x100)..VirtualAddress::new(0x300)),
      &(VirtualAddress::new(0x200)..VirtualAddress::new(0x400)),
    ));
  }

  #[test]
  fn segment_contains() {
    let mut segment = ExecutionSegment::at_address(VirtualAddress::new(0x1000), 2).unwrap();
    segment.add_section(ExecutionSection {
      segment_offset: 0x20,
      executable_offset: None,
      size: 0x100,
    }).unwrap();
    assert!(!segment.contains_address(&VirtualAddress::new(0x10)));
    assert!(!segment.contains_address(&VirtualAddress::new(0x1010)));
    assert!(segment.contains_address(&VirtualAddress::new(0x1020)));
    assert!(segment.contains_address(&VirtualAddress::new(0x10f0)));
  }

  #[test]
  fn explicit_mmap() {
    let mut regions = MemoryRegions::new();
    assert_eq!(
      regions.mmap(Some(VirtualAddress::new(0x4000)), 0x1000, MMapBacking::Anonymous).unwrap(),
      VirtualAddress::new(0x4000),
    );
    assert_eq!(
      regions.mmap(Some(VirtualAddress::new(0x6000)), 0x2000, MMapBacking::Anonymous).unwrap(),
      VirtualAddress::new(0x6000),
    );
    assert_eq!(
      regions.mmap(Some(VirtualAddress::new(0x5000)), 0x2000, MMapBacking::Anonymous).unwrap(),
      VirtualAddress::new(0xbfffe000),
    );
  }

  #[test]
  fn unmapping() {
    let mut regions = MemoryRegions::new();
    regions.mmap(Some(VirtualAddress::new(0x1000)), 0x1000, MMapBacking::Anonymous).unwrap();
    assert_eq!(
      regions.munmap(VirtualAddress::new(0x1000), 0x1000).unwrap(),
      VirtualAddress::new(0x1000)..VirtualAddress::new(0x2000),
    );
    assert!(regions.mmap_regions.is_empty());
    regions.mmap(Some(VirtualAddress::new(0x1000)), 0x2000, MMapBacking::Anonymous).unwrap();
    regions.mmap(Some(VirtualAddress::new(0x4000)), 0x3000, MMapBacking::Anonymous).unwrap();
    assert_eq!(
      regions.munmap(VirtualAddress::new(0x2000), 0x2000).unwrap(),
      VirtualAddress::new(0x2000)..VirtualAddress::new(0x4000),
    );
    {
      let shrunk = regions.mmap_regions.get(&VirtualAddress::new(0x1000)).unwrap();
      assert_eq!(shrunk.address, VirtualAddress::new(0x1000));
      assert_eq!(shrunk.size, 0x1000);
    }
    assert_eq!(regions.mmap_regions.len(), 2);
    assert_eq!(
      regions.munmap(VirtualAddress::new(0x1000), 0x4000).unwrap(),
      VirtualAddress::new(0x1000)..VirtualAddress::new(0x5000),
    );
    {
      let shrunk = regions.mmap_regions.get(&VirtualAddress::new(0x5000)).unwrap();
      assert_eq!(shrunk.address, VirtualAddress::new(0x5000));
      assert_eq!(shrunk.size, 0x2000);
    }
    assert_eq!(regions.mmap_regions.len(), 1);
  }

  #[test]
  fn auto_allocated_mmap() {
    let mut regions = MemoryRegions::new();
    assert_eq!(
      regions.mmap(None, 0x1000, MMapBacking::Anonymous).unwrap(),
      VirtualAddress::new(0xbffff000),
    );
    assert_eq!(
      regions.mmap(None, 0x400, MMapBacking::Anonymous).unwrap(),
      VirtualAddress::new(0xbfffe000),
    );
  }

  #[test]
  fn auto_allocated_reuses_space() {
    let mut regions = MemoryRegions::new();
    assert_eq!(
      regions.mmap(None, 0x5000, MMapBacking::Anonymous).unwrap(),
      VirtualAddress::new(0xbfffb000),
    );
    regions.munmap(VirtualAddress::new(0xbfffc000), 0x2000).unwrap();
    assert_eq!(
      *regions.mmap_regions.get(&VirtualAddress::new(0xbfffb000)).unwrap(),
      MMapRegion { address: VirtualAddress::new(0xbfffb000), size: 0x1000, backed_by: MMapBacking::Anonymous },
    );
    assert_eq!(
      *regions.mmap_regions.get(&VirtualAddress::new(0xbfffe000)).unwrap(),
      MMapRegion { address: VirtualAddress::new(0xbfffe000), size: 0x2000, backed_by: MMapBacking::Anonymous },
    );
    assert_eq!(
      regions.mmap(None, 0x1000, MMapBacking::Anonymous).unwrap(),
      VirtualAddress::new(0xbfffd000),
    );
    assert_eq!(
      regions.mmap(None, 0x2000, MMapBacking::Anonymous).unwrap(),
      VirtualAddress::new(0xbfff9000),
    );
  }

  #[test]
  fn clipping_sections() {
    assert_eq!(
      (ExecutionSection { segment_offset: 0x1000, executable_offset: None, size: 0x2400 })
        .clip_to(0x500..0x4000),
      (ExecutionSection { segment_offset: 0x1000, executable_offset: None, size: 0x2400 })
    );
    assert_eq!(
      (ExecutionSection { segment_offset: 0x1000, executable_offset: None, size: 0x2400 })
        .clip_to(0x1200..0x4000),
      (ExecutionSection { segment_offset: 0x1200, executable_offset: None, size: 0x2200 })
    );
    assert_eq!(
      (ExecutionSection { segment_offset: 0x1000, executable_offset: None, size: 0x2400 })
        .clip_to(0x800..0x2300),
      (ExecutionSection { segment_offset: 0x1000, executable_offset: None, size: 0x1300 })
    );
    assert_eq!(
      (ExecutionSection { segment_offset: 0x1000, executable_offset: None, size: 0x2400 })
        .clip_to(0x2500..0x2900),
      (ExecutionSection { segment_offset: 0x2500, executable_offset: None, size: 0x400 })
    );
    assert_eq!(
      (ExecutionSection { segment_offset: 0x1000, executable_offset: None, size: 0x2400 })
        .clip_to(0x3400..0x4000),
      (ExecutionSection { segment_offset: 0x3400, executable_offset: None, size: 0 })
    );
    assert_eq!(
      (ExecutionSection { segment_offset: 0x1000, executable_offset: None, size: 0x2400 })
        .clip_to(0x4000..0x5000),
      (ExecutionSection { segment_offset: 0x4000, executable_offset: None, size: 0 })
    );
    assert_eq!(
      (ExecutionSection { segment_offset: 0, executable_offset: Some(0x350), size: 0x500 })
        .clip_to(0x200..0x4000),
      (ExecutionSection { segment_offset: 0x200, executable_offset: Some(0x550), size: 0x300 })
    );
  }
}
