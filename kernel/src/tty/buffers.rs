use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::buffers::RingBuffer;

const BUFFER_SIZE: usize = 64;

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
    /*
    let output = Vec::with_capacity(BUFFER_SIZE);
    let input = Vec::with_capacity(BUFFER_SIZE);
    for _ in 0..BUFFER_SIZE {
      output.push(0);
      input.push(0);
    }
    let output_box: Box<[u8]> = output.into_boxed_slice();
    let input_box: Box<[u8]> = input.into_boxed_slice();

    let output_slice = output_box.as_ref();
    let input_slice = input_box.as_ref();

    let output_raw_ptr = Box::into_raw(output_box);
    let input_raw_ptr = Box::into_raw(input_box);
    */

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