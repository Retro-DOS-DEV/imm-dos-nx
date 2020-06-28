use core::cmp;
use core::fmt;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ProcessID(u32);

impl ProcessID {
  pub fn new(id: u32) -> ProcessID {
    ProcessID(id)
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
