use core::sync::atomic::{AtomicU32, Ordering};
use super::driver::{DeviceDriver, LocalHandle};

pub struct NullDevice {
  next_handle: AtomicU32,
}

impl NullDevice {
  pub const fn new() -> NullDevice {
    NullDevice {
      next_handle: AtomicU32::new(1),
    }
  }
}

impl DeviceDriver for NullDevice {
  fn open(&self) -> Result<LocalHandle, ()> {
    let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
    Ok(LocalHandle::new(handle))
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