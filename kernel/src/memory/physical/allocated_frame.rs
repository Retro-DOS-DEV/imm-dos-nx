use core::mem::ManuallyDrop;
use super::frame::Frame;
use super::super::address::PhysicalAddress;

#[must_use]
pub struct AllocatedFrame {
  frame_start: PhysicalAddress
}

impl AllocatedFrame {
  pub fn new(frame_start: PhysicalAddress) -> Self {
    Self {
      frame_start,
    }
  }

  pub fn get_address(&self) -> PhysicalAddress {
    self.frame_start
  }

  pub fn to_frame(self) -> Frame {
    let af = ManuallyDrop::new(self);
    Frame::new(af.frame_start.as_usize())
  }
}

impl Drop for AllocatedFrame {
  fn drop(&mut self) {
    panic!("AllocatedFrame must be mapped or freed");
  }
}