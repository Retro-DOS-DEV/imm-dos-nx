use core::fmt::{self, Write};
use crate::{devices, interrupts};

#[cfg(not(feature = "testing"))]
pub fn _kprint(args: fmt::Arguments) {
  let int_reenable = interrupts::control::is_interrupt_enabled();
  interrupts::control::cli();
  unsafe {
    devices::VGA_TEXT.write_fmt(args).unwrap();
  }
  if int_reenable {
    interrupts::control::sti();
  }
}

#[cfg(feature = "testing")]
pub fn _kprint(args: fmt::Arguments) {
  unsafe {
    let serial = devices::get_raw_serial();
    serial.write_fmt(args).unwrap();
  }
}

#[macro_export]
macro_rules! kprint {
  ($($arg:tt)*) => ($crate::debug::_kprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
  () => ($crate::kprint!("\n"));
  ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! klog {
  ($($arg:tt)*) => ($crate::tty::console_write(format_args!($($arg)*)));
}
