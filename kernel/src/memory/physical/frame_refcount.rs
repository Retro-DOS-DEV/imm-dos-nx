use alloc::vec::Vec;
use super::frame::Frame;
use super::super::address::PhysicalAddress;

/// While the purpose of the Frame Bitmap is to determine which areas of memory
/// are available, the Refcount Table determines how many different pages refer
/// to allocated, "anonymous" memory.
/// When an Anonymous frame is released, the reference count is decremented.
/// If that count reaches zero, the frame is freed in the Bitmap.
///
/// The reference count is also necessary for copy-on-write support. When a CoW
/// page is written to, the page fault handler will check how many processes
/// still point to the current page. If it is greater than one, the page will be
/// copied to a new frame, and the reference count decremented.
pub struct FrameRefcount {
  references: Vec<u8>,
}

impl FrameRefcount {
  pub fn new(frame_count: usize) -> FrameRefcount {
    let mut references = Vec::with_capacity(frame_count);
    for _ in 0..frame_count {
      references.push(0);
    }
    FrameRefcount {
      references,
    }
  }

  /// Increment the number of references to a given frame,
  /// returning the new total.
  pub fn reference_frame_at_index(&mut self, index: usize) -> u8 {
    // Do some amount of checking if the references exceed 255
    let new_count = self.references[index] + 1;
    self.references[index] = new_count;
    new_count
  }

  /// Increment the number of references to the frame containing a given
  /// physical memory address, returning the new total.
  pub fn reference_frame_at_address(&mut self, addr: PhysicalAddress) -> u8 {
    let index = addr.as_usize() / 0x1000;
    self.reference_frame_at_index(index)
  }

  pub fn reference_frame(&mut self, frame: Frame) -> u8 {
    self.reference_frame_at_address(frame.get_address())
  }

  /// Decrement the number of references to the specific frame
  pub fn release_frame_at_index(&mut self, index: usize) -> u8 {
    let current_count = self.references[index];
    if current_count == 0 {
      return 0;
    }
    let new_count = current_count - 1;
    self.references[index] = new_count;
    new_count
  }

  pub fn release_frame_at_address(&mut self, addr: PhysicalAddress) -> u8 {
    let index = addr.as_usize() / 0x1000;
    self.release_frame_at_index(index)
  }

  pub fn release_frame(&mut self, frame: Frame) -> u8 {
    self.release_frame_at_address(frame.get_address())
  }

  pub fn current_count_at_index(&mut self, index: usize) -> u8 {
    self.references[index]
  }

  pub fn current_count_at_address(&mut self, addr: PhysicalAddress) -> u8 {
    let index = addr.as_usize() / 0x1000;
    self.current_count_at_index(index)
  }
}