/// A file handle for a pipe can be a read-only handle, or a write-only handle
#[derive(Copy, Clone)]
pub enum PipeHandle {
  ReadHandle(usize),
  WriteHandle(usize),
}

impl PipeHandle {
  pub fn can_read(&self) -> bool {
    match self {
      PipeHandle::ReadHandle(_) => true,
      PipeHandle::WriteHandle(_) => false,
    }
  }

  pub fn can_write(&self) -> bool {
    match self {
      PipeHandle::WriteHandle(_) => true,
      PipeHandle::ReadHandle(_) => false,
    }
  }

  pub fn to_index(&self) -> usize {
    match self {
      PipeHandle::ReadHandle(i) => *i,
      PipeHandle::WriteHandle(i) => *i,
    }
  }
}