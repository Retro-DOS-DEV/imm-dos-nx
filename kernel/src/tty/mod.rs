pub mod keyboard;
pub mod router;
pub mod tty;

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
