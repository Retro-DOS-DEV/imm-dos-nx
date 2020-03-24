pub struct LocalHandle(u32);

impl LocalHandle {
  pub fn new(handle: u32) -> LocalHandle {
    LocalHandle(handle)
  }

  pub fn as_u32(&self) -> u32 {
    self.0
  }
}

pub trait DeviceDriver {
  fn open(&self) -> Result<LocalHandle, ()> {
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
}