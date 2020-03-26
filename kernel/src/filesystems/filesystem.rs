use crate::files::handle::LocalHandle;

pub trait FileSystem {
  fn open(&self, path: &str) -> Result<LocalHandle, ()>;
  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()>;
  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()>;
  fn close(&self, handle: LocalHandle) -> Result<(), ()>;
}