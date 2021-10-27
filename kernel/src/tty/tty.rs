use alloc::vec::Vec;
use crate::hardware::vga::text_mode::{Color, TextMode};
use crate::memory::address::VirtualAddress;

const BACK_BUFFER_SIZE: usize = 80 * 25 * 2;
const VGA_TEXT_LOCATION: usize = 0xc00b8000;

#[derive(Copy, Clone)]
pub enum ParseState {
  Ready,
  EscapeStart, // Recognized an ESC sequence
  CSI, // Recognized a CSI sequence
}

/// How the terminal processes input
pub enum ReadMode {
  /// Send individual bytes directly to the TTY device, no output
  Raw,
  /// Process one line at a time
  Canonical,
}

/// Interface for a TTY. It parses ANSI-style terminal bytes and 
pub struct TTY {
  /// Read mode determines how data is collected before passed to readers
  read_mode: ReadMode,
  /// Whether echoing is enabled, controled by ioctl commands
  echo: bool,
  /// Whether the cursor is currently visible
  show_cursor: bool,
  /// Track the current parsing state
  parse_state: ParseState,
  csi_args: Vec<Option<u32>>,
  /// Access to VGA video memory, also stores the current cursor info
  text_buffer: TextMode,
  /// Length of actual text (non special characters) in the read buffer, when in
  /// canonical mode
  canonical_length: usize,

  back_buffer: Vec<u8>,
}

impl TTY {
  pub fn new() -> TTY {
    let mut back_buffer = Vec::with_capacity(BACK_BUFFER_SIZE);
    for _ in 0..BACK_BUFFER_SIZE {
      back_buffer.push(0);
    }
    TTY {
      read_mode: ReadMode::Canonical,
      echo: true,
      show_cursor: true,
      parse_state: ParseState::Ready,
      csi_args: Vec::with_capacity(8),
      text_buffer: TextMode::new(VirtualAddress::new(VGA_TEXT_LOCATION)),
      canonical_length: 0,
      back_buffer,
    }
  }

  pub fn reset_canonical_length(&mut self) {
    self.canonical_length = 0;
  }

  pub fn is_canonical_mode(&self) -> bool {
    match self.read_mode {
      ReadMode::Canonical => true,
      _ => false,
    }
  }

  pub fn send_data(&mut self, byte: u8) {
    if byte == 8 {
      // handle backspace
      if self.show_cursor {
        self.text_buffer.invert_cursor();
      }
      self.text_buffer.move_cursor_relative(-1, 0);
      self.text_buffer.write_byte(b' ');
      self.text_buffer.move_cursor_relative(-1, 0);
      if self.show_cursor {
        self.text_buffer.invert_cursor();
      }
      return;
    }

    let output = unsafe { self.process_character(byte) };

    if let Some(ch) = output {
      self.text_buffer.write_byte(ch);
      if self.show_cursor {
        self.text_buffer.invert_cursor();
      }
    }
  }

  pub fn handle_input(&mut self, byte: u8) {
    if self.is_canonical_mode() {
      if byte == 8 {
        if self.canonical_length > 0 {
          self.canonical_length -= 1;
          self.send_data(byte);
        }
        return;
      }
      if byte == b'\n' {
        self.canonical_length = 0;
      } else {
        self.canonical_length += 1;
      }
    }

    if self.echo {
      self.send_data(byte);
    }
  }

  pub fn get_csi_arg(&self, index: usize, default: u32) -> u32 {
    match self.csi_args.get(index) {
      Some(opt) => match opt {
        Some(val) => *val,
        None => default,
      },
      None => default,
    }
  }

  pub unsafe fn process_character(&mut self, byte: u8) -> Option<u8> {
    match self.parse_state {
      ParseState::Ready => {
        match byte {
          0x1b => {
            self.parse_state = ParseState::EscapeStart;
            return None;
          },
          _ => return Some(byte),
        }
      },

      ParseState::EscapeStart => {
        match byte {
          0x5b => {
            self.parse_state = ParseState::CSI;
            while !self.csi_args.is_empty() {
              self.csi_args.pop();
            }
            self.csi_args.push(None);
            return None;
          },
          _ => {
            self.parse_state = ParseState::Ready;
            return None;
          }
        }
      },

      ParseState::CSI => {
        self.text_buffer.disable_cursor();
        let done = match byte {
          b'0'..=b'9' => {
            // arguments are pushed in ascii digits
            let digit = (byte - 48) as u32;
            let last_index = self.csi_args.len() - 1;
            match self.csi_args.get_mut(last_index) {
              Some(slot) => {
                let current = match slot {
                  Some(value) => *value * 10,
                  None => 0,
                } + digit;
                *slot = Some(current);
              },
              None => (),
            }
            false
          },
          b';' => {
            self.csi_args.push(None);
            false
          },
          b'A' => { // Cursor Up
            let delta = self.get_csi_arg(0, 1);
            self.text_buffer.move_cursor_relative(0, delta as isize * -1);
            true
          },
          b'B' => { // Cursor Down
            let delta = self.get_csi_arg(0, 1);
            self.text_buffer.move_cursor_relative(0, delta as isize);
            true
          },
          b'C' => { // Cursor Forward
            let delta = self.get_csi_arg(0, 1);
            self.text_buffer.move_cursor_relative(delta as isize, 0);
            true
          },
          b'D' => { // Cursor Back
            let delta = self.get_csi_arg(0, 1);
            self.text_buffer.move_cursor_relative(delta as isize * -1, 0);
            true
          },
          b'E' => { // Cursor to next line start
            let delta = self.get_csi_arg(0, 1);
            true
          },
          b'F' => { // Cursor to previous line start
            let delta = self.get_csi_arg(0, 1);
            true
          },
          b'G' => { // Cursor to col
            let col = self.get_csi_arg(0, 1);
            true
          },
          b'H' => { // Cursor to position
            let row = self.get_csi_arg(0, 1);
            let col = self.get_csi_arg(1, 1);
            true
          },
          b'J' => { // Clear screen
            let direction = self.get_csi_arg(0, 0);
            match direction {
              0 => self.text_buffer.clear_screen_to_end(),
              1 => self.text_buffer.clear_screen_to_beginning(),
              2 | 3 => self.text_buffer.clear_screen(),
              _ => (),
            }
            true
          },
          b'K' => { // Clear in line
            let direction = self.get_csi_arg(0, 0);
            match direction {
              0 => self.text_buffer.clear_row_to_end(),
              1 => self.text_buffer.clear_row_to_beginning(),
              2 | 3 => self.text_buffer.clear_row(),
              _ => (),
            }
            true
          },
          
          b'm' => { // Select Graphic Rendition
            let modifier = self.get_csi_arg(0, 0);
            match modifier {
              0 => { // reset
                self.text_buffer.reset_colors();
              },

              30 => self.text_buffer.set_fg_color(Color::Black),
              31 => self.text_buffer.set_fg_color(Color::Red),
              32 => self.text_buffer.set_fg_color(Color::Green),
              33 => self.text_buffer.set_fg_color(Color::Brown),
              34 => self.text_buffer.set_fg_color(Color::Blue),
              35 => self.text_buffer.set_fg_color(Color::Magenta),
              36 => self.text_buffer.set_fg_color(Color::Cyan),
              37 => self.text_buffer.set_fg_color(Color::LightGrey),

              39 => self.text_buffer.set_fg_color(Color::LightGrey),

              40 => self.text_buffer.set_bg_color(Color::Black),
              41 => self.text_buffer.set_bg_color(Color::Red),
              42 => self.text_buffer.set_bg_color(Color::Green),
              43 => self.text_buffer.set_bg_color(Color::Brown),
              44 => self.text_buffer.set_bg_color(Color::Blue),
              45 => self.text_buffer.set_bg_color(Color::Magenta),
              46 => self.text_buffer.set_bg_color(Color::Cyan),
              47 => self.text_buffer.set_bg_color(Color::LightGrey),

              49 => self.text_buffer.set_bg_color(Color::Black),

              90 => self.text_buffer.set_fg_color(Color::DarkGrey),
              91 => self.text_buffer.set_fg_color(Color::LightRed),
              92 => self.text_buffer.set_fg_color(Color::LightGreen),
              93 => self.text_buffer.set_fg_color(Color::LightBrown),
              94 => self.text_buffer.set_fg_color(Color::LightBlue),
              95 => self.text_buffer.set_fg_color(Color::LightMagenta),
              96 => self.text_buffer.set_fg_color(Color::LightCyan),
              97 => self.text_buffer.set_fg_color(Color::White),

              _ => (),
            }
            true
          },

          _ => true,
        };
        if done {
          self.parse_state = ParseState::Ready;
        }
        if self.show_cursor {
          self.text_buffer.invert_cursor();
        }
        return None;
      },
    }
  }

  /// Copy VRAM to the back buffer, and make the text buffer point to the
  /// back buffer.
  pub unsafe fn swap_out(&mut self) {
    let count = BACK_BUFFER_SIZE as isize / 4;
    let dest_ptr = self.back_buffer.as_mut_ptr() as *mut u32;
    let src = self.text_buffer.set_buffer_pointer(dest_ptr as usize);
    let src_ptr = src as *const u32;
    for off in 0..count {
      *dest_ptr.offset(off) = *src_ptr.offset(off);
    }
  }

  pub fn force_background(&mut self) {
    let back_ptr = self.back_buffer.as_ptr();
    self.text_buffer.set_buffer_pointer(back_ptr as usize);
  }

  /// Copy the back buffer to VRAM, and make the text buffer point to VRAM.
  pub unsafe fn swap_in(&mut self) {
    let count = BACK_BUFFER_SIZE as isize / 4;
    let dest = VGA_TEXT_LOCATION;
    let dest_ptr = dest as *mut u32;
    self.text_buffer.set_buffer_pointer(dest);
    let src_ptr = self.back_buffer.as_ptr() as *mut u32;
    for off in 0..count {
      *dest_ptr.offset(off) = *src_ptr.offset(off);
    }
  }
}