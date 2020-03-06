use crate::hardware::vga::text_mode;

pub static mut VGA_TEXT: text_mode::TextMode = text_mode::TextMode::new(0xb8000);
