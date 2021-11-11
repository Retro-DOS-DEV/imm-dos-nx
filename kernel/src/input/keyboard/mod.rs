use crate::x86::io::Port;

pub mod codes;
#[cfg(not(test))]
pub mod device;

pub use codes::KeyCode;

/// A way of encoding a keyboard event into a combination of a button action
/// and the unique key that changed
#[derive(Copy, Clone)]
pub enum KeyAction {
  Press(KeyCode),
  Release(KeyCode),
}

impl KeyAction {
  /// Convert the key event to a simple multi-byte format that can be read by
  /// other programs
  pub fn to_raw(&self) -> [u8; 2] {
    match self {
      KeyAction::Press(code) => [1, *code as u8],
      KeyAction::Release(code) => [2, *code as u8],
    }
  }
}

/// State machine for translating raw keyboard scancodes into usable
/// information. Most key events are a single scancode long, but some are
/// extended, so we need a simple state machine to keep track of the multi-
/// code events.
pub struct Keyboard {
  receiving_extended_code: bool,
  data: Port,
  status: Port,
}

impl Keyboard {
  pub const fn new() -> Keyboard {
    Keyboard {
      receiving_extended_code: false,
      data: Port::new(0x60), // PS/2 read+write data port
      status: Port::new(0x64), // Status and command register
    }
  }

  /// Handle a raw stream of bytes from a PS/2 keyboard, one at a time.
  /// Each byte can trigger at most one key action (such as a key press or
  /// release), so the method returns an optional KeyAction if one has been
  /// generated.
  pub fn handle_raw_data(&mut self, scan_code: u8) -> Option<KeyAction> {
    if scan_code == 0xe0 {
      self.receiving_extended_code = true;
      return None;
    }
    let scan_code_key = scan_code & 0x7f;
    let pressed = scan_code & 0x80 == 0;

    let key_code = if self.receiving_extended_code {
      codes::get_extended_keycode(scan_code_key)
    } else {
      codes::get_keycode(scan_code_key)
    };
    self.receiving_extended_code = false;

    match key_code {
      KeyCode::None => None,
      _ => if pressed {
        Some(KeyAction::Press(key_code))
      } else {
        Some(KeyAction::Release(key_code))
      },
    }
  }
}