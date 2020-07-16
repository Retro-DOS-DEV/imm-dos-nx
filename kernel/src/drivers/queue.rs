use crate::process::{get_current_pid, id::ProcessID, send_signal, yield_coop};

pub trait ReadQueue {
  /**
   * Push a process onto an internal queue. Implementation details are specific
   * to each struct with this trait.
   */
  fn add_process_to_queue(&self, pid: ProcessID) -> usize;

  fn remove_first_in_queue(&self) -> Option<ProcessID>;

  fn get_queue_length(&self) -> usize;

  fn get_first_process_in_queue(&self) -> Option<ProcessID>;

  /**
   * Implementation-specific method to determine if data is ready to be read to
   * the buffer.
   */
  fn is_data_available(&self) -> bool;

  /**
   * Implementation-specific method to copy ready data to the buffer. Will copy
   * at most `buffer.len()` bytes, if they are available. Returns the number of
   * bytes copied during the current method call.
   */
  fn read_available_data(&self, buffer: &mut [u8]) -> usize;

  /**
   * Make a process wait until data can be read. The default implementation can
   * be overridden if needed.
   */
  fn process_wait(&self, pid: ProcessID) {
    send_signal(pid, syscall::signals::STOP);
    yield_coop();
  }

  fn process_wake(&self, pid: ProcessID) {
    send_signal(pid, syscall::signals::CONTINUE);
  }

  /**
   * Perform a queued blocking read into a buffer. The current process will
   * sleep until it is at the head of the queue, and then will try to read data
   * into the buffer. If at any point data is unavailable, the process will
   * sleep until it has more to read.
   * It is the responsibility of the interrupt or external process determining
   * data availability to re-wake the queue.
   * It may be desirable to replace the STOP call with a timeout, so that the
   * thread can occasionally wake itself if something goes wrong with
   * interrupts. This would also allow implementation of timeouts.
   */
  fn blocking_read(&self, buffer: &mut [u8]) -> usize {
    let current_pid = get_current_pid();
    let len = self.add_process_to_queue(current_pid);
    if len > 1 {
      self.process_wait(current_pid);
    }
    let mut first = self.get_first_process_in_queue();
    while first != Some(current_pid) {
      // Got woken even though it's not first
      self.process_wait(current_pid);
      first = self.get_first_process_in_queue();
    }
    let mut bytes_read = 0;
    let mut left_to_read = buffer.len();
    let mut partial_buffer = buffer;
    while left_to_read > 0 {
      let read_chunk = self.read_available_data(partial_buffer);
      bytes_read += read_chunk;
      left_to_read -= read_chunk;
      partial_buffer = &mut partial_buffer[read_chunk..];
      if left_to_read > 0 && !self.is_data_available() {
        self.process_wait(current_pid);
      }
    }
    self.remove_first_in_queue();
    if self.is_data_available() {
      let first = self.get_first_process_in_queue();
      if let Some(pid) = first {
        self.process_wake(pid);
      }
    }

    bytes_read
  }
}
