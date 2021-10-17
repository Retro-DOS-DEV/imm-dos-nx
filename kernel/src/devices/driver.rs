use crate::files::cursor::SeekMethod;
use crate::task::id::ProcessID;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IOHandle(usize);

impl IOHandle {
  pub fn new(inner: usize) -> Self {
    Self(inner)
  }

  pub fn as_usize(&self) -> usize {
    self.0
  }
}

pub trait DeviceDriver {
  #![allow(unused_variables)]

  fn open(&self) -> Result<IOHandle, ()>;

  fn read(&self, index: IOHandle, buffer: &mut [u8]) -> Result<usize, ()>;

  fn write(&self, index: IOHandle, buffer: &[u8]) -> Result<usize, ()>;

  fn close(&self, index: IOHandle) -> Result<(), ()>;

  fn seek(&self, index: IOHandle, offset: SeekMethod) -> Result<usize, ()> {
    Err(())
  }

  fn reopen(&self, index: IOHandle, id: ProcessID) -> Result<IOHandle, ()> {
    Err(())
  }
}

pub type DeviceDriverType = dyn DeviceDriver + Sync + Send;
