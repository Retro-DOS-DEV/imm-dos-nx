use core::fmt::{self, Write};
use crate::{devices, interrupts};

#[cfg(not(feature = "testing"))]
pub fn _kprint(args: fmt::Arguments) {
  /*
  let int_reenable = interrupts::control::is_interrupt_enabled();
  interrupts::control::cli();
  unsafe {
    devices::VGA_TEXT.write_fmt(args).unwrap();
  }
  if int_reenable {
    interrupts::control::sti();
  }
  */
  unsafe {
    let mut serial = crate::input::com::serial::SerialPort::new(0x3f8);
    serial.write_fmt(args).unwrap();
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
  ($($arg:tt)*) => ($crate::vterm::console_write(format_args!($($arg)*)));
}

pub fn log_dos_syscall(method: u8) {
  kprintln!("DOS API: {:X}", method);
}

pub fn log_dos_interrupt(interrupt: u8) {
  kprintln!("DOS INT: {:X}", interrupt);
}
