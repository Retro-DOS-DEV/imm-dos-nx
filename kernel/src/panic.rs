use core::panic::PanicInfo;
use crate::kprintln;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  kprintln!("PANIC: {}", info);
  loop {}
}