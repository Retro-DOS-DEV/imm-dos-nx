#[derive(Copy, Clone)]
#[repr(u8)]
pub enum KeyCode {
  None = 0x00,

  Delete = 0x07,
  Backspace = 0x08,
  Tab = 0x09,

  Enter = 0x0d,

  Caps = 0x10,
  Shift = 0x11,
  Control = 0x12,
  Menu = 0x13,
  Alt = 0x14,

  Escape = 0x1b,

  Space = 0x20,
  ArrowLeft = 0x21,
  ArrowUp = 0x22,
  ArrowRight = 0x23,
  ArrowDown = 0x24,

  Comma = 0x2c,
  Minus = 0x2d,
  Period = 0x2e,
  Slash = 0x2f,
  Num0 = 0x30,
  Num1 = 0x31,
  Num2 = 0x32,
  Num3 = 0x33,
  Num4 = 0x34,
  Num5 = 0x35,
  Num6 = 0x36,
  Num7 = 0x37,
  Num8 = 0x38,
  Num9 = 0x39,
  Semicolon = 0x3a,
  Quote = 0x3b,
  LessThan = 0x3c,
  Equals = 0x3d,
  GreaterThan = 0x3e,

  A = 0x41,
  B = 0x42,
  C = 0x43,
  D = 0x44,
  E = 0x45,
  F = 0x46,
  G = 0x47,
  H = 0x48,
  I = 0x49,
  J = 0x4a,
  K = 0x4b,
  L = 0x4c,
  M = 0x4d,
  N = 0x4e,
  O = 0x4f,
  P = 0x50,
  Q = 0x51,
  R = 0x52,
  S = 0x53,
  T = 0x54,
  U = 0x55,
  V = 0x56,
  W = 0x57,
  X = 0x58,
  Y = 0x59,
  Z = 0x5a,
  BracketLeft = 0x5b,
  Backslash = 0x5c,
  BracketRight = 0x5d,

  Backtick = 0x5f,
}

pub const US_LAYOUT: [(u8, u8); 0x60] = [
  (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0x7f, 0x7f),
  (0x08, 0x08), (0x09, 0x09), (0, 0), (0, 0), (0, 0), (0x0a, 0x0a), (0, 0), (0, 0),
  (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
  (0, 0), (0, 0), (0, 0), (0x1b, 0x1b), (0, 0), (0, 0), (0, 0), (0, 0),
  (0x20, 0x20), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
  (0, 0), (0, 0), (0, 0), (0, 0), (0x2c, 0x3c), (0x2d, 0x5f), (0x2e, 0x3e), (0x2f, 0x3f),
  (0x30, 0x29), (0x31, 0x21), (0x32, 0x40), (0x33, 0x23), (0x34, 0x24), (0x35, 0x25), (0x36, 0x5e), (0x37, 0x26),
  (0x38, 0x2a), (0x39, 0x28), (0x3b, 0x3a), (0x27, 0x22), (0, 0), (0x3d, 0x2b), (0, 0), (0, 0),
  (0, 0), (0x61, 0x41), (0x62, 0x42), (0x63, 0x43), (0x64, 0x44), (0x65, 0x45), (0x66, 0x46), (0x67, 0x47),
  (0x68, 0x48), (0x69, 0x49), (0x6a, 0x4a), (0x6b, 0x4b), (0x6c, 0x4c), (0x6d, 0x4d), (0x6e, 0x4e), (0x6f, 0x4f),
  (0x70, 0x50), (0x71, 0x51), (0x72, 0x52), (0x73, 0x53), (0x74, 0x54), (0x75, 0x55), (0x76, 0x56), (0x77, 0x57),
  (0x78, 0x58), (0x79, 0x59), (0x7a, 0x5a), (0x5b, 0x7b), (0x5c, 0x7c), (0x5d, 0x7d), (0, 0), (0x60, 0x7e),
];

pub const SCANCODES_TO_KEYCODES: [KeyCode; 60] = [
  KeyCode::None, KeyCode::Escape, KeyCode::Num1, KeyCode::Num2,
  KeyCode::Num3, KeyCode::Num4, KeyCode::Num5, KeyCode::Num6,
  KeyCode::Num7, KeyCode::Num8, KeyCode::Num9, KeyCode::Num0,
  KeyCode::Minus, KeyCode::Equals, KeyCode::Backspace, KeyCode::Tab,
  KeyCode::Q, KeyCode::W, KeyCode::E, KeyCode::R,
  KeyCode::T, KeyCode::Y, KeyCode::U, KeyCode::I,
  KeyCode::O, KeyCode::P, KeyCode::BracketLeft, KeyCode::BracketRight,
  KeyCode::Enter, KeyCode::Control, KeyCode::A, KeyCode::S,
  KeyCode::D, KeyCode::F, KeyCode::G, KeyCode::H,
  KeyCode::J, KeyCode::K, KeyCode::L, KeyCode::Semicolon,
  KeyCode::Quote, KeyCode::Backtick, KeyCode::Shift, KeyCode::Backslash,
  KeyCode::Z, KeyCode::X, KeyCode::C, KeyCode::V,
  KeyCode::B, KeyCode::N, KeyCode::M, KeyCode::Comma,
  KeyCode::Period, KeyCode::Slash, KeyCode::Shift, KeyCode::None,
  KeyCode::Alt, KeyCode::Space, KeyCode::Caps, KeyCode::None,
];

pub fn get_keycode(scan_code: u8) -> KeyCode {
  if scan_code < 60 {
    SCANCODES_TO_KEYCODES[scan_code as usize]
  } else {
    KeyCode::None
  }
}

pub fn get_extended_keycode(scan_code: u8) -> KeyCode {
  match scan_code {
    0x1c => KeyCode::Enter,
    0x48 => KeyCode::ArrowUp,
    0x4b => KeyCode::ArrowLeft,
    0x4d => KeyCode::ArrowRight,
    0x50 => KeyCode::ArrowDown,
    _ => KeyCode::None,
  }
}
