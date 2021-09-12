use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::input::keyboard::{KeyAction, codes::KeyCode};
use spin::RwLock;

use super::buffers::TTYReadWriteBuffers;
use super::keyboard::KeyState;
use super::tty::TTY;

/// Associates a TTY driver, containing internal screen state and the ability to
/// write to the VGA buffer, with a device file that can be written and read by
/// other processes.
pub struct TTYData {
  tty: Arc<RwLock<TTY>>,
  buffers: Arc<TTYReadWriteBuffers>,
}

impl TTYData {
  pub fn new(tty: TTY) -> TTYData {
    TTYData {
      tty: Arc::new(RwLock::new(tty)),
      buffers: Arc::new(TTYReadWriteBuffers::new()),
    }
  }

  pub fn get_tty(&self) -> Arc<RwLock<TTY>> {
    Arc::clone(&self.tty)
  }

  pub fn get_buffers(&self) -> Arc<TTYReadWriteBuffers> {
    Arc::clone(&self.buffers)
  }
}

/// The TTY Router keeps a record of which TTY is currently "active," and routes
/// all input events there. The active TTY will output keyboard actions to any
/// processes listening to its TTY device file (ie, "DEV:\TTY1")
pub struct TTYRouter {
  tty_set: RwLock<Vec<TTYData>>,
  active_tty: usize,
  key_state: KeyState,
}

impl TTYRouter {
  pub fn new() -> TTYRouter {
    let mut set = Vec::with_capacity(1);
    let mut tty0 = TTY::new();
    tty0.set_active(true);

    set.push(TTYData::new(tty0));
    // Put all other TTYs into the background by default
    let mut tty1 = TTY::new();
    tty1.force_background();
    set.push(TTYData::new(tty1));
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
    set.push(TTYData::new(tty));
    index
  }

  pub fn get_tty_buffers(&self, index: usize) -> Option<Arc<TTYReadWriteBuffers>> {
    let set = self.tty_set.read();
    let data = set.get(index);
    match data {
      Some(tty) => Some(tty.get_buffers()),
      None => None
    }
  }

  pub fn get_active_tty(&self) -> Option<Arc<RwLock<TTY>>> {
    let set = self.tty_set.read();
    let active = set.get(self.active_tty);
    match active {
      Some(data) => Some(Arc::clone(&data.tty)),
      None => None
    }
  }

  pub fn set_active_tty(&mut self, index: usize) {
    if self.tty_set.read().len() <= index {
      return;
    }
    if self.active_tty == index {
      return;
    }
    if let Some(tty) = self.get_active_tty() {
      let mut prev = tty.write();
      prev.set_active(false);
      unsafe { prev.swap_out(); }
    }
    self.active_tty = index;
    if let Some(tty) = self.get_active_tty() {
      let mut next = tty.write();
      next.set_active(true);
      unsafe { next.swap_in(); }
    }
  }

  pub fn send_key_action(&mut self, action: KeyAction) {
    let mut buffer: [u8; 4] = [0; 4];

    let output = self.key_state.process_key_action(action, &mut buffer);
    if let Some(len) = output {
      match action {
        KeyAction::Press(KeyCode::Num0) => {
          if self.key_state.alt {
            self.set_active_tty(0);
            return;
          }
        },
        KeyAction::Press(KeyCode::Num1) => {
          if self.key_state.alt {
            self.set_active_tty(1);
            return;
          }
        },
        _ => (),
      }

      let tty_set = self.tty_set.read();
      if let Some(active) = tty_set.get(self.active_tty) {
        let mut tty = active.tty.write();
        let data: &[u8] = &buffer[0..len];
        for i in 0..len {
          tty.handle_input(data[i]);
        }
        active.buffers.output_buffer.write(&data);
      }
    }
  }

  /// Iterate through all ring buffers, and send all available data to the
  /// matching TTY device.
  pub fn process_buffers(&self) {
    let set = self.tty_set.read();
    for data in set.iter() {
      let buffers = data.get_buffers();
      match data.tty.try_write() {
        Some(mut tty) => {
          let mut data: [u8; 4] = [0; 4];
          let mut to_read = buffers.input_buffer.available_bytes();
          while to_read > 0 {
            let bytes_read = buffers.input_buffer.read(&mut data);
            to_read = if bytes_read == data.len() {
              to_read - bytes_read
            } else {
              0
            };
            for i in 0..bytes_read {
              tty.send_data(data[i]);
            }
          }
        },
        // If the tty is locked, we'll just get to it on the next call
        None => (),
      }
    }
  }
}
