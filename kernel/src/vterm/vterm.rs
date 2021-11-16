use crate::hardware::vga::text_mode::TextMode;
use crate::memory::address::PhysicalAddress;
use crate::tty::parser::{Parser, TTYAction};
use super::memory::MemoryBackup;

/// A vterm virtualizes access to the keyboard input and video output.
/// This is how the operating system achieves multitasking from the user's
/// perspective. DOS is inherently a single-tasking environment, where each
/// program takes over the entire screen. By capturing keyboard hooks to switch
/// between environments, it allows the user to run multiple DOS applications in
/// parallel.
/// 
/// Switching requires that each vterm stores all state necessary to reconstruct
/// the video state at any time, and can track any changes that happen while
/// inactive.
pub struct VTerm {
  pub video_mode: u8,
  memory_backups: [Option<MemoryBackup>; 32],
  text_mode_state: TextMode,
  ansi_parser: Parser,

  // ==== mode flags

  /// determines whether to echo characters received as input
  echo_input_flag: bool,
}

impl VTerm {
  pub fn with_video_mode(mode: u8) -> Self {
    let mut memory_backups = [None; 32];
    // all vterms have a memory backup for the "text mode" page at 0xb8000
    let backup = MemoryBackup::allocate(PhysicalAddress::new(0xb8000));
    let backup_location = backup.mapped_to;
    memory_backups[(0xb8000 - 0xa0000) / 0x1000] = Some(backup);
    Self {
      video_mode: mode,
      memory_backups,
      text_mode_state: TextMode::new(backup_location),
      ansi_parser: Parser::new(),
      echo_input_flag: true,
    }
  }

  pub fn get_memory_backup(&self, address: PhysicalAddress) -> Option<&MemoryBackup> {
    let addr = address.as_usize();
    if addr < 0xa0000 {
      return None;
    }
    if addr >= 0xc0000 {
      return None;
    }
    let index = (addr - 0xa0000) / 0x1000;
    self.memory_backups[index].as_ref()
  }

  /// When a VTerm becomes active, all stashed video state needs to be restored.
  /// Each active video memory area is copied back to physical memory. Depending
  /// on video state, some other IO ports may be set as well.
  pub fn make_active(&mut self) {
    unsafe {
      for backup in &self.memory_backups {
        if let Some(b) = backup {
          b.copy_from_buffer();
        }
      }
    }
    // When the terminal is active, write text mode content directly to video
    self.text_mode_state.set_buffer_pointer(0xc00b8000);
  }

  pub fn make_initial(&mut self) {
    self.text_mode_state.set_buffer_pointer(0xc00b8000);
  }

  /// When a VTerm becomes inactive, it needs to store its current state. This
  /// involves copying all active video memory areas to their back buffers.
  pub fn make_inactive(&mut self) {
    unsafe {
      for backup in &self.memory_backups {
        if let Some(b) = backup {
          b.copy_to_buffer();
        }
      }
    }
    let text_backup_addr = self.get_memory_backup(PhysicalAddress::new(0xb8000))
      .and_then(|backup| Some(backup.mapped_to.as_usize()));
    if let Some(addr) = text_backup_addr {
      self.text_mode_state.set_buffer_pointer(addr);
    }
  }

  /// Directly write a character to the text mode buffer
  pub fn write_character(&mut self, ch: u8) {
    if ch < 0x20 {
      self.text_mode_state.write_byte(b'^');
      self.text_mode_state.write_byte(ch + 0x40);
    } else {
      self.text_mode_state.write_byte(ch);
    }
  }

  /// Receive a buffer of characters directly from the keyboard, process them,
  /// and add them to the "read" side of the associated TTY device if there are
  /// any active readers.
  pub fn handle_input(&mut self, chars: &[u8]) {
    if self.echo_input_flag {
      for ch in chars {
        self.write_character(*ch);
      }
    }
    // find the matching TTY device and add these chars to the reader buffer
  }

  pub fn send_characters(&mut self, chars: &[u8]) {
    for ch in chars {
      let action = self.ansi_parser.process_character(*ch);
      match action {
        TTYAction::Print(print) => self.write_character(print),
        _ => {
          // if echoing control characters is enabled, print it
          self.write_character(*ch);
        },
      }
      match action {
        TTYAction::MoveCursor(dx, dy) => {
          self.text_mode_state.move_cursor_relative(dx, dy);
        },
        TTYAction::SetColumn(col) => {

        },
        TTYAction::SetPosition(col, row) => {
          self.text_mode_state.move_cursor(col as u8, row as u8);
        },
        TTYAction::ClearScreen => {
          self.text_mode_state.clear_screen();
        },
        TTYAction::ClearToBeginning => {
          self.text_mode_state.clear_screen_to_beginning();
        },
        TTYAction::ClearToEnd => {
          self.text_mode_state.clear_screen_to_end();
        },
        TTYAction::ClearRow => {
          self.text_mode_state.clear_row();
        },
        TTYAction::ClearRowToBeginning => {
          self.text_mode_state.clear_row_to_beginning();
        },
        TTYAction::ClearRowToEnd => {
          self.text_mode_state.clear_row_to_end();
        },
        TTYAction::NextLineStart(dist) => {

        },
        TTYAction::PrevLineStart(dist) => {

        },
        TTYAction::ScrollUp(lines) => {
          self.text_mode_state.scroll(lines as u8);
        },
        TTYAction::ScrollDown(lines) => {

        },
        TTYAction::ResetColors => {
          self.text_mode_state.reset_colors();
        },
        TTYAction::SetFgColor(fg) => {
          self.text_mode_state.set_fg_color(fg);
        },
        TTYAction::SetBgColor(bg) => {
          self.text_mode_state.set_bg_color(bg);
        },
        _ => (),
      }
    }
  }

  /// Scroll the text mode up by a specified number of rows
  pub fn scroll(&mut self, delta: usize) {
    self.text_mode_state.scroll(delta as u8);
  }
}