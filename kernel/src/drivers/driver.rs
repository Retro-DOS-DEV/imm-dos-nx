use crate::files::{cursor::SeekMethod, handle::LocalHandle};

pub trait DeviceDriver {
  fn open(&self, handle: LocalHandle) -> Result<(), ()> {
    Err(())
  }

  fn close(&self, _handle: LocalHandle) -> Result<(), ()> {
    Err(())
  }

  fn read(&self, _handle: LocalHandle, _buffer: &mut [u8]) -> Result<usize, ()> {
    Err(())
  }

  fn write(&self, _handle: LocalHandle, _buffer: &[u8]) -> Result<usize, ()> {
    Err(())
  }

  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()> {
    Err(())
  }
}