use core::fmt;
use core::ptr::{read_volatile, write_volatile};
use crate::memory::address::VirtualAddress;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Color {
  Black = 0,
  Blue = 1,
  Green = 2,
  Cyan = 3,
  Red = 4,
  Magenta = 5,
  Brown = 6,
  LightGrey = 7,
  DarkGrey = 8,
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  LightMagenta = 13,
  LightBrown = 14,
  White = 15,
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ColorCode(pub u8);

impl ColorCode {
  pub const fn new(fg: Color, bg: Color) -> ColorCode {
    ColorCode((bg as u8) << 4 | (fg as u8))
  }

  pub fn as_u8(&self) -> u8 {
    self.0
  }

  pub fn set_fg(&self, fg: Color) -> ColorCode {
    ColorCode((self.0 & 0xf0) | (fg as u8))
  }

  pub fn set_bg(&self, bg: Color) -> ColorCode {
    ColorCode((bg as u8) << 4 | (self.0 & 0x0f))
  }
}

pub struct TextMode {
  base_pointer: *mut u8,

  cursor_col: u8,
  cursor_row: u8,
  
  current_color: ColorCode,
}

impl TextMode {
  pub const fn new(base: VirtualAddress) -> TextMode {
    TextMode {
      base_pointer: base.as_usize() as *mut u8,
      cursor_col: 0,
      cursor_row: 24,
      current_color: ColorCode::new(Color::LightGrey, Color::Black),
    }
  }
  
  pub fn set_fg_color(&mut self, color: Color) {
    self.current_color = self.current_color.set_fg(color);
  }

  pub fn set_bg_color(&mut self, color: Color) {
    self.current_color = self.current_color.set_bg(color);
  }

  pub fn reset_colors(&mut self) {
    self.current_color = ColorCode::new(Color::LightGrey, Color::Black);
  }

  pub fn clear_screen(&mut self) {
    let mut offset = 0;
    unsafe {
      while offset < 2 * 80 * 25 {
        write_volatile(self.base_pointer.offset(offset), 0x20);
        offset += 2;
      }
    }
  }

  pub fn clear_screen_to_beginning(&mut self) {
    let mut offset = 0;
    let limit = (self.cursor_col as isize) + (self.cursor_row as isize * 80);
    unsafe {
      while offset <= 2 * limit {
        write_volatile(self.base_pointer.offset(offset), 0x20);
        offset += 2;
      }
    }
  }

  pub fn clear_screen_to_end(&mut self) {
    let mut offset = (self.cursor_col as isize) + (self.cursor_row as isize * 80) * 2;
    unsafe {
      while offset < 2 * 80 * 25 {
        write_volatile(self.base_pointer.offset(offset), 0x20);
        offset += 2;
      }
    }
  }

  pub fn clear_row(&mut self) {
    let mut offset = self.cursor_row as isize * 80 * 2;
    let limit = offset + 80 * 2;
    unsafe {
      while offset < limit {
        write_volatile(self.base_pointer.offset(offset), 0x20);
        offset += 2;
      }
    }
  }

  pub fn clear_row_to_beginning(&mut self) {
    let mut offset = self.cursor_row as isize * 80 * 2;
    let limit = offset + (self.cursor_col as isize) * 2;
    unsafe {
      while offset <= limit {
        write_volatile(self.base_pointer.offset(offset), 0x20);
        offset += 2;
      }
    }
  }

  pub fn clear_row_to_end(&mut self) {
    let mut offset = (self.cursor_row as isize * 80 * 2) + (self.cursor_col as isize * 2);
    let limit = (self.cursor_row as isize + 1) * 80 * 2;
    unsafe {
      while offset < limit {
        write_volatile(self.base_pointer.offset(offset), 0x20);
        offset += 2;
      }
    }
  }

  pub fn scroll(&mut self, rows: u8) {
    if rows == 0 {
      return;
    }
    if rows > 24 {
      self.clear_screen();
      return;
    }
    let mut dest = self.base_pointer;
    let scroll_rows = 25 - rows;
    let offset = (rows as isize) * 80 * 2;
    unsafe {
      for _i in 0..scroll_rows {
        for _j in 0..80 {
          let value = read_volatile(dest.offset(offset));
          let color = read_volatile(dest.offset(offset + 1));
          write_volatile(dest, value);
          write_volatile(dest.offset(1), color);
          dest = dest.offset(2);
        }
      }
      for _i in 0..rows {
        for _j in 0..80 {
          write_volatile(dest, 0x20);
          write_volatile(dest.offset(1), self.current_color.as_u8());
          dest = dest.offset(2);
        }
      }
    }
  }

  pub fn newline(&mut self) {
    self.cursor_col = 0;
    if self.cursor_row < 24 {
      self.cursor_row += 1;
      return;
    }
    self.scroll(1);
  }

  pub fn advance_cursor(&mut self) {
    if self.cursor_col < 79 {
      self.cursor_col += 1;
      return;
    }
    self.newline();
  }

  pub fn backspace(&mut self) {
    if self.cursor_col > 0 {
      self.cursor_col -= 1;
      self.set_current_character(b' ');
    } else if self.cursor_row > 0 {
      self.cursor_col = 79;
      self.cursor_row -= 1;
      self.set_current_character(b' ');
    }
  }

  pub fn set_current_character(&self, ch: u8) {
    let offset = (self.cursor_row as isize) * 160 + (self.cursor_col as isize) * 2;
    unsafe {
      write_volatile(self.base_pointer.offset(offset), ch);
    }
  }

  pub fn move_cursor(&mut self, col: u8, row: u8) {
    self.cursor_col = col;
    if self.cursor_col > 79 {
      self.cursor_col = 79;
    }
    self.cursor_row = row;
    if self.cursor_row > 24 {
      self.cursor_row = 24;
    }
  }

  pub fn move_cursor_relative(&mut self, dcol: isize, drow: isize) {
    let new_col = self.cursor_col as isize + dcol;
    self.cursor_col = if new_col < 0 {
      0
    } else if new_col > 79 {
      79
    } else {
      new_col as u8
    };
    let new_row = self.cursor_row as isize + drow;
    self.cursor_row = if new_row < 0 {
      0
    } else if new_row > 24 {
      24
    } else {
      new_row as u8
    };
  }

  pub fn invert_cursor(&self) {
    let offset = (self.cursor_row as isize) * 160 + (self.cursor_col as isize) * 2;
    unsafe {
      let cursor_color_ptr = self.base_pointer.offset(offset + 1);
      let current_color = read_volatile(cursor_color_ptr);
      let inverted_color = ((current_color & 0xf) << 4) | ((current_color & 0xf0) >> 4);
      write_volatile(cursor_color_ptr, inverted_color);
    }
  }

  pub fn disable_cursor(&self) {
    let offset = (self.cursor_row as isize) * 160 + (self.cursor_col as isize) * 2;
    unsafe {
      let cursor_color_ptr = self.base_pointer.offset(offset + 1);
      write_volatile(cursor_color_ptr, self.current_color.as_u8());
    }
  }

  pub fn write_byte(&mut self, byte: u8) {
    match byte {
      b'\n' => unsafe {
        self.disable_cursor();
        self.newline()
      },
      0x20..=0x7e => unsafe {
        let offset = (self.cursor_row as isize) * 160 + (self.cursor_col as isize) * 2;
        write_volatile(self.base_pointer.offset(offset), byte);
        write_volatile(self.base_pointer.offset(offset + 1), self.current_color.as_u8());
        self.advance_cursor();
      },
      _ => (),
    }
  }

  pub fn write_string(&mut self, s: &str) {
    for byte in s.bytes() {
      self.write_byte(byte);
    }
  }

  pub fn set_buffer_pointer(&mut self, ptr: usize) -> usize {
    let current_ptr = self.base_pointer as usize;
    self.base_pointer = ptr as *mut u8;
    current_ptr
  }
}

impl fmt::Write for TextMode {
  fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
    self.write_string(s);
    Ok(())
  }
}
