use super::super::address::PhysicalAddress;
use super::frame::Frame;

#[derive(Copy, Clone, Eq)]
pub struct FrameRange {
  start: usize, // First byte in the frame range
  length: usize, // Size of the range, in bytes
}

impl FrameRange {
  /**
   * If start is not page-aligned (address & 0xfff == 0),
   * bad things will happen...
   */
  pub const fn new(start: usize, length: usize) -> FrameRange {
    FrameRange {
      start,
      length,
    }
  }

  pub fn get_first_frame_index(&self) -> usize {
    self.start >> 12
  }

  pub fn get_last_frame_index(&self) -> usize {
    (self.start + self.length - 1) >> 12
  }

  pub fn get_starting_address(&self) -> PhysicalAddress {
    PhysicalAddress::new(self.start)
  }

  pub fn get_ending_address(&self) -> PhysicalAddress {
    PhysicalAddress::new(self.start + self.length - 1)
  }

  pub fn get_first_frame(&self) -> Frame {
    Frame::new(self.start)
  }

  pub fn contains_address(&self, addr: PhysicalAddress) -> bool {
    let addr_usize = addr.as_usize();
    self.start <= addr_usize && (self.start + self.length) > addr_usize
  }

  pub fn size_in_frames(&self) -> usize {
    self.length >> 12
  }

  pub fn size_in_bytes(&self) -> usize {
    self.length
  }

  pub unsafe fn zero_memory(&self) {
    let start = self.start;
    let mut ptr = start as *mut u32;
    let size = self.size_in_bytes() >> 2;
    for _ in 0..size {
      *ptr = 0;
      ptr = ptr.offset(1);
    }
  }
}

impl PartialEq for FrameRange {
  fn eq(&self, other: &Self) -> bool {
    self.get_starting_address() == other.get_starting_address() &&
    self.size_in_bytes() == other.size_in_bytes()
  }
}

impl core::fmt::Debug for FrameRange {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_tuple("FrameRange")
      .field(&self.get_starting_address())
      .field(&self.get_ending_address())
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use super::{FrameRange, PhysicalAddress};

  #[test]
  fn bounds() {
    let f = FrameRange::new(0x4000, 0x4000);
    assert_eq!(f.get_first_frame_index(), 4);
    assert_eq!(f.get_last_frame_index(), 7);
    assert_eq!(f.get_starting_address().as_usize(), 0x4000);
    assert_eq!(f.get_ending_address().as_usize(), 0x7fff);
    assert!(f.contains_address(PhysicalAddress::new(0x4000)));
    assert!(f.contains_address(PhysicalAddress::new(0x5055)));
    assert!(f.contains_address(PhysicalAddress::new(0x7fff)));
    assert!(!f.contains_address(PhysicalAddress::new(0x8000)))
  }
}
