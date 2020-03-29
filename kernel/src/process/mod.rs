use alloc::sync::Arc;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod map;
pub mod process_state;

static mut PROCESS_MAP: Option<RwLock<map::ProcessMap>> = None;

pub fn init() {
  unsafe {
    PROCESS_MAP = Some(RwLock::new(map::ProcessMap::new()));
  }
}

pub fn all_processes() -> RwLockReadGuard<'static, map::ProcessMap> {
  unsafe {
    match &PROCESS_MAP {
      Some(lock) => lock.read(),
      None => {
        panic!("Process Map not initialized");
      }
    }
  }
}

pub fn all_processes_mut() -> RwLockWriteGuard<'static, map::ProcessMap> {
  unsafe {
    match &PROCESS_MAP {
      Some(lock) => lock.write(),
      None => {
        panic!("Process Map not initialized");
      }
    }
  }
}

pub fn current_process() -> Option<Arc<process_state::ProcessState>> {
  let map = all_processes();
  match map.get_current_process() {
    Some(p) => Some(p.clone()),
    None => None,
  }
}

pub fn make_current(pid: process_state::ProcessID) {
  let mut map = all_processes_mut();
  map.make_current(pid);
}