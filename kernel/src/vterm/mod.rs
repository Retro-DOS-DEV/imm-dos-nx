pub mod keys;
pub mod memory;
pub mod router;
pub mod vterm;

use crate::input::keyboard::KeyAction;
use router::VTermRouter;
use spin::RwLock;

static mut ROUTER: Option<RwLock<VTermRouter>> = None;

pub fn init_vterm() {
  let global_router = router::VTermRouter::new(5);

  unsafe {
    ROUTER = Some(RwLock::new(global_router));
  }
}

pub fn process_key_action(action: KeyAction) {
  match unsafe {&ROUTER} {
    Some(r) => r.write().send_key_action(action),
    None => (), // do nothing
  }
}

pub fn get_router() -> &'static RwLock<router::VTermRouter> {
  match unsafe {&ROUTER} {
    Some(r) => &r,
    None => panic!("VTerms have not been initialized"),
  }
}

#[cfg(not(test))]
pub fn begin_session(tty: usize, program: &str) -> Result<(), ()> {
  let current_id = crate::task::get_current_id();
  let tty_device = alloc::format!("DEV:\\TTY{}", tty);
  let stdin = crate::task::io::open_path(&tty_device).unwrap();
  let stdout = crate::task::io::dup(stdin, None).unwrap();
  let stderr = crate::task::io::dup(stdin, None).unwrap();

  // set foreground process for vterm here

  crate::task::exec::exec(program, crate::loaders::InterpretationMode::Native).map_err(|_| ())
}

#[inline(never)]
pub extern "C" fn vterm_process() {
  loop {
    // Check each TTY buffer for new data that we need to process
    let router = get_router();
    match router.try_write() {
      Some(mut r) => r.process_buffers(),
      None => (),
    }
    crate::task::yield_coop();
  }
}

/// Empty singleton-style struct to implement easy formatted writing
pub struct Console();

impl core::fmt::Write for Console {
  fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
    let router = get_router();
    // Alternatively, this could be a try_read and push the data to the buffer.
    // That might be better...
    match router.try_write() {
      Some(mut r) => r.write_to_console(s),
      None => (),
    }
    Ok(())
  }
}

/// Write content to TTY0, aka the Console
pub fn console_write(args: core::fmt::Arguments) {
  use core::fmt::Write;

  let mut con = Console();
  con.write_fmt(args).unwrap();
}
