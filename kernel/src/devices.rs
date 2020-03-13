use crate::hardware::{pic, pit};
use crate::hardware::vga::text_mode;

pub static mut PIC: pic::PIC = pic::PIC::new();
pub static mut PIT: pit::PIT = pit::PIT::new();
pub static mut VGA_TEXT: text_mode::TextMode = text_mode::TextMode::new(0xb8000);

pub unsafe fn init() {
  PIC.init();
  PIT.set_divider(11932); // approximately 100Hz
}