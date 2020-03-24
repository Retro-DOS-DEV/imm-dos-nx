use core::sync::atomic::{AtomicU32, Ordering};
use super::driver::{DeviceDriver, LocalHandle};

pub struct ZeroDevice {
  next_handle: AtomicU32,
}

impl ZeroDevice {
  pub const fn new() -> ZeroDevice {
    ZeroDevice {
      next_handle: AtomicU32::new(1),
    }
  }
}

impl DeviceDriver for ZeroDevice {
  fn open(&self) -> Result<LocalHandle, ()> {
    let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
    Ok(LocalHandle::new(handle))
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