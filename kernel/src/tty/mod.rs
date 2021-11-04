pub mod buffers;
pub mod device;
pub mod keyboard;
pub mod router;
pub mod tty;

use core::fmt::Write;
use crate::input::keyboard::KeyAction;
use crate::task::yield_coop;
use spin::RwLock;

pub static mut ROUTER: Option<RwLock<router::TTYRouter>> = None;

pub fn init_ttys() {
  let global_router = router::TTYRouter::new();
  for tty in 0..global_router.tty_count() {
    crate::devices::create_tty(tty);
  }
  unsafe {
    ROUTER = Some(RwLock::new(global_router));
  }
  console_write(format_args!("TTY system \x1b[92mready\x1b[m\n"));
}

pub fn get_router() -> &'static RwLock<router::TTYRouter> {
  match unsafe {&ROUTER} {
    Some(r) => &r,
    None => panic!("TTYs have not been initialized"),
  }
}

pub fn process_key_action(action: KeyAction) {
  match unsafe {&ROUTER} {
    Some(r) => r.write().send_key_action(action),
    None => (), // do nothing
  }
}

pub fn begin_session(tty: usize, program: &str) -> Result<(), ()> {
  let current_id = crate::task::switching::get_current_id();
  let tty_device = alloc::format!("DEV:\\TTY{}", tty);
  let stdin = crate::task::io::open_path(&tty_device).unwrap();
  let stdout = crate::task::io::dup(stdin, None).unwrap();
  let stderr = crate::task::io::dup(stdin, None).unwrap();

  get_router().read().set_foreground_process(tty, current_id);

  crate::task::exec::exec(program, crate::loaders::InterpretationMode::Native).map_err(|_| ())
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
    use crate::devices::driver::DeviceDriver;

    let device = device::TTYDevice::for_tty(0);
    device.write(
      crate::devices::driver::IOHandle::new(0),
      s.as_bytes(),
    ).map(|_| ()).map_err(|_| core::fmt::Error)
  }
}

/// Write content to TTY0, aka the Console
pub fn console_write(args: core::fmt::Arguments) {
  let mut con = Console();
  con.write_fmt(args).unwrap();
}
