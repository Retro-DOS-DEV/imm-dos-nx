use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use crate::buffers::RingBuffer;
use crate::collections::SlotList;
use crate::devices::driver::IOHandle;
use crate::devices::queue::QueuedIO;
use crate::task::id::ProcessID;
use spin::RwLock;

const BUFFER_SIZE: usize = 512;

/// Ring buffers for reading and writing to a TTY device file
pub struct TTYReadWriteBuffers {
  /// Pointers to allocated buffer data. These objects get leaked to get around
  /// lifetime constraints. Keeping pointers let us dealloc them when this
  /// struct is dropped.
  output_raw_ptr: *mut [u8; BUFFER_SIZE],
  input_raw_ptr: *mut [u8; BUFFER_SIZE],
  /// Ring buffer for data sent from the TTY to readers
  pub output_buffer: RingBuffer<'static>,
  /// Ring buffer containing data written to the TTY
  pub input_buffer: RingBuffer<'static>,
}

impl TTYReadWriteBuffers {
  pub fn new() -> TTYReadWriteBuffers {
    let output_box: Box<[u8; BUFFER_SIZE]> = Box::new([0; BUFFER_SIZE]);
    let input_box: Box<[u8; BUFFER_SIZE]> = Box::new([0; BUFFER_SIZE]);

    let output_raw_ptr = Box::into_raw(output_box);
    let input_raw_ptr = Box::into_raw(input_box);

    let output_slice = unsafe { &*output_raw_ptr };
    let input_slice = unsafe { &*input_raw_ptr };

    TTYReadWriteBuffers {
      output_raw_ptr,
      input_raw_ptr,
      output_buffer: RingBuffer::new(output_slice),
      input_buffer: RingBuffer::new(input_slice),
    }
  }

  pub fn read(&self, buffer: &mut [u8]) -> usize {
    self.output_buffer.read(buffer)
  }

  pub fn write(&self, buffer: &[u8]) -> usize {
    self.input_buffer.write(buffer)
  }
}

impl Drop for TTYReadWriteBuffers {
  fn drop(&mut self) {
    Box::from(self.output_raw_ptr);
    Box::from(self.input_raw_ptr);
  }
}


pub struct Descriptor {
  pub process: ProcessID,
  pub handle: IOHandle,
}

/// Stores input from the keyboard that is waiting to be read
pub struct TTYReaderBuffer {
  /// Pointer to allocated buffer data. The buffer gets leaked to get around
  /// lifetime constraints. Keeping a pointer let us dealloc it when this
  /// struct is dropped.
  buffer_raw_ptr: *mut [u8; BUFFER_SIZE],
  /// Ring buffer for data sent from the TTY to readers
  pub buffer: RingBuffer<'static>,

  open_handles: Arc<RwLock<SlotList<Descriptor>>>,
  io_queue: RwLock<VecDeque<IOHandle>>,
}

impl TTYReaderBuffer {
  pub fn new(open_handles: Arc<RwLock<SlotList<Descriptor>>>) -> Self {
    let buffer_box: Box<[u8; BUFFER_SIZE]> = Box::new([0; BUFFER_SIZE]);
    let buffer_raw_ptr = Box::into_raw(buffer_box);
    let buffer_slice = unsafe { &*buffer_raw_ptr };

    Self {
      buffer_raw_ptr,
      buffer: RingBuffer::new(buffer_slice),
      io_queue: RwLock::new(VecDeque::new()),
      open_handles,
    }
  }

  pub fn read(&self, handle: IOHandle, dest: &mut [u8]) -> usize {
    self.perform_io(handle, || {
      let mut bytes_read = 0;
      let mut byte_buffer: [u8; 1] = [0];
      while bytes_read < dest.len() && byte_buffer[0] != b'\n' {
        if self.buffer.available_bytes() < 1 {
          crate::task::get_current_process().write().io_block(None);
          crate::task::yield_coop();
        }
        let partial_read = self.buffer.read(&mut byte_buffer);
        if partial_read > 0 {
          if byte_buffer[0] == 8 /* and in canonical mode */ {
            // handle backspace
            if bytes_read > 0 {
              bytes_read -= 1;
            }
          } else {
            dest[bytes_read] = byte_buffer[0];
            bytes_read += partial_read;
          }
        }
      }
      bytes_read
    })
  }

  pub fn add_data(&self, data: &[u8]) {
    self.buffer.write(&data);
    self.wake_front();
  }
}

impl Drop for TTYReaderBuffer {
  fn drop(&mut self) {
    Box::from(self.buffer_raw_ptr);
  }
}

impl QueuedIO<(), usize> for TTYReaderBuffer {
  fn get_process_id_for_handle(&self, handle: IOHandle) -> Option<ProcessID> {
    self.open_handles
      .read()
      .iter()
      .find_map(|o| if o.handle == handle { Some(o.process) } else { None } )
  }

  fn get_io_queue(&self) -> &RwLock<VecDeque<IOHandle>> {
    &self.io_queue
  }
}


pub struct TTYWriterBuffer {
  buffer_raw_ptr: *mut [u8; BUFFER_SIZE],
  pub buffer: RingBuffer<'static>,
}

impl TTYWriterBuffer {
  pub fn new() -> Self {
    let buffer_box: Box<[u8; BUFFER_SIZE]> = Box::new([0; BUFFER_SIZE]);
    let buffer_raw_ptr = Box::into_raw(buffer_box);
    let buffer_slice = unsafe { &*buffer_raw_ptr };

    Self {
      buffer_raw_ptr,
      buffer: RingBuffer::new(buffer_slice),
    }
  }

  pub fn write(&self, _handle: IOHandle, buffer: &[u8]) -> usize {
    // TODO: handle partial / blocked write
    self.buffer.write(buffer)
  }

  pub fn read(&self, buffer: &mut [u8]) -> usize {
    self.buffer.read(buffer)
  }

  pub fn available_bytes(&self) -> usize {
    self.buffer.available_bytes()
  }
}

impl Drop for TTYWriterBuffer {
  fn drop(&mut self) {
    Box::from(self.buffer_raw_ptr);
  }
}