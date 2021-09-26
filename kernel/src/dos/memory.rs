
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

pub unsafe fn get_asciiz_string(addr: SegmentedAddress) -> &'static str {
  let start = addr.as_address();
  let start_ptr = start as *const u8;
  // for sanity, limit the string length to the end of the DS segment
  let max_length: usize = 0x10000 - (addr.offset as usize);
  let mut length: usize = 0;
  loop {
    let ch = start_ptr.offset(length as isize);
    if *ch == 0 {
      break;
    }
    if length >= max_length {
      break;
    }
    length += 1;
  }
  let slice = core::slice::from_raw_parts(start_ptr, length);
  core::str::from_utf8_unchecked(slice)
}