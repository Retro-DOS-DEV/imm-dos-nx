use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::task::id::ProcessID;
#[cfg(not(test))]
use crate::task::switching::get_process;

const BUFFER_LENGTH: usize = 32;

/// When a process attempts to read an input device, an InputBuffer is
/// constructed. When the process makes a read request, the buffer is marked as
/// active and incoming data will be enqueued. When data is added to the buffer,
/// it wakes up the process and frees it to read the new values.
pub struct InputBuffer {
  process: ProcessID,
  data: [u8; BUFFER_LENGTH],
  head: AtomicUsize,
  tail: AtomicUsize,
  is_reading: AtomicBool,
}

impl InputBuffer {
  pub fn for_process(id: ProcessID) -> Self {
    Self {
      process: id,
      data: [0; BUFFER_LENGTH],
      head: AtomicUsize::new(0),
      tail: AtomicUsize::new(0),
      is_reading: AtomicBool::new(false),
    }
  }

  pub fn get_process_id(&self) -> ProcessID {
    self.process
  }

  pub fn start_read(&self) {
    self.is_reading.store(true, Ordering::SeqCst);
    self.head.store(0, Ordering::SeqCst);
    self.tail.store(0, Ordering::SeqCst);
  }

  pub fn write_pair(&self, pair: [u8; 2]) {
    if !self.is_reading.load(Ordering::SeqCst) {
      return;
    }
    let current_head = self.head.load(Ordering::SeqCst);
    let current_tail = self.tail.load(Ordering::SeqCst);
    if current_tail - current_head >= self.data.len() - 2 {
      // buffer is full
      return;
    }
    self.tail.store(current_tail + 2, Ordering::SeqCst);
    let location = current_tail & (BUFFER_LENGTH - 1);
    unsafe {
      let data = core::slice::from_raw_parts_mut(
        &self.data[0] as *const u8 as *mut u8,
        self.data.len(),
      );
      data[location] = pair[0];
      data[location + 1] = pair[1];
    }

    #[cfg(not(test))]
    {
      match get_process(&self.process) {
        Some(locked) => locked.write().io_resume(),
        None => (),
      }
    }
  }

  pub fn read_to_buffer(&self, buffer: &mut [u8]) -> usize {
    let current_head = self.head.load(Ordering::SeqCst);
    let current_tail = self.tail.load(Ordering::SeqCst);
    let available_length = current_tail - current_head;
    if available_length == 0 {
      return 0;
    }
    let writing_length = buffer.len().min(available_length);
    let writing_start = current_head & (BUFFER_LENGTH - 1);
    for i in 0..writing_length {
      let index = (writing_start + i) & (BUFFER_LENGTH - 1);
      buffer[i] = self.data[index];
    }
    self.head.store(current_head + writing_length, Ordering::SeqCst);
    writing_length
  }

  pub fn bytes_available(&self) -> usize {
    let current_head = self.head.load(Ordering::SeqCst);
    let current_tail = self.tail.load(Ordering::SeqCst);
    current_tail - current_head
  }
}

#[cfg(test)]
mod tests {
  use super::{InputBuffer, ProcessID};

  #[test]
  fn maximum_length() {
    let buffer = InputBuffer::for_process(ProcessID::new(1));
    buffer.start_read();
    for i in 0..20 {
      buffer.write_pair([i * 2, i * 2 + 1]);
    }
    let mut read_buffer: [u8; 32] = [0; 32];
    assert_eq!(buffer.read_to_buffer(&mut read_buffer), 30);
    for i in 0..30 {
      assert_eq!(read_buffer[i], i as u8);
    }
  }

  #[test]
  fn read_write() {
    let buffer = InputBuffer::for_process(ProcessID::new(1));
    buffer.start_read();
    for i in 0..5 {
      buffer.write_pair([i * 2, i * 2 + 1]);
    }
    let mut read_buffer: [u8; 30] = [0; 30];
    assert_eq!(buffer.read_to_buffer(&mut read_buffer[0..6]), 6);
    for i in 0..6 {
      assert_eq!(read_buffer[i], i as u8);
    }
    for i in 0..4 {
      buffer.write_pair([i * 2, i * 2 + 1]);
    }
    assert_eq!(buffer.read_to_buffer(&mut read_buffer[0..4]), 4);
    for i in 0..4 {
      assert_eq!(read_buffer[i], i as u8 + 6);
    }
    assert_eq!(buffer.read_to_buffer(&mut read_buffer), 8);
    for i in 0..8 {
      assert_eq!(read_buffer[i], i as u8);
    }
  }
}
