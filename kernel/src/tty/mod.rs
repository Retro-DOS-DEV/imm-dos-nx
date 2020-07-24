pub mod buffers;
pub mod device;
pub mod keyboard;
pub mod router;
pub mod tty;

use core::fmt::Write;
use crate::process::yield_coop;
use spin::RwLock;

pub static mut ROUTER: Option<RwLock<router::TTYRouter>> = None;

pub fn init_ttys() {
  let global_router = router::TTYRouter::new();
  unsafe {
    ROUTER = Some(RwLock::new(global_router));
  }
}

pub fn get_router() -> &'static RwLock<router::TTYRouter> {
  match unsafe {&ROUTER} {
    Some(r) => &r,
    None => panic!("TTYs have not been initialized"),
  }
}

/// Process runs within kernel mode and processes all data that has come into
/// DEV:/TTY files, sending it back to each TTY struct
#[inline(never)]
pub extern "C" fn ttys_process() {

  loop {
    // Check each TTY buffer for new data that we need to process
    let router = get_router();
    match router.try_read() {
      Some(r) => r.process_buffers(),
      None => (),
    }
    yield_coop();
  }
}

pub struct Console();

impl core::fmt::Write for Console {
  fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
    let router = get_router().read();
    match router.get_tty_buffers(0) {
      Some(b) => {
        b.input_buffer.write(s.as_bytes());
        Ok(())
      },
      None => Err(core::fmt::Error),
    }
  }
}

/// Write content to TTY0, aka the Console
pub fn console_write(args: core::fmt::Arguments) {
  let mut con = Console();
  con.write_fmt(args).unwrap();
}
