use alloc::boxed::Box;
use crate::buffers::RingBuffer;

const BUFFER_SIZE: usize = 256;

/// A Pipe is a simple fifo queue of byte data, allowing data to be passed
/// between different processes.
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

  /// Get the number of bytes that have been written, but not yet read
  pub fn available_bytes(&self) -> usize {
    self.data_buffer.available_bytes()
  }

  /// Return true if there are bytes to read
  pub fn can_read(&self) -> bool {
    self.available_bytes() > 0
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