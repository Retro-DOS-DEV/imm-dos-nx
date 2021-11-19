use crate::input::keyboard::{KeyAction, KeyCode, codes::US_LAYOUT};

/// In order to apply meta keys like shift, control, and alt, the router needs
/// to track when they are pressed and released. KeyState helps track this, and
/// use the meta key state to determine the characters that should be sent from
/// each key press.
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

  /// Process a raw KeyAction from the keyboard, converting it to either a meta-
  /// key effect or a stream of bytes to be handled by the TTY parser.
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
          _ => {
            let len = self.key_code_to_ascii(code, buffer);
            Some(len)
          },
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

  /// Convert a KeyCode into a series of ASCII characters, placing them in the
  /// buffer and returning the number of characters.
  pub fn key_code_to_ascii(&self, input: KeyCode, buffer: &mut [u8]) -> usize {
    if self.ctrl {
      match input {
        KeyCode::C => {
          buffer[0] = 0x03; // EOT / Break
          return 1;
        },
        _ => (),
      }
    }
    match input {
      KeyCode::ArrowUp => {
        buffer[0] = 0x1b;
        buffer[1] = b'[';
        buffer[2] = b'A';
        3
      },
      KeyCode::ArrowDown => {
        buffer[0] = 0x1b;
        buffer[1] = b'[';
        buffer[2] = b'B';
        3
      },
      KeyCode::ArrowRight => {
        buffer[0] = 0x1b;
        buffer[1] = b'[';
        buffer[2] = b'C';
        3
      },
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
