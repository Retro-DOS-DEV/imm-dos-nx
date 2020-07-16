use alloc::vec::Vec;
use crate::process::{all_processes, id::ProcessID, send_signal, yield_coop};
use spin::Mutex;

pub struct BlockQueue {
  queue: Mutex<Vec<ProcessID>>,
}

impl BlockQueue {
  pub fn new() -> BlockQueue {
    BlockQueue {
      queue: Mutex::new(Vec::new()),
    }
  }

  pub fn add_process(&self, id: ProcessID) {
    self.queue.lock().push(id);
  }

  pub fn block_current(&self) {
    let pid = {
      all_processes().get_current_pid()
    };
    self.add_process(pid);
    send_signal(pid, syscall::signals::STOP);
    yield_coop();
  }

  pub fn unblock(&self) {
    let mut queue = self.queue.lock();
    while !queue.is_empty() {
      let pid = queue.pop();
      if let Some(id) = pid {
        send_signal(id, syscall::signals::CONTINUE);
      }
    }
  }
}
