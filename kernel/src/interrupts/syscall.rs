use crate::kprintln;

pub extern "C" fn syscall_handler() {
  kprintln!("A Syscall!");
}