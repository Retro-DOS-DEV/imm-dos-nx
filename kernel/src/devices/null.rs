use core::sync::atomic::{AtomicUsize, Ordering};
use super::driver::DeviceDriver;

pub struct NullDriver {
  next_handle: AtomicUsize,
}

impl NullDriver {
  pub const fn new() -> Self {
    Self {
      next_handle: AtomicUsize::new(1),
    }
  }
}

impl DeviceDriver for NullDriver {
  fn open(&self) -> Result<usize, ()> {
    let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
    Ok(handle)
  }

  fn close(&self, _index: usize) -> Result<(), ()> {
    Ok(())
  }

  fn read(&self, _index: usize, _buffer: &mut [u8]) -> Result<usize, ()> {
    Ok(0)
  }

  fn write(&self, _index: usize, buffer: &[u8]) -> Result<usize, ()> {
    Ok(buffer.len())
  }
}
