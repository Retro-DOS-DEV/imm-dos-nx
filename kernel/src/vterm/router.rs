use alloc::vec::Vec;
use crate::input::keyboard::{KeyAction, KeyCode};
use super::keys::KeyState;
use super::vterm::VTerm;

/// The vterm router collects all 
pub struct VTermRouter {
  vterm_list: Vec<VTerm>,
  active_vterm: usize,
  key_state: KeyState,
}

impl VTermRouter {
  pub fn new() -> Self {
    Self {
      vterm_list: Vec::new(),
      active_vterm: 0,
      key_state: KeyState::new(),
    }
  }

  pub fn set_active_vterm(&mut self, active: usize) {
    self.active_vterm = active;
    unsafe {
      let buffer = 0xc00b8000 as *mut u8;
      core::ptr::write_volatile(buffer, (active + 48) as u8);
    }
  }

  pub fn send_key_action(&mut self, action: KeyAction) {
    if self.key_state.alt {
      match action {
        KeyAction::Press(KeyCode::Num0) => {
          self.set_active_vterm(0);
          return;
        },
        KeyAction::Press(KeyCode::Num1) => {
          self.set_active_vterm(1);
          return;
        },
        KeyAction::Press(KeyCode::Num2) => {
          self.set_active_vterm(2);
          return;
        },
        KeyAction::Press(KeyCode::Num3) => {
          self.set_active_vterm(3);
          return;
        },
        KeyAction::Press(KeyCode::Num4) => {
          self.set_active_vterm(4);
          return;
        },
        KeyAction::Press(KeyCode::Num5) => {
          self.set_active_vterm(5);
          return;
        },
        KeyAction::Press(KeyCode::Num6) => {
          self.set_active_vterm(6);
          return;
        },
        KeyAction::Press(KeyCode::Num7) => {
          self.set_active_vterm(7);
          return;
        },
        KeyAction::Press(KeyCode::Num8) => {
          self.set_active_vterm(8);
          return;
        },
        KeyAction::Press(KeyCode::Num9) => {
          self.set_active_vterm(9);
          return;
        },
        _ => (),
      }
    }
    let mut input_buffer: [u8; 4] = [0; 4];
    let output = self.key_state.process_key_action(action, &mut input_buffer);
    if let Some(len) = output {
      
    }
  }
}