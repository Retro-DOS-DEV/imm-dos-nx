pub mod buffers;
pub mod device;
pub mod keyboard;
pub mod router;
pub mod tty;

use crate::process::yield_coop;
use spin::RwLock;

pub static mut ROUTER: Option<RwLock<router::TTYRouter>> = None;

pub fn init_ttys() {
  let global_router = router::TTYRouter::new();
  unsafe {
    ROUTER = Some(RwLock::new(router::TTYRouter::new()));
  }
}

pub fn get_router() -> &'static RwLock<router::TTYRouter> {
  match unsafe {&ROUTER} {
    Some(r) => &r,
    None => panic!("TTYs have not been initialized"),
  }
}

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
