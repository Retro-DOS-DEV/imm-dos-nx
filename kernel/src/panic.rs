use core::panic::PanicInfo;
use crate::hardware::qemu;
use crate::kprintln;

#[cfg(not(feature = "testing"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  kprintln!("PANIC: {}", info);
  loop {}
}

#[cfg(feature = "testing")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  kprintln!("[FAILED] {}", info);
  qemu::debug_exit(3);
  loop {}
}