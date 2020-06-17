use super::super::address::VirtualAddress;
use super::super::physical::frame_range::FrameRange;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MemoryRegionType {
  MemMapped(usize, usize), // Backed by a memmapped file
  Direct(FrameRange), // Backed by an explicit physical memory range
  IO(FrameRange), // Similar to Direct, but used for IO devices like PCI
  Anonymous(FrameRange), // Backed by arbitrarily-allocated physical memory
}

pub struct VirtualMemoryRegion {
  start: VirtualAddress, // Starting byte of the region, should be page-aligned
  size: usize, // Length of the region, in bytes
  backed_by: MemoryRegionType, // In a page fault, where does the data come from?
  writable: bool, // Should page table entry be writable
}

impl VirtualMemoryRegion {
  pub fn get_starting_address_as_usize(&self) -> usize {
    self.start.as_usize()
  }

  pub fn contains_address(&self, addr: VirtualAddress) -> bool {
    let addr_usize = addr.as_usize();
    let start_usize = self.start.as_usize();
    addr_usize >= start_usize && addr_usize < start_usize + self.size
  }

  pub fn backing_type(&self) -> MemoryRegionType {
    self.backed_by
  }

  pub fn is_writable(&self) -> bool {
    self.writable
  }
}

impl PartialOrd for VirtualMemoryRegion {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for VirtualMemoryRegion {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.get_starting_address_as_usize()
      .cmp(&other.get_starting_address_as_usize())
  }
}

impl PartialEq for VirtualMemoryRegion {
  fn eq(&self, other: &Self) -> bool {
    self.get_starting_address_as_usize() == other.get_starting_address_as_usize()
  }
}

impl Eq for VirtualMemoryRegion {}
