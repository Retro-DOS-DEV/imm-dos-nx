use core::panic::PanicInfo;
use crate::hardware::qemu;
use crate::kprintln;

#[cfg(all(not(feature = "testing"), not(test)))]
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