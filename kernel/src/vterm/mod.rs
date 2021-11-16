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
