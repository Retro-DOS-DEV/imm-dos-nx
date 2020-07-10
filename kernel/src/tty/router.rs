use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::drivers::keyboard::KeyAction;
use spin::RwLock;

use super::keyboard::KeyState;
use super::tty::TTY;

/**
 * The TTY Router keeps a record of which TTY is currently "active," and routes
 * all input events there. The active TTY will output keyboard actions to any
 * processes listening to its TTY device file (ie, "DEV:\TTY1")
 */
pub struct TTYRouter {
  tty_set: RwLock<Vec<Arc<RwLock<TTY>>>>,
  active_tty: usize,
  key_state: KeyState,
}

impl TTYRouter {
  pub fn new() -> TTYRouter {
    let mut set = Vec::with_capacity(1);
    let mut tty = TTY::new();
    tty.set_active(true);
    set.push(Arc::new(RwLock::new(tty)));
    TTYRouter {
      tty_set: RwLock::new(set),
      active_tty: 0,
      key_state: KeyState::new(),
    }
  }

  pub fn create_tty(&self) -> usize {
    let tty = TTY::new();
    
    let mut set = self.tty_set.write();
    let index = set.len();
    set.push(Arc::new(RwLock::new(tty)));
    index
  }

  pub fn get_active_tty(&self) -> Option<Arc<RwLock<TTY>>> {
    let set = self.tty_set.read();
    let active = set.get(self.active_tty);
    match active {
      Some(tty) => Some(Arc::clone(tty)),
      None => None
    }
  }

  pub fn set_active_tty(&mut self, index: usize) {
    if self.tty_set.read().len() <= index {
      return;
    }
    if let Some(tty) = self.get_active_tty() {
      tty.write().set_active(false);
    }
    self.active_tty = index;
    if let Some(tty) = self.get_active_tty() {
      tty.write().set_active(true);
    }
  }

  pub fn send_key_action(&mut self, action: KeyAction) {
    let output = self.key_state.process_key_action(action);
    if let Some(ascii) = output {
      if let Some(tty) = self.get_active_tty() {
        tty.write().send_data(ascii);
      }
    }
  }
}
