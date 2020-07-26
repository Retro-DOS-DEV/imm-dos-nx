use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::files::handle::LocalHandle;
use crate::filesystems::FileSystemType;

pub mod collection;
pub mod errors;
pub mod fs;
pub mod handle;
pub mod pipe;

pub use errors::PipeError;
pub use handle::PipeHandle;
pub use pipe::Pipe;

use collection::PipeCollection;

static mut PIPES: Option<Arc<PipeCollection>> = None;

pub fn create_fs() -> Box<FileSystemType> {
  unsafe {
    let pipes = Arc::new(PipeCollection::new());
    let pipe_fs = Box::new(fs::PipeFileSystem::new(&pipes));
    PIPES = Some(pipes);
    pipe_fs
  }
}

pub fn create_pipe() -> Result<(LocalHandle, LocalHandle), PipeError> {
  let pipes = match unsafe { &PIPES } {
    Some(pipes) => pipes,
    None => panic!("Pipe collection was not created"),
  };
  pipes.create()
}
