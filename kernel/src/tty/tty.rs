use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::hardware::vga::text_mode::{TextMode};
use crate::memory::address::VirtualAddress;

const BACK_BUFFER_SIZE: usize = 80 * 25 * 2;

#[derive(Copy, Clone)]
pub enum ParseState {
  Ready,
  EscapeStart, // Recognized an ESC sequence
  CSI, // Recognized a CSI sequence
}

/// Interface for a TTY. It parses ANSI-style terminal bytes and 
pub struct TTY {
  /// Whether this TTY is currently active, determines whether it outputs new
  /// characters to video RAM
  is_active: bool,
  /// Whether echoing is enabled, controled by ioctl commands
  echo: bool,
  /// Whether the cursor is currently visible
  show_cursor: bool,
  /// Track the current parsing state
  parse_state: ParseState,
  arg_digits_written: usize,
  csi_args: Vec<Option<u32>>,
  /// Access to VGA video memory, also stores the current cursor info
  text_buffer: TextMode,

  back_buffer: Vec<u8>,
}

impl TTY {
  pub fn new() -> TTY {
    let mut back_buffer = Vec::with_capacity(BACK_BUFFER_SIZE);
    for _ in 0..BACK_BUFFER_SIZE {
      back_buffer.push(0);
    }
    TTY {
      is_active: false,
      echo: true,
      show_cursor: true,
      parse_state: ParseState::Ready,
      arg_digits_written: 0,
      csi_args: Vec::with_capacity(8),
      text_buffer: TextMode::new(VirtualAddress::new(0xc00b8000)),
      back_buffer,
    }
  }

  pub fn set_active(&mut self, active: bool) {
    self.is_active = active;
  }

  pub fn send_data(&mut self, byte: u8) {
    //if self.is_active {
      let output = unsafe { self.process_character(byte) };

      if let Some(ch) = output {
        if self.echo {
          self.text_buffer.write_byte(ch);
          if self.show_cursor {
            self.text_buffer.invert_cursor();
          }
        }
      }
    //}
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
            self.arg_digits_written = 0;
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
          b';' => {
            self.csi_args.push(None);
            self.arg_digits_written = 0;
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
    let dest = 0xb8000;
    let dest_ptr = dest as *mut u32;
    self.text_buffer.set_buffer_pointer(dest);
    let src_ptr = self.back_buffer.as_ptr() as *mut u32;
    for off in 0..count {
      *dest_ptr.offset(off) = *src_ptr.offset(off);
    }
  }
}