use crate::collections::SlotList;
use crate::files::handle::{Handle, LocalHandle};
use spin::RwLock;
use super::{Pipe, PipeError, PipeHandle};

pub struct PipeCollection {
  pipes: RwLock<SlotList<Pipe>>,
  handles: RwLock<SlotList<PipeHandle>>,
}

impl PipeCollection {
  pub const fn new() -> PipeCollection {
    PipeCollection {
      pipes: RwLock::new(SlotList::new()),
      handles: RwLock::new(SlotList::new()),
    }
  }

  /// Create a pipe and a pair of read/write handles
  pub fn create(&self) -> Result<(LocalHandle, LocalHandle), PipeError> {
    let pipe_index = {
      let mut pipes = self.pipes.write();
      pipes.insert(Pipe::new())
    };
    let (read_handle, write_handle) = {
      let mut handles = self.handles.write();
      let read_index = handles.insert(PipeHandle::ReadHandle(pipe_index));
      let write_index = handles.insert(PipeHandle::WriteHandle(pipe_index));
      (LocalHandle::new(read_index as u32), LocalHandle::new(write_index as u32))
    };
    Ok((read_handle, write_handle))
  }

  /// Read available bytes into a mutable slice, using a Pipe Read Handle.
  /// Returns the number of bytes copied to the buffer.
  pub fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, PipeError> {
    let pipe_handle = {
      let handles = self.handles.read();
      *handles.get(handle.as_usize()).ok_or(PipeError::InvalidHandle)?
    };
    match pipe_handle {
      PipeHandle::ReadHandle(index) => {
        let pipes = self.pipes.read();
        let pipe = pipes.get(index).ok_or(PipeError::UnknownPipe)?;
        let read = pipe.data_buffer.read(buffer);
        Ok(read)
      },
      PipeHandle::WriteHandle(_) => Err(PipeError::WrongHandleType),
    }
  }

  /// Write bytes from a slice into the pipe, using a Pipe Write Handle.
  /// Returns the number of bytes copied to the pipe.
  pub fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, PipeError> {
    let pipe_handle = {
      let handles = self.handles.read();
      *handles.get(handle.as_usize()).ok_or(PipeError::InvalidHandle)?
    };
    match pipe_handle {
      PipeHandle::WriteHandle(index) => {
        let pipes = self.pipes.read();
        let pipe = pipes.get(index).ok_or(PipeError::UnknownPipe)?;
        let written = pipe.data_buffer.write(buffer);
        Ok(written)
      },
      PipeHandle::ReadHandle(_) => Err(PipeError::WrongHandleType),
    }
  }

  pub fn get_available_bytes(&self, handle: LocalHandle) -> Result<usize, PipeError> {
    let pipe_handle = {
      let handles = self.handles.read();
      *handles.get(handle.as_usize()).ok_or(PipeError::InvalidHandle)?
    };
    match pipe_handle {
      PipeHandle::ReadHandle(index) => {
        let pipes = self.pipes.read();
        let pipe = pipes.get(index).ok_or(PipeError::UnknownPipe)?;
        Ok(pipe.available_bytes())
      },
      PipeHandle::WriteHandle(_) => Err(PipeError::WrongHandleType),
    }
  }
}