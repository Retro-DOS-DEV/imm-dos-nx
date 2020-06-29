use crate::process;

pub fn yield_coop() {
  process::yield_coop();
}

pub fn sleep(ms: u32) {
  process::sleep(ms as usize)
}

pub fn fork() -> u32 {
  process::fork()
}