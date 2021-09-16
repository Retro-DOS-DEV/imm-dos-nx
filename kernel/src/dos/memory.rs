
/// 16-bit code addresses memory using segments
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SegmentedAddress {
  pub segment: u16,
  pub offset: u16,
}

impl SegmentedAddress {
  pub fn as_address(&self) -> usize {
    ((self.segment as usize) << 4) + (self.offset as usize)
  }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Segment(pub u16);

impl Segment {
  pub fn as_u16(&self) -> u16 {
    self.0
  }

  pub fn as_address(&self) -> usize {
    (self.0 as usize) << 4
  }
}