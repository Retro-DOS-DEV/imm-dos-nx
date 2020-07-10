use crate::drivers::keyboard::{KeyAction, codes::{KeyCode, US_LAYOUT}};

pub struct KeyState {
  pub alt: bool,
  pub ctrl: bool,
  pub shift: bool,
}

impl KeyState {
  pub fn new() -> KeyState {
    KeyState {
      alt: false,
      ctrl: false,
      shift: false,
    }
  }

  pub fn process_key_action(&mut self, action: KeyAction) -> Option<u8> {
    match action {
      KeyAction::Press(code) => {
        match code {
          KeyCode::Alt => {
            self.alt = true;
            None
          },
          KeyCode::Control => {
            self.ctrl = true;
            None
          },
          KeyCode::Shift => {
            self.shift = true;
            None
          },
          _ => Some(self.key_code_to_ascii(code)),
        }
      },
      KeyAction::Release(code) => {
        match code {
          KeyCode::Alt => self.alt = false,
          KeyCode::Control => self.ctrl = false,
          KeyCode::Shift => self.shift = false,
          _ => (),
        }
        None
      },
    }
  }

  pub fn key_code_to_ascii(&self, input: KeyCode) -> u8 {
    let index = input as usize;
    let (normal, shifted) = if index < 0x60 {
      US_LAYOUT[index]
    } else {
      (0, 0)
    };
    if self.shift {
      shifted
    } else {
      normal
    }
  }
}
