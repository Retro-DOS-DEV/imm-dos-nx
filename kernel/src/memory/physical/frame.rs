use super::super::address::PhysicalAddress;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Frame(usize);

impl Frame {
  pub const fn new(start: usize) -> Frame {
    Frame(start & 0xfffff000)
  }

  pub fn next_frame(&self) -> Frame {
    Frame(self.0 + 0x1000)
  }

  pub fn get_address(&self) -> PhysicalAddress {
    PhysicalAddress::new(self.0)
  }

  pub fn containing_address(addr: PhysicalAddress) -> Frame {
    let frame_start = addr.as_usize() & 0xfffff000;
    Frame::new(frame_start)
  }

  pub unsafe fn zero_memory(&self) {
    let start = self.get_address().as_usize();
    let mut ptr = start as *mut u32;
    for _ in 0..1024 {
      *ptr = 0;
      ptr = ptr.offset(1);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Frame;

  #[test]
  fn frame_alignment() {
    let f = Frame::new(0xf030);
    assert_eq!(f.get_address().as_usize(), 0xf000);
  }
}
