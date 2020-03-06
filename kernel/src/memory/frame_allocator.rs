use core::fmt;

use super::frame::Frame;

pub enum FrameAllocatorError {
  OutOfMemory,
}

impl fmt::Debug for FrameAllocatorError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      FrameAllocatorError::OutOfMemory => write!(f, "FrameAllocatorError: Out of Physical Memory"),
    }
  }
}

pub type FrameAllocatorResult = Result<Frame, FrameAllocatorError>;

pub trait FrameAllocator {
  fn allocate(&mut self) -> FrameAllocatorResult;
  fn is_free(&self, frame: Frame) -> bool;
  fn release(&mut self, frame: Frame);
  fn count_frames(&self) -> usize;
  fn count_free_frames(&self) -> usize;
}
