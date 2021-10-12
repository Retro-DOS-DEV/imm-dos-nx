use core::sync::atomic::{AtomicUsize, Ordering};
use super::driver::{DeviceDriver, IOHandle};

pub struct ZeroDriver {
  next_handle: AtomicUsize,
}

impl ZeroDriver {
  pub const fn new() -> Self {
    Self {
      next_handle: AtomicUsize::new(1),
    }
  }
}

impl DeviceDriver for ZeroDriver {
  fn open(&self) -> Result<IOHandle, ()> {
    let handle = IOHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst));
    Ok(handle)
  }

  fn close(&self, _index: IOHandle) -> Result<(), ()> {
    Ok(())
  }

  fn read(&self, _index: IOHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    for i in 0..buffer.len() {
      buffer[i] = 0;
    }
    Ok(buffer.len())
  }

  fn write(&self, _index: IOHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(buffer.len())
  }
}
