
/**
 * Entry point for the kernel. Establishes devices, address space, and creates
 * process number 1. It then executes the actual init process, transitioning to
 * ring 3 and launching the OS.
 */
pub fn init() -> ! {


  loop {
    unsafe {
      llvm_asm!("hlt" : : : : "volatile");
    }
  }
}