pub mod keys;
pub mod memory;
pub mod router;
pub mod vterm;

use crate::input::keyboard::KeyAction;
use router::VTermRouter;
use spin::RwLock;

static mut ROUTER: Option<RwLock<VTermRouter>> = None;

pub fn init_vterm() {
  let global_router = router::VTermRouter::new();

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
