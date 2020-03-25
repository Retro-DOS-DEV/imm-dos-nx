use crate::files::handle::LocalHandle;
use super::driver::{DeviceDriver};

pub struct NullDevice {

}

impl NullDevice {
  pub const fn new() -> NullDevice {
    NullDevice {

    }
  }
}

impl DeviceDriver for NullDevice {
  fn open(&self, _handle: LocalHandle) -> Result<(), ()> {
    Ok(())
  }

  fn close(&self, _handle: LocalHandle) -> Result<(), ()> {
    Ok(())
  }

  fn read(&self, _handle: LocalHandle, _buffer: &mut [u8]) -> Result<usize, ()> {
    Ok(0)
  }

  fn write(&self, _handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(buffer.len())
  }
}