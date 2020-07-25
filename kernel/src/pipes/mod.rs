use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::buffers::RingBuffer;
use crate::files::handle::{Handle, LocalHandle};
use spin::RwLock;

const BUFFER_SIZE: usize = 256;

pub struct Pipe {
  /// Pointer to the heap data
  data_raw_ptr: usize,
  /// Ring buffer containing pipe data
  pub data_buffer: RingBuffer<'static>,
}

impl Pipe {
  pub fn new() -> Pipe {
    let data_box: Box<[u8; BUFFER_SIZE]> = Box::new([0; BUFFER_SIZE]);

    let data_raw_ptr = Box::into_raw(data_box);

    let data_slice = unsafe { &*data_raw_ptr };

    Pipe {
      data_raw_ptr: data_raw_ptr as usize,
      data_buffer: RingBuffer::new(data_slice),
    }
  }
}

impl Drop for Pipe {
  fn drop(&mut self) {
    unsafe {
      let ptr = self.data_raw_ptr as *mut [u8; BUFFER_SIZE];
      Box::from_raw(ptr);
    }
  }
}

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
}

pub struct PipeCollection {
  pipes: RwLock<Vec<Option<Pipe>>>,
  handles: RwLock<Vec<Option<PipeHandle>>>,
}

impl PipeCollection {
  pub const fn new() -> PipeCollection {
    PipeCollection {
      pipes: RwLock::new(Vec::new()),
      handles: RwLock::new(Vec::new()),
    }
  }

  fn get_pipe_handle(&self, handle: LocalHandle) -> Option<PipeHandle> {
    let handles = self.handles.read();
    let slot = handles.get(handle.as_usize())?;
    *slot
  }

  fn find_empty_handle_slot(&self) -> usize {
    let mut handles = self.handles.write();
    let mut slot: Option<usize> = None;
    let mut index = 0;
    while index < handles.len() && slot.is_none() {
      if handles.get(index).is_none() {
        slot = Some(index);
      }
      index += 1;
    }
    match slot {
      Some(i) => i,
      None => {
        let last = handles.len();
        handles.push(None);
        last
      },
    }
  }

  /// Create a pipe and a pair of read/write handles
  pub fn create(&self) -> Result<(LocalHandle, LocalHandle), ()> {
    let pipe_index = {
      let mut pipes = self.pipes.write();
      let mut slot: Option<usize> = None;
      let mut index = 0;
      while index < pipes.len() && slot.is_none() {
        if pipes.get(index).is_none() {
          slot = Some(index);
        }
        index += 1;
      }
      let slot_index = match slot {
        Some(i) => i,
        None => {
          let last = pipes.len();
          pipes.push(None);
          last
        },
      };
      pipes[slot_index] = Some(Pipe::new());
      slot_index
    };

    let (read_handle, write_handle) = {
      let read_index = {
        let slot = self.find_empty_handle_slot();
        let mut handles = self.handles.write();
        handles[slot] = Some(PipeHandle::ReadHandle(pipe_index));
        slot
      };
      let write_index = {
        let slot = self.find_empty_handle_slot();
        let mut handles = self.handles.write();
        handles[slot] = Some(PipeHandle::WriteHandle(pipe_index));
        slot
      };
      (LocalHandle::new(read_index as u32), LocalHandle::new(write_index as u32))
    };

    Ok((read_handle, write_handle))
  }


  pub fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let pipe_handle = self.get_pipe_handle(handle).ok_or(())?;
    match pipe_handle {
      PipeHandle::ReadHandle(index) => {
        let pipes = self.pipes.read();
        let slot = pipes.get(index).ok_or(())?;
        match slot {
          Some(pipe) => {
            let read = pipe.data_buffer.read(buffer);
            Ok(read)
          },
          None => Err(()),
        }
      },
      _ => Err(())
    }
  }

  pub fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    let pipe_handle = self.get_pipe_handle(handle).ok_or(())?;
    match pipe_handle {
      PipeHandle::WriteHandle(index) => {
        let pipes = self.pipes.read();
        let slot = pipes.get(index).ok_or(())?;
        match slot {
          Some(pipe) => {
            let written = pipe.data_buffer.write(buffer);
            Ok(written)
          },
          None => Err(()),
        }
      },
      _ => Err(())
    }
  }
}

pub static PIPES: PipeCollection = PipeCollection::new();
