use core::cmp;
use core::fmt;
use core::ops;

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

impl ops::Add<usize> for PhysicalAddress {
  type Output = Self;

  fn add(self, other: usize) -> Self {
    Self(self.0 + other)
  }
}

impl ops::Sub<PhysicalAddress> for PhysicalAddress {
  type Output = usize;

  fn sub(self, other: Self) -> usize {
    self.as_usize() - other.as_usize()
  }
}

impl fmt::Debug for PhysicalAddress {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "PhysicalAddress({:#010x})", self.0)
  }
}

pub const PAGE_SIZE_IN_BYTES: usize = 0x1000;

#[derive(Copy, Clone, Eq)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
  pub const fn new(addr: usize) -> VirtualAddress {
    VirtualAddress(addr)
  }

  pub const fn as_usize(&self) -> usize {
    self.0 as usize
  }

  pub const fn as_u32(&self) -> u32 {
    self.0 as u32
  }

  pub fn get_page_directory_index(&self) -> usize {
    self.0 >> 22
  }

  pub fn get_page_table_index(&self) -> usize {
    (self.0 >> 12) & 0x3ff
  }

  pub fn offset(&self, delta: usize) -> VirtualAddress {
    VirtualAddress::new(self.0 + delta)
  }

  pub fn is_page_aligned(&self) -> bool {
    self.0 & 0xfff == 0
  }

  pub fn next_page_barrier(&self) -> VirtualAddress {
    if self.0 & 0xfff == 0 {
      VirtualAddress::new(self.0)
    } else {
      let next = self.0 + 0x1000;
      VirtualAddress::new(next & 0xfffff000)
    }
  }

  pub fn prev_page_barrier(&self) -> VirtualAddress {
    VirtualAddress::new(self.0 & 0xfffff000)
  }
}

impl cmp::Ord for VirtualAddress {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for VirtualAddress {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for VirtualAddress {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl ops::Add<usize> for VirtualAddress {
  type Output = Self;

  fn add(self, other: usize) -> Self {
    Self(self.0 + other)
  }
}

impl ops::Sub<VirtualAddress> for VirtualAddress {
  type Output = usize;

  fn sub(self, other: Self) -> usize {
    self.as_usize() - other.as_usize()
  }
}

impl ops::Sub<usize> for VirtualAddress {
  type Output = VirtualAddress;

  fn sub(self, other: usize) -> VirtualAddress {
    VirtualAddress::new(self.as_usize() - other)
  }
}

impl fmt::Debug for VirtualAddress {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "VirtualAddress({:#010x})", self.0)
  }
}