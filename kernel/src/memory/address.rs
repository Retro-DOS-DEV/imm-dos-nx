use core::cmp;
use core::fmt;

#[derive(Copy, Clone, Eq)]
#[repr(transparent)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
  pub const fn new(addr: usize) -> PhysicalAddress {
    PhysicalAddress(addr)
  }

  pub fn as_usize(&self) -> usize {
    self.0 as usize
  }

  pub fn as_u32(&self) -> u32 {
    self.0 as u32
  }
}

impl cmp::Ord for PhysicalAddress {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for PhysicalAddress {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for PhysicalAddress {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl fmt::Debug for PhysicalAddress {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "PhysicalAddress({:#010x})", self.0)
  }
}

#[derive(Copy, Clone, Eq)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
  pub const fn new(addr: usize) -> VirtualAddress {
    VirtualAddress(addr)
  }

  pub fn as_usize(&self) -> usize {
    self.0 as usize
  }

  pub fn as_u32(&self) -> u32 {
    self.0 as u32
  }

  pub fn get_page_directory_index(&self) -> usize {
    self.0 >> 22
  }

  pub fn get_page_table_index(&self) -> usize {
    (self.0 >> 12) & 0x3ff
  }
}

impl PartialEq for VirtualAddress {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl fmt::Debug for VirtualAddress {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "VirtualAddress({:#010x})", self.0)
  }
}