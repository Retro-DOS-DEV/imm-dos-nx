use crate::process::{self, id::ProcessID};

/**
 * The Input thread runs with kernel-level permissions, and sleeps until an
 * input interrupt occurs (keyboard, COM). In order to complete quickly, input
 * interrupts push data onto a queue and return without further processing.
 * The Input thread checks this queue whenever it is awake, and 
 */

pub static mut INPUT_THREAD_ID: ProcessID = ProcessID::new(0);

#[inline(never)]
pub extern "C" fn run_input() {
  unsafe {
    INPUT_THREAD_ID = process::get_current_pid();
  }
  crate::kprintln!("INPUT THREAD REPORTING!");
  loop {
    process::send_signal(unsafe { INPUT_THREAD_ID }, syscall::signals::STOP);
    process::yield_coop();
    crate::kprintln!("INPUT CHECK");
  }
}