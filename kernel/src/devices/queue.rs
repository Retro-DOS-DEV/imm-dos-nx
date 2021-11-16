use alloc::collections::VecDeque;
use crate::task::id::ProcessID;
use crate::task::{get_process, get_current_process, yield_coop};
use spin::RwLock;
use super::driver::IOHandle;

pub trait QueuedIO<T, IOResult> {
  fn get_process_id_for_handle(&self, handle: IOHandle) -> Option<ProcessID>;
  fn get_io_queue(&self) -> &RwLock<VecDeque<IOHandle>>;

  fn add_to_queue(&self, handle: IOHandle) -> usize {
    let mut queue = self.get_io_queue().write();
    queue.push_back(handle);
    queue.len()
  }

  fn wake_front(&self) {
    let next: Option<IOHandle> = self.get_io_queue().read().front().copied();
    let next_lock = next
      .and_then(|handle| self.get_process_id_for_handle(handle))
      .and_then(|id| get_process(&id));
    if let Some(lock) = next_lock {
      lock.write().io_resume();
    }
  }

  fn perform_io<F>(&self, handle: IOHandle, f: F) -> IOResult
    where F: FnOnce() -> IOResult {
    let queued = self.add_to_queue(handle);
    if queued > 1 {
      // if there are already others in line, block until the process is first
      get_current_process().write().io_block(None);
      yield_coop();
    }

    let result = f();

    self.get_io_queue().write().pop_front();
    self.wake_front();
    result
  }
}