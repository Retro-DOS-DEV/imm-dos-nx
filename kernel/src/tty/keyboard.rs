use crate::input::keyboard::{KeyAction, codes::{KeyCode, US_LAYOUT}};

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

  pub fn process_key_action(&mut self, action: KeyAction, buffer: &mut [u8]) -> Option<usize> {
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
          _ => Some(self.key_code_to_ascii(code, buffer)),
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

  pub fn key_code_to_ascii(&self, input: KeyCode, buffer: &mut [u8]) -> usize {
    match input {
      KeyCode::ArrowLeft => {
        buffer[0] = 0x1b;
        buffer[1] = b'[';
        buffer[2] = b'D';
        3
      },

      _ => {
        let index = input as usize;
        let (normal, shifted) = if index < 0x60 {
          US_LAYOUT[index]
        } else {
          (0, 0)
        };
        buffer[0] = if self.shift {
          shifted
        } else {
          normal
        };
        1
      }
    }
  }
}
