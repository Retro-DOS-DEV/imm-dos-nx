use alloc::sync::Arc;
use crate::files::handle::LocalHandle;
use crate::filesystems::filesystem::FileSystem;
use super::collection::PipeCollection;

pub struct PipeFileSystem {
  collection: Arc<PipeCollection>,
}

impl PipeFileSystem {
  pub fn new(collection: &Arc<PipeCollection>) -> PipeFileSystem {
    PipeFileSystem {
      collection: Arc::clone(collection),
    }
  }
}

impl FileSystem for PipeFileSystem {
  /// Open only works for named pipes, which are not yet implemented
  fn open(&self, path: &str) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    self.collection.read(handle, buffer).map_err(|_| ())
  }

  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    self.collection.write(handle, buffer).map_err(|_| ())
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    Err(())
  }

  fn dup(&self, handle: LocalHandle) -> Result<LocalHandle, ()> {
    Err(())
  }
}