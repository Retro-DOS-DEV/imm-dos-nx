use alloc::sync::Arc;
use crate::process;

pub mod exec;
pub mod file;
pub mod fs;

fn current_process() -> Arc<process::process_state::ProcessState> {
  process::current_process().expect("Running a syscall for an unknown process")
}
