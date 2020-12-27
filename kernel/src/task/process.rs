use crate::memory::address::VirtualAddress;
use super::id::ProcessID;
use super::ipc::{IPCMessage, IPCPacket, IPCQueue};
use super::memory::MemoryRegions;
use super::state::RunState;

pub struct Process {
  /// The unique ID of this process
  id: ProcessID,
  /// The ID of the parent process
  parent_id: ProcessID,
  /// Stores the details of all addresses mapped into the process's memory.
  /// When a page fault occurs, this information is used to determine how
  /// content is paged into memory, or if it's a crash-causing fault.
  memory: MemoryRegions,
  /// Represents the current execution state of the process
  state: RunState,
  /// The number of system ticks when this process was started
  start_ticks: u32,
  /// Stores IPC messages that have been sent to this process
  ipc_queue: IPCQueue,
}

impl Process {
  /// Generate the init process
  pub fn initial(current_ticks: u32) -> Self {
    Self {
      id: ProcessID::new(1),
      parent_id: ProcessID::new(1),
      memory: MemoryRegions::new(),
      state: RunState::Running,
      start_ticks: current_ticks,
      ipc_queue: IPCQueue::new(),
    }
  }

  pub fn get_id(&self) -> &ProcessID {
    &self.id
  }

  pub fn get_parent_id(&self) -> &ProcessID {
    &self.parent_id
  }

  /// Based on the current system time in ticks, how long has this process been
  /// running?
  pub fn uptime_ticks(&self, current_ticks: u32) -> u32 {
    current_ticks - self.start_ticks
  }

  /// Determine if the scheduler can re-enter this process
  pub fn can_resume(&self) -> bool {
    match self.state {
      RunState::Running | RunState::Resumed(_) => true,
      _ => false,
    }
  }

  /// End all execution of the process, and mark its resources for cleanup.
  pub fn terminate(&mut self) {
    self.state = RunState::Terminated;
  }

  /// Pause this process for a specified number of milliseconds. When the
  /// duration has passed, the process's state will return to Running.
  pub fn sleep(&mut self, duration: usize) {
    self.state = RunState::Sleeping(duration);
  }

  /// Pause the process due to a signal. It will not resume until woken by
  /// a different signal.
  pub fn pause(&mut self) {
    self.state = RunState::Paused;
  }

  /// Resume the process due to a signal. If the process is not explicitly
  /// paused, this is a no-op.
  pub fn resume(&mut self) {
    match self.state {
      RunState::Paused => self.state = RunState::Running,
      _ => (),
    }
  }

  /// Tell a process that a child has exited. If the process is currently
  /// waiting on that child, it will resume execution.
  pub fn child_returned(&mut self, child_id: ProcessID, code: u32) {
    let waiting_on = match self.state {
      RunState::WaitingForChild(id) => id,
      _ => return,
    };
    if child_id == waiting_on {
      self.state = RunState::Resumed(code);
    }
  }

  /// Attempt to read an IPC message. If none is available, the process will
  /// block until a message is received or the optional timeout argument
  /// expires.
  /// Because entries in the IPC queue are only expired when it is read or
  /// written, the current time needs to be passed to this method to clean up
  /// any items that are due for removal.
  pub fn ipc_read(&mut self, current_ticks: u32, timeout: Option<usize>) -> (Option<IPCPacket>, bool) {
    let (first_read, has_more) = self.ipc_queue.read(current_ticks);
    if first_read.is_some() {
      return (first_read, has_more);
    }
    // Nothing in the queue, block the process until something arrives
    self.state = RunState::AwaitingIPC(timeout);
    super::yield_coop();
    // At this point, either a message was enqueued or the timeout expired
    // `current_ticks` will be outdated, but we don't care because if there is
    // an entry, it hasn't expired.
    self.ipc_queue.read(current_ticks)
  }

  /// Send an IPC message to this process. If the process is currently blocked
  /// on reading the IPC queue, it will wake up.
  /// Each message is accompanied by an expiration time (in system ticks), after
  /// which point the message will be considered invalid if it hasn't been read.
  pub fn ipc_receive(&mut self, current_ticks: u32, from: ProcessID, message: IPCMessage, expiration_ticks: u32) {
    self.ipc_queue.add(from, message, current_ticks, expiration_ticks);
    match self.state {
      RunState::AwaitingIPC(_) => {
        self.state = RunState::Running;
      },
      _ => (),
    }
  }

  /// Update any internal timers based on regular system clock updates.
  pub fn update_timeouts(&mut self, delta_ms: usize) {
    match self.state {
      RunState::AwaitingIPC(Some(timeout)) => {
        self.state = if timeout < delta_ms {
          RunState::Running
        } else {
          RunState::AwaitingIPC(Some(timeout - delta_ms))
        };
      },
      RunState::Sleeping(timeout) => {
        self.state = if timeout < delta_ms {
          RunState::Running
        } else {
          RunState::Sleeping(timeout - delta_ms)
        };
      },
      _ => (),
    }
  }

  /// Increase the process heap by a specific number of bytes. The old heap
  /// endpoint will be returned.
  pub fn increase_heap(&mut self, increment: usize) -> VirtualAddress {
    let start = self.memory.get_heap_start();
    let prev_size = self.memory.get_heap_size();
    self.memory.set_heap_size(prev_size + increment);
    start + prev_size
  }
}

#[cfg(test)]
mod tests {
  use super::{Process, VirtualAddress};

  #[test]
  fn sleeping() {
    let mut p = Process::initial(0);
    p.sleep(2000);
    assert!(!p.can_resume());
    p.update_timeouts(500);
    p.update_timeouts(1000);
    assert!(!p.can_resume());
    p.update_timeouts(700);
    assert!(p.can_resume());
  }

  #[test]
  fn heap_modification() {
    let mut p = Process::initial(0);
    // just put the heap endpoint somewhere
    p.increase_heap(0x250);
    // simulate `brk`
    {
      let prev = p.increase_heap(0);
      p.increase_heap(VirtualAddress::new(0x1200) - prev);
      assert_eq!(p.memory.get_heap_start() + p.memory.get_heap_size(), VirtualAddress::new(0x1200));
    }
    // simulate `sbrk`
    {
      let prev = p.increase_heap(0);
      assert_eq!(prev, p.increase_heap(0x430));
      assert_eq!(prev + 0x430, p.memory.get_heap_start() + p.memory.get_heap_size());
    }
  }
}
