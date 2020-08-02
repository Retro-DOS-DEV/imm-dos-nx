use crate::files::handle::LocalHandle;
use super::super::address::VirtualAddress;
use super::super::physical::frame_range::FrameRange;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MemoryRegionType {
  /// Memory backed by a memmapped file
  MemMapped(usize, LocalHandle, usize),
  /// Memory backed by an explicit physical memory range, like video RAM
  Direct(FrameRange),
  /// Backed by arbitrarily-allocated physical memory
  Anonymous(ExpansionDirection),
  /// Similar to Anonymous, but guaranteed to be backed by a contiguous range of
  /// physical memory within the lower 16MB
  DMA(FrameRange),
}

/// Used for ranges that auto-expand when you access the first/last frame.
/// Upon mapping that frame due to a pagefault, the range will get extended if
/// there is space.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ExpansionDirection {
  Before,
  After,
  None,
}

/// Determines which user-mode permission flags get set on Page Table entries
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Permissions {
  ReadOnly,
  ReadWrite,
  CopyOnWrite,
}

/// A Virtual Memory Region represents how pages should be mapped within a
/// range of virtual addresses. When a page fault occurs, the kernel attempts to
/// find a range containing the requested address, and maps it according to the
/// parameters of the range.
#[derive(Copy, Clone)]
pub struct VirtualMemoryRegion {
  /// Starting byte of the region, should be page-aligned
  start: VirtualAddress,
  /// Length of the region, in bytes
  size: usize,
  /// In a page fault, where does the data come from?
  backed_by: MemoryRegionType,
  /// Should the page table entry be writable in user mode
  permissions: Permissions,
}

impl VirtualMemoryRegion {
  pub const fn new(start: VirtualAddress, size: usize, backed_by: MemoryRegionType, permissions: Permissions) -> VirtualMemoryRegion {
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

  pub fn set_starting_address(&mut self, start: VirtualAddress) {
    self.start = start;
  }

  pub fn get_starting_address(&self) -> VirtualAddress {
    self.start
  }

  pub fn get_starting_address_as_usize(&self) -> usize {
    self.start.as_usize()
  }

  pub fn get_size(&self) -> usize {
    self.size
  }

  pub fn set_size(&mut self, size: usize) {
    self.size = size;
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

  /// Expand the range by a specified number of frames, in the range's expansion
  /// direction. If the expansion direction is None, the range will not be
  /// modified.
  /// Returns the new size of the range.
  pub fn expand(&mut self, new_frame_count: usize) -> usize {
    let expansion_size = new_frame_count * 0x1000;
    match self.backed_by {
      MemoryRegionType::Anonymous(exp) => match exp {
        ExpansionDirection::After => {
          self.size += expansion_size;
        },
        ExpansionDirection::Before => {
          if self.start.as_usize() >= expansion_size {
            self.start = VirtualAddress::new(self.start.as_usize() - expansion_size);
            self.size += expansion_size;
          }
        },
        _ => (),
      },
      _ => (),
    }
    self.size
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
  /// Memory ranges are intended to be sorted by their start point. We rely on
  /// the collection to prevent any overlaps.
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.get_starting_address_as_usize()
      .cmp(&other.get_starting_address_as_usize())
  }
}

impl PartialEq for VirtualMemoryRegion {
  /// For sorting purposes, we consider two memory regions equivalent if they
  /// start at the same location.
  fn eq(&self, other: &Self) -> bool {
    self.get_starting_address_as_usize() == other.get_starting_address_as_usize()
  }
}

impl Eq for VirtualMemoryRegion {}
