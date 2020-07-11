use crate::process::{self, id::ProcessID};

/**
 * The Input thread runs with kernel-level permissions, and sleeps until an
 * input interrupt occurs (keyboard, COM). In order to complete quickly, input
 * interrupts push data onto a queue and return without further processing.
 * The Input thread checks this queue whenever it is awake, and 
 */

pub static mut INPUT_THREAD_ID: ProcessID = ProcessID::new(0);

#[naked]
#[inline(never)]
pub fn run_input() {
  unsafe {
    llvm_asm!("1: jmp 1b");
    llvm_asm!("mov esp, 0xffbfeffc" : : : : "intel", "volatile");
    INPUT_THREAD_ID = process::get_current_pid();
  }
  crate::kprintln!("INPUT THREAD REPORTING!");
  loop {
    process::send_signal(unsafe { INPUT_THREAD_ID }, syscall::signals::STOP);
    process::yield_coop();
    crate::kprintln!("INPUT CHECK");
  }
}