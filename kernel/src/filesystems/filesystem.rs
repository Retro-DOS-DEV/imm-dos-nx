use crate::files::{cursor::SeekMethod, handle::LocalHandle};

pub trait FileSystem {
  fn open(&self, path: &str) -> Result<LocalHandle, ()>;
  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()>;
  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()>;
  fn close(&self, handle: LocalHandle) -> Result<(), ()>;
  fn dup(&self, handle: LocalHandle) -> Result<LocalHandle, ()>;
  fn ioctl(&self, handle: LocalHandle, command: u32, arg: u32) -> Result<u32, ()>;
  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()>;
}