use core::fmt::{self, Write};

use crate::devices;

pub fn _kprint(args: fmt::Arguments) {
  unsafe {
    devices::VGA_TEXT.write_fmt(args).unwrap();
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
