use crate::files::handle::LocalHandle;
use super::driver::{DeviceDriver};

pub struct ZeroDevice {

}

impl ZeroDevice {
  pub const fn new() -> ZeroDevice {
    ZeroDevice {

    }
  }
}

impl DeviceDriver for ZeroDevice {
  fn open(&self, _handle: LocalHandle) -> Result<(), ()> {
    Ok(())
  }

  fn close(&self, _handle: LocalHandle) -> Result<(), ()> {
    Ok(())
  }

  fn read(&self, _handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let mut index = 0;
    while index < buffer.len() {
      buffer[index] = 0;
      index += 1;
    }
    Ok(index)
  }

  fn write(&self, _handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(buffer.len())
  }
}