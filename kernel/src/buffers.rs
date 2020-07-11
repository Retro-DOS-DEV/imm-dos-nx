use core::sync::atomic::{AtomicUsize, Ordering};

/**
 * Simple single-producer, single-consumer ring buffer
 */
pub struct RingBuffer<'data> {
  head: AtomicUsize,
  tail: AtomicUsize,
  data: &'data [u8],
}

impl<'data> RingBuffer<'data> {
  pub const fn new(data: &'data [u8]) -> RingBuffer<'data> {
    RingBuffer {
      head: AtomicUsize::new(0),
      tail: AtomicUsize::new(0),
      data,
    }
  }

  pub fn read(&self, dest: &mut [u8]) -> usize {
    let mut to_read = dest.len();
    let len = self.data.len();
    let tail = self.tail.load(Ordering::SeqCst);
    let head = self.head.load(Ordering::SeqCst);
    let available_len = tail - head;
    if available_len < to_read {
      to_read = available_len;
    }
    unsafe {
      let data_ptr = self.data.as_ptr();
      for i in 0..to_read {
        let ptr = data_ptr.offset(((head + i) % len) as isize);
        dest[i] = *ptr;
      }
    }
    self.head.fetch_add(to_read, Ordering::SeqCst);
    to_read
  }

  pub fn write(&self, src: &[u8]) -> usize {
    let mut to_write = src.len();
    let len = self.data.len();
    let tail = self.tail.load(Ordering::SeqCst);
    let head = self.head.load(Ordering::SeqCst);
    let available_room = head + len - tail;
    if available_room < to_write {
      to_write = available_room;
    }
    unsafe {
      let data_ptr = self.data.as_ptr() as *mut u8;
      for i in 0..to_write {
        let ptr = data_ptr.offset(((tail + i) % len) as isize);
        *ptr = src[i];
      }
    }
    self.tail.fetch_add(to_write, Ordering::SeqCst);
    to_write
  }

  pub fn available_bytes(&self) -> usize {
    let tail = self.tail.load(Ordering::SeqCst);
    let head = self.head.load(Ordering::SeqCst);
    tail - head
  }
}