use crate::files::handle::LocalHandle;
use super::super::address::VirtualAddress;
use super::super::physical::frame_range::FrameRange;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MemoryRegionType {
  MemMapped(usize, LocalHandle, usize), // Backed by a memmapped file
  Direct(FrameRange), // Backed by an explicit physical memory range
  IO(FrameRange), // Similar to Direct, but used for IO devices like PCI
  Anonymous(ExpansionDirection), // Backed by arbitrarily-allocated physical memory
}

/**
 * Used for ranges that auto-expand when you access the first/last frame.
 * Upon mapping that frame due to a pagefault, the range will get extended if
 * there is space.
 */
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ExpansionDirection {
  Before,
  After,
  None,
}

/**
 * 
 */
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Permissions {
  ReadOnly,
  ReadWrite,
  CopyOnWrite,
}

#[derive(Copy, Clone)]
pub struct VirtualMemoryRegion {
  start: VirtualAddress, // Starting byte of the region, should be page-aligned
  size: usize, // Length of the region, in bytes
  backed_by: MemoryRegionType, // In a page fault, where does the data come from?
  permissions: Permissions, // Should page table entry be writable
}

impl VirtualMemoryRegion {
  pub fn new(start: VirtualAddress, size: usize, backed_by: MemoryRegionType, permissions: Permissions) -> VirtualMemoryRegion {
    VirtualMemoryRegion {
      start,
      size,
      backed_by,
      permissions,
    }
  }

  pub fn empty() -> VirtualMemoryRegion {
    VirtualMemoryRegion {
      start: VirtualAddress::new(0),
      size: 0,
      backed_by: MemoryRegionType::Anonymous(ExpansionDirection::None),
      permissions: Permissions::ReadOnly,
    }
  }

  pub fn get_starting_address_as_usize(&self) -> usize {
    self.start.as_usize()
  }

  pub fn get_size(&self) -> usize {
    self.size
  }

  pub fn contains_address(&self, addr: VirtualAddress) -> bool {
    let addr_usize = addr.as_usize();
    let start_usize = self.start.as_usize();
    addr_usize >= start_usize && addr_usize < start_usize + self.size
  }

  pub fn backing_type(&self) -> MemoryRegionType {
    self.backed_by
  }

  pub fn get_permissions(&self) -> Permissions {
    self.permissions
  }

  pub fn can_extend_before(&self) -> bool {
    match self.backed_by {
      MemoryRegionType::Anonymous(ExpansionDirection::Before) => true,
      _ => false,
    }
  }

  pub fn extend_before(&mut self, count: usize) {
    if self.start.as_usize() >= 0x1000 * count {
      self.start = VirtualAddress::new(self.start.as_usize() - 0x1000 * count);
    }
  }

  pub fn copy_for_new_process(&self) -> VirtualMemoryRegion {
    self.copy_with_permissions(self.permissions)
  }

  pub fn copy_with_permissions(&self, permissions: Permissions) -> VirtualMemoryRegion {
    VirtualMemoryRegion {
      start: self.start,
      size: self.size,
      backed_by: self.backed_by,
      permissions,
    }
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
