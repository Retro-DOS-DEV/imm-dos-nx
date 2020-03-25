use super::filesystem::FileSystem;

pub struct DevFileSystem {

}

impl FileSystem for DevFileSystem {
  fn open(&self, path: &'static str) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    Err(())
  }

  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Err(())
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    Err(())
  }
}