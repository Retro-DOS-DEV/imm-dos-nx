use alloc::vec::Vec;
use crate::hardware::vga::text_mode::Color;

/// A state machine that tracks the current parsing state of multi-byte ANSI
/// codes.
pub struct Parser {
  state: ParseState,
  csi_args: Vec<Option<u32>>,
}

/// Tracks the current state in the Parser state machine
#[derive(Copy, Clone)]
pub enum ParseState {
  /// Initial state, ready to read any character
  Ready,
  /// Recognized an ESC sequence
  EscapeStart,
  /// Recognized a full CSI sequence
  CSI,
}

#[derive(Copy, Clone)]
pub enum TTYAction {
  None,
  Print(u8),
  MoveCursor(isize, isize),
  SetColumn(usize),
  SetPosition(usize, usize),
  ClearScreen,
  ClearToBeginning,
  ClearToEnd,
  ClearRow,
  ClearRowToBeginning,
  ClearRowToEnd,
  NextLineStart(usize),
  PrevLineStart(usize),
  ScrollUp(usize),
  ScrollDown(usize),
  ResetColors,
  SetFgColor(Color),
  SetBgColor(Color),
}

impl Parser {
  pub fn new() -> Self {
    Self {
      state: ParseState::Ready,
      csi_args: Vec::new(),
    }
  }

  pub fn get_csi_arg(&self, index: usize, fallback: u32) -> u32 {
    match self.csi_args.get(index) {
      Some(opt) => match opt {
        Some(val) => *val,
        None => fallback,
      },
      None => fallback,
    }
  }

  pub fn process_character(&mut self, ch: u8) -> TTYAction {
    match self.state {
      ParseState::Ready => {
        match ch {
          0x1b => {
            self.state = ParseState::EscapeStart;
            return TTYAction::None;
          },
          _ => return TTYAction::Print(ch),
        }
      },
      ParseState::EscapeStart => {
        match ch {
          0x5b => {
            self.state = ParseState::CSI;
            while !self.csi_args.is_empty() {
              self.csi_args.pop();
            }
            self.csi_args.push(None);
            return TTYAction::None;
          },
          _ => {
            self.state = ParseState::Ready;
            return TTYAction::None;
          }
        }
      },
      ParseState::CSI => {
        let (action, done) = match ch {
          b'0'..=b'9' => {
            // arguments are pushed in ascii digits
            let digit = (ch - 48) as u32;
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
            (TTYAction::None, false)
          },
          b';' => {
            self.csi_args.push(None);
            (TTYAction::None, false)
          },
          b'A' => { // Cursor Up
            let delta = self.get_csi_arg(0, 1);
            (TTYAction::MoveCursor(0, delta as isize * -1), true)
          },
          b'B' => { // Cursor Down
            let delta = self.get_csi_arg(0, 1);
            (TTYAction::MoveCursor(0, delta as isize), true)
          },
          b'C' => { // Cursor Forward
            let delta = self.get_csi_arg(0, 1);
            (TTYAction::MoveCursor(delta as isize, 0), true)
          },
          b'D' => { // Cursor Back
            let delta = self.get_csi_arg(0, 1);
            (TTYAction::MoveCursor(delta as isize * -1, 0), true)
          },
          b'E' => { // Cursor to next line start
            let delta = self.get_csi_arg(0, 1);
            (TTYAction::NextLineStart(delta as usize), true)
          },
          b'F' => { // Cursor to previous line start
            let delta = self.get_csi_arg(0, 1);
            (TTYAction::PrevLineStart(delta as usize), true)
          },
          b'G' => { // Cursor to col
            let col = self.get_csi_arg(0, 1);
            (TTYAction::SetColumn(col as usize), true)
          },
          b'H' => { // Cursor to position
            let row = self.get_csi_arg(0, 1);
            let col = self.get_csi_arg(1, 1);
            (TTYAction::SetPosition(row as usize, col as usize), true)
          },
          b'J' => { // Clear screen
            let direction = self.get_csi_arg(0, 0);
            let action = match direction {
              0 => TTYAction::ClearToEnd,
              1 => TTYAction::ClearToBeginning,
              2 | 3 => TTYAction::ClearScreen,
              _ => TTYAction::None,
            };
            (action, true)
          },
          b'K' => { // Clear in line
            let direction = self.get_csi_arg(0, 0);
            let action = match direction {
              0 => TTYAction::ClearRowToEnd,
              1 => TTYAction::ClearRowToBeginning,
              2 | 3 => TTYAction::ClearRow,
              _ => TTYAction::None,
            };
            (action, true)
          },
          b'S' => { // Scroll Up
            let delta = self.get_csi_arg(0, 1);
            (TTYAction::ScrollUp(delta as usize), true)
          },
          b'T' => { // Scroll Down
            let delta = self.get_csi_arg(0, 1);
            (TTYAction::ScrollDown(delta as usize), true)
          },
          
          b'm' => { // Select Graphic Rendition
            let modifier = self.get_csi_arg(0, 0);
            let action = match modifier {
              0 => TTYAction::ResetColors,

              30 => TTYAction::SetFgColor(Color::Black),
              31 => TTYAction::SetFgColor(Color::Red),
              32 => TTYAction::SetFgColor(Color::Green),
              33 => TTYAction::SetFgColor(Color::Brown),
              34 => TTYAction::SetFgColor(Color::Blue),
              35 => TTYAction::SetFgColor(Color::Magenta),
              36 => TTYAction::SetFgColor(Color::Cyan),
              37 => TTYAction::SetFgColor(Color::LightGrey),

              39 => TTYAction::SetFgColor(Color::LightGrey),

              40 => TTYAction::SetBgColor(Color::Black),
              41 => TTYAction::SetBgColor(Color::Red),
              42 => TTYAction::SetBgColor(Color::Green),
              43 => TTYAction::SetBgColor(Color::Brown),
              44 => TTYAction::SetBgColor(Color::Blue),
              45 => TTYAction::SetBgColor(Color::Magenta),
              46 => TTYAction::SetBgColor(Color::Cyan),
              47 => TTYAction::SetBgColor(Color::LightGrey),

              49 => TTYAction::SetBgColor(Color::Black),

              90 => TTYAction::SetFgColor(Color::DarkGrey),
              91 => TTYAction::SetFgColor(Color::LightRed),
              92 => TTYAction::SetFgColor(Color::LightGreen),
              93 => TTYAction::SetFgColor(Color::LightBrown),
              94 => TTYAction::SetFgColor(Color::LightBlue),
              95 => TTYAction::SetFgColor(Color::LightMagenta),
              96 => TTYAction::SetFgColor(Color::LightCyan),
              97 => TTYAction::SetFgColor(Color::White),

              100 => TTYAction::SetBgColor(Color::DarkGrey),
              101 => TTYAction::SetBgColor(Color::LightRed),
              102 => TTYAction::SetBgColor(Color::LightGreen),
              103 => TTYAction::SetBgColor(Color::LightBrown),
              104 => TTYAction::SetBgColor(Color::LightBlue),
              105 => TTYAction::SetBgColor(Color::LightMagenta),
              106 => TTYAction::SetBgColor(Color::LightCyan),
              107 => TTYAction::SetBgColor(Color::White),

              _ => TTYAction::None,
            };
            (action, true)
          },

          _ => (TTYAction::None, true),
        };
        if done {
          self.state = ParseState::Ready;
        }
        return action;
      },
    }
  }
}
