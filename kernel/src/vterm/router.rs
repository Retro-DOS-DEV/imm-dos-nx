use alloc::vec::Vec;
use crate::hardware::vga::text_mode::{Color, ColorCode};
use crate::input::keyboard::{KeyAction, KeyCode};
use crate::memory::address::PhysicalAddress;
use super::keys::KeyState;
use super::vterm::VTerm;

/// The vterm router collects all 
pub struct VTermRouter {
  vterm_list: Vec<VTerm>,
  active_vterm: usize,
  key_state: KeyState,
}

impl VTermRouter {
  pub fn new(count: usize) -> Self {
    let mut vterm_list = Vec::new();
    for i in 0..count {
      let mode = if i == count - 1 {
        0x13
      } else {
        0x03
      };

      let mut term = VTerm::with_video_mode(mode);
      if i == 0 {
        term.make_initial();
        term.scroll(2);
      }
      // make the associated tty device
      let index = crate::tty::device::create_tty();
      term.set_tty_index(index);

      vterm_list.push(term);
    }
    Self {
      vterm_list,
      active_vterm: 0,
      key_state: KeyState::new(),
    }
  }

  pub fn set_active_vterm(&mut self, active: usize) {
    let current_term = match self.vterm_list.get_mut(self.active_vterm) {
      Some(v) => v,
      None => return,
    };
    current_term.make_inactive();

    let next_vterm = match self.vterm_list.get_mut(active) {
      Some(v) => v,
      None => return,
    };
    self.active_vterm = active;
    let video_mode = next_vterm.video_mode;
    // This will pause the calling process (likely the input process) until the
    // hardware request finishes.
    // If it fails to complete, it should time out after a second, unlocking the
    // input process.
    #[cfg(not(test))]
    {
      crate::hardware::vga::driver::request_mode_change_with_timeout(video_mode, 1000);
      let current_mode = crate::hardware::vga::driver::get_video_mode();
      if video_mode != current_mode {
        crate::kprintln!("Failed to set video mode");
        return;
      }
    }

    next_vterm.make_active();

    if video_mode == 0x03 {
      unsafe {
        let buffer = 0xc00b8000 as *mut u16;
        let low = ((active & 0xff) + 48) as u16;
        let high = ColorCode::new(Color::White, Color::Black).as_u8() as u16;
        core::ptr::write_volatile(buffer, low | (high << 8));
      }
    }
  }

  pub fn get_active_vterm(&self) -> usize {
    self.active_vterm
  }

  pub fn change_video_mode(&mut self, index: usize, mode: u8) {
    let mut vterm = match self.vterm_list.get_mut(index) {
      Some(v) => v,
      None => return,
    };
    if vterm.video_mode == mode {
      return;
    }
    vterm.video_mode = mode;
    #[cfg(not(test))]
    if self.active_vterm == index {
      crate::hardware::vga::driver::request_mode_change_with_timeout(mode, 1000);
      let current_mode = crate::hardware::vga::driver::get_video_mode();
      if mode != current_mode {
        crate::kprintln!("Failed to set video mode");
        return;
      }
    }
  }

  pub fn enter_dos_mode(&mut self, index: usize) {
    let vterm = match self.vterm_list.get_mut(index) {
      Some(v) => v,
      None => return,
    };
    vterm.enter_dos_mode();
  }

  pub fn exit_dos_mode(&mut self, index: usize) {
    let vterm = match self.vterm_list.get_mut(index) {
      Some(v) => v,
      None => return,
    };
    vterm.exit_dos_mode();
  }

  pub fn add_memory_backup(&mut self, index: usize, address: usize) -> PhysicalAddress {
    let vterm = match self.vterm_list.get_mut(index) {
      Some(v) => v,
      None => panic!("Adding memory backup to invalid vterm"),
    };
    let backup = vterm.add_memory_backup(address);
    backup.get_buffer_physical_address()
  }

  pub fn send_key_action(&mut self, action: KeyAction) {
    if self.key_state.alt {
      match action {
        KeyAction::Press(KeyCode::Num0) => {
          self.set_active_vterm(0);
          return;
        },
        KeyAction::Press(KeyCode::Num1) => {
          self.set_active_vterm(1);
          return;
        },
        KeyAction::Press(KeyCode::Num2) => {
          self.set_active_vterm(2);
          return;
        },
        KeyAction::Press(KeyCode::Num3) => {
          self.set_active_vterm(3);
          return;
        },
        KeyAction::Press(KeyCode::Num4) => {
          self.set_active_vterm(4);
          return;
        },
        KeyAction::Press(KeyCode::Num5) => {
          self.set_active_vterm(5);
          return;
        },
        KeyAction::Press(KeyCode::Num6) => {
          self.set_active_vterm(6);
          return;
        },
        KeyAction::Press(KeyCode::Num7) => {
          self.set_active_vterm(7);
          return;
        },
        KeyAction::Press(KeyCode::Num8) => {
          self.set_active_vterm(8);
          return;
        },
        KeyAction::Press(KeyCode::Num9) => {
          self.set_active_vterm(9);
          return;
        },
        _ => (),
      }
    }
    let mut input_buffer: [u8; 4] = [0; 4];
    let output = self.key_state.process_key_action(action, &mut input_buffer);
    if let Some(len) = output {
      let current_term = match self.vterm_list.get_mut(self.active_vterm) {
        Some(v) => v,
        None => return,
      };
      current_term.handle_input(&input_buffer[0..len]);
    }
  }

  pub fn process_buffers(&mut self) {
    let mut data: [u8; 4] = [0; 4];
    for vterm in self.vterm_list.iter_mut() {
      let tty_index = vterm.get_tty_index();
      let write_buffer = crate::tty::device::get_write_buffer(tty_index);

      let mut to_read = write_buffer.available_bytes();
      while to_read > 0 {
        let bytes_read = write_buffer.read(&mut data);
        to_read = if bytes_read == data.len() {
          to_read - bytes_read
        } else {
          0
        };
        vterm.send_characters(&data[0..bytes_read]);
      }
    }
  }

  pub fn write_to_console(&mut self, s: &str) {
    let vterm_zero = self.vterm_list.get_mut(0);
    let console = match vterm_zero {
      Some(vterm) => vterm,
      None => return,
    };
    console.send_characters(s.as_bytes());
  }
}