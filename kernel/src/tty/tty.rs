use crate::hardware::vga::text_mode::{ColorCode, TextMode};
use crate::memory::address::VirtualAddress;

pub struct TTY {
  is_active: bool,
  text_buffer: TextMode,
}

impl TTY {
  pub fn new() -> TTY {
    TTY {
      is_active: false,
      text_buffer: TextMode::new(VirtualAddress::new(0xc00b8000)),
    }
  }

  pub fn set_active(&mut self, active: bool) {
    self.is_active = active;
  }

  pub fn send_data(&mut self, byte: u8) {
    if self.is_active {
      self.text_buffer.write_byte(byte);
      self.text_buffer.invert_cursor();
    }
  }
}