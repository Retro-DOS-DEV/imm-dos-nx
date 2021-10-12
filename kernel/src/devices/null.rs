use core::sync::atomic::{AtomicUsize, Ordering};
use super::driver::{DeviceDriver, IOHandle};

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
  fn open(&self) -> Result<IOHandle, ()> {
    let handle = IOHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst));
    Ok(handle)
  }

  fn close(&self, _index: IOHandle) -> Result<(), ()> {
    Ok(())
  }

  fn read(&self, _index: IOHandle, _buffer: &mut [u8]) -> Result<usize, ()> {
    Ok(0)
  }

  fn write(&self, _index: IOHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(buffer.len())
  }
}
