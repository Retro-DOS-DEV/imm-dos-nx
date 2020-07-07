use alloc::sync::Arc;
use crate::files::handle::LocalHandle;
use crate::x86::io::Port;
use spin::Mutex;
use super::driver::DeviceDriver;

pub mod codes;
pub mod readers;

use codes::KeyCode;

pub struct Keyboard {
  meta_keys: MetaKeyState,
  receiving_extended_code: bool,
  data: Port,

  open_readers: Mutex<readers::OpenReaders>,
}

pub struct MetaKeyState {
  shift_pressed: bool,
  ctrl_pressed: bool,
  alt_pressed: bool,
}

pub enum KeyAction {
  Press(KeyCode),
  Release(KeyCode),
}

impl Keyboard {
  pub fn new() -> Keyboard {
    Keyboard {
      meta_keys: MetaKeyState {
        shift_pressed: false,
        ctrl_pressed: false,
        alt_pressed: false,
      },
      receiving_extended_code: false,
      data: Port::new(0x60),
      open_readers: Mutex::new(readers::OpenReaders::new()),
    }
  }

  pub fn handle_interrupt(&mut self) {
    let code = unsafe {
      self.data.read_u8()
    };
    match self.generate_action_from_scan_code(code) {
      Some(action) => self.process_action(action),
      None => (),
    }
  }

  pub fn generate_action_from_scan_code(&mut self, scan_code: u8) -> Option<KeyAction> {
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
      }
    }
  }

  pub fn process_action(&mut self, action: KeyAction) {
    match action {
      KeyAction::Press(code) => match code {
        KeyCode::Alt => self.meta_keys.alt_pressed = true,
        KeyCode::Control => self.meta_keys.ctrl_pressed = true,
        KeyCode::Shift => self.meta_keys.shift_pressed = true,
        _ => {
          let mut open_readers = self.open_readers.lock();
          for (_, codes) in open_readers.get_map().iter_mut() {
            codes.push(code);
          }
        },
      },
      KeyAction::Release(code) => match code {
        KeyCode::Alt => self.meta_keys.alt_pressed = false,
        KeyCode::Control => self.meta_keys.ctrl_pressed = false,
        KeyCode::Shift => self.meta_keys.shift_pressed = false,
        _ => (),
      },
    }
  }
}

pub struct KeyboardDevice {
  keyboard: Arc<Mutex<Keyboard>>,
}

impl KeyboardDevice {
  pub fn new(keyboard: Arc<Mutex<Keyboard>>) -> KeyboardDevice {
    KeyboardDevice {
      keyboard,
    }
  }
}

impl DeviceDriver for KeyboardDevice {
  fn open(&self, handle: LocalHandle) -> Result<(), ()> {
    let keyboard = self.keyboard.lock();
    let mut open_readers = keyboard.open_readers.lock();
    open_readers.open(handle);
    Ok(())
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    let keyboard = self.keyboard.lock();
    let mut open_readers = keyboard.open_readers.lock();
    open_readers.close(handle);
    Ok(())
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let keyboard = self.keyboard.lock();
    let mut open_readers = keyboard.open_readers.lock();
    let read_len = open_readers.read(handle, buffer);
    Ok(read_len)
  }

  fn write(&self, _handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(0)
  }
}
