use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
use crate::memory::address::VirtualAddress;
use super::id::ProcessID;
use super::process_state::ProcessState;

/**
 * Mapping of PIDs to process structures.
 * Also keeps track of the "current" process.
 */
pub struct ProcessMap {
  current: ProcessID,
  next_pid: AtomicU32,
  processes: BTreeMap<ProcessID, Arc<ProcessState>>,
}

impl ProcessMap {
  pub fn new() -> ProcessMap {
    ProcessMap {
      current: ProcessID::new(0),
      next_pid: AtomicU32::new(1),
      processes: BTreeMap::new(),
    }
  }

  pub fn get_next_pid(&self) -> ProcessID {
    let pid = self.next_pid.fetch_add(1, Ordering::SeqCst);
    ProcessID::new(pid)
  }

  pub fn iter(&self) -> alloc::collections::btree_map::Iter<ProcessID, Arc<ProcessState>> {
    self.processes.iter()
  }

  pub fn get_next_running_process(&self) -> ProcessID {
    let mut first = None;
    for (pid, process) in self.processes.iter() {
      if process.is_running() {
        if first.is_none() {
          first = Some(*pid);
        }
        if *pid > self.current {
          return *pid;
        }
      }
    }
    // If we hit the end of the list, loop back to the first running process
    // we found. If there is none, we stay on the current process.
    match first {
      Some(pid) => pid,
      None => self.current,
    }
  }
  

  pub fn spawn_first_process(&mut self, heap_location: VirtualAddress) -> ProcessID {
    let pid = self.get_next_pid();
    self.processes.insert(
      pid,
      Arc::new(ProcessState::first(pid, heap_location))
    );
    pid
  }

  pub fn fork_current(&mut self) -> ProcessID {
    let pid = self.get_next_pid();
    let cur = self.get_current_process().expect("No current process to fork");
    let next = cur.fork(pid);
    self.processes.insert(
      pid,
      Arc::new(next),
    );
    pid
  }

  pub fn get_process(&self, pid: ProcessID) -> Option<&Arc<ProcessState>> {
    self.processes.get(&pid)
  }

  pub fn get_current_process(&self) -> Option<&Arc<ProcessState>> {
    self.processes.get(&self.current)
  }

  pub fn get_current_pid(&self) -> ProcessID {
    self.current
  }

  pub fn make_current(&mut self, pid: ProcessID) {
    self.current = pid;
  }
}