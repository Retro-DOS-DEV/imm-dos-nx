use crate::process::{id::ProcessID, send_signal};
use spin::RwLock;

/// A WakeReference stores a process that should be woken from sleep when an
/// event like an interrupt occurs. It provides some utilities to simplify
/// implementation of broader blocking methods that are concurrent-safe.
pub struct WakeReference {
  pid: RwLock<Option<ProcessID>>,
}

impl WakeReference {
  pub const fn new() -> WakeReference {
    WakeReference {
      pid: RwLock::new(None),
    }
  }

  /// Set the internal reference to the process specified in `pid` only if there
  /// is no reference already set. This is useful for methods where multiple
  /// entry calls may be running in parallel.
  pub fn maybe_set_process(&self, pid: ProcessID) {
    let mut pid_ref = self.pid.write();
    if let None = *pid_ref {
      *pid_ref = Some(pid);
    }
  }

  /// Force the internal reference to point to the process specified in `pid`.
  /// This overrides any previous value without waking it.
  pub fn set_process(&self, pid: ProcessID) {
    let mut pid_ref = self.pid.write();
    *pid_ref = Some(pid);
  }

  /// Remove any reference to a process
  pub fn clear_process(&self) {
    let mut pid_ref = self.pid.write();
    *pid_ref = None;
  }

  /// Wake up the process, if one is referenced
  pub fn wake(&self) {
    let pid_ref = *self.pid.read();
    if let Some(pid) = pid_ref {
      send_signal(pid, syscall::signals::CONTINUE);
    }
  }
}
