use core::cmp;
use core::fmt;
use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ProcessID(u32);

impl ProcessID {
  pub const fn new(id: u32) -> ProcessID {
    ProcessID(id)
  }

  pub fn as_u32(&self) -> u32 {
    self.0
  }
}

impl cmp::Ord for ProcessID {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for ProcessID {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for ProcessID {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Eq for ProcessID {}

impl fmt::Debug for ProcessID {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "PID({})", self.0)
  }
}

pub struct IDGenerator(AtomicU32);

impl IDGenerator {
  pub const fn new() -> Self {
    Self(AtomicU32::new(0))
  }

  pub fn next(&self) -> ProcessID {
    let id = self.0.fetch_add(1, Ordering::SeqCst);
    ProcessID::new(id)
  }
}
