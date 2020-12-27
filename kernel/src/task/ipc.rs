use alloc::collections::VecDeque;
use super::id::ProcessID;

/// IPC is implemented by passing a simple tuple of u32 values from one process
/// to another.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IPCMessage(pub u32, pub u32, pub u32, pub u32);

/// A packet associates an IPC message with its sender.
#[derive(Debug, Eq, PartialEq)]
pub struct IPCPacket {
  pub from: ProcessID,
  pub message: IPCMessage,
}

/// For storing IPC messages in a process's receiving queue, each message is
/// associated with an expiration time. The time is recorded in system ticks,
/// and indicates the time after which this entry is no longer valid.
/// Expiration is used to keep the queue from growing too large. Rather than
/// update all process queues whenever system time is incremented, the kernel
/// only checks for expired items when the queue is accessed.
pub struct EnqueuedIPC {
  pub packet: IPCPacket,
  pub expiration_ticks: u32,
}

/// Each process has an IPCQueue which stores messages that have been sent to
/// the process.
pub struct IPCQueue {
  queue: VecDeque<EnqueuedIPC>,
}

impl IPCQueue {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
    }
  }

  fn remove_expired_entries(&mut self, current_ticks: u32) {
    while let Some(entry) = self.queue.front() {
      if entry.expiration_ticks > current_ticks {
        return;
      }
      self.queue.pop_front();
    }
  }

  /// Add a message from another process.
  pub fn add(&mut self, from: ProcessID, message: IPCMessage, current_ticks: u32, expiration_ticks: u32) {
    self.remove_expired_entries(current_ticks);
    let for_queue = EnqueuedIPC {
      packet: IPCPacket {
        from,
        message,
      },
      expiration_ticks,
    };
    self.queue.push_back(for_queue);
  }

  /// Attempt to read a packet from the message queue. The first parameter of
  /// the return value is an option that may contain a packet if one exists. The
  /// second parameter is a boolean reflecting whether there are more packets
  /// to read.
  pub fn read(&mut self, current_ticks: u32) -> (Option<IPCPacket>, bool) {
    self.remove_expired_entries(current_ticks);
    let message = self.queue.pop_front().map(|entry| entry.packet);
    let has_more = !self.queue.is_empty();
    (message, has_more)
  }
}

#[cfg(test)]
mod tests {
  use super::{IPCMessage, IPCPacket, IPCQueue, ProcessID};

  #[test]
  fn add_and_read() {
    let mut queue = IPCQueue::new();
    {
      let (front, remaining) = queue.read(0);
      assert!(front.is_none());
      assert!(!remaining);
    }
    queue.add(
      ProcessID::new(10),
      IPCMessage(1, 2, 3, 4),
      0,
      2000,
    );
    queue.add(
      ProcessID::new(14),
      IPCMessage(5, 6, 7, 8),
      0,
      2000,
    );
    {
      let (front, remaining) = queue.read(0);
      assert_eq!(front.unwrap(), IPCPacket {
        from: ProcessID::new(10),
        message: IPCMessage(1, 2, 3, 4),
      });
      assert!(remaining);
    }
    {
      let (front, remaining) = queue.read(0);
      assert_eq!(front.unwrap(), IPCPacket {
        from: ProcessID::new(14),
        message: IPCMessage(5, 6, 7, 8),
      });
      assert!(!remaining);
    }
  }

  #[test]
  fn expiration() {
    let mut queue = IPCQueue::new();
    queue.add(
      ProcessID::new(10),
      IPCMessage(1, 2, 3, 4),
      0,
      2000,
    );
    queue.add(
      ProcessID::new(12),
      IPCMessage(5, 6, 7, 8),
      3000,
      5000,
    );
    {
      let (front, remaining) = queue.read(4000);
      assert_eq!(front.unwrap(), IPCPacket {
        from: ProcessID::new(12),
        message: IPCMessage(5, 6, 7, 8),
      });
      assert!(!remaining);
    }
  }
}
