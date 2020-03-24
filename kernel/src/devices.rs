use alloc::boxed::Box;
use crate::drivers;
use crate::hardware::{pic, pit};
use crate::hardware::vga::text_mode;
use spin::RwLock;

pub static mut PIC: pic::PIC = pic::PIC::new();
pub static mut PIT: pit::PIT = pit::PIT::new();
pub static mut VGA_TEXT: text_mode::TextMode = text_mode::TextMode::new(0xb8000);

pub static DEV: RwLock<drivers::DeviceDrivers> = RwLock::new(drivers::DeviceDrivers::new());

pub unsafe fn init() {
  PIC.init();
  PIT.set_divider(11932); // approximately 100Hz

  {
    let mut drivers = DEV.write();
    drivers.register_driver(Box::new(drivers::zero::ZeroDevice::new()));
    drivers.register_driver(Box::new(drivers::null::NullDevice::new()));
    drivers.register_driver(Box::new(drivers::com::ComDevice::new(0x3f8)));
  }
}