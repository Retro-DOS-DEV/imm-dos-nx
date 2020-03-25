use alloc::boxed::Box;
use alloc::sync::Arc;
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
    drivers.register_driver("ZERO", Box::new(drivers::zero::ZeroDevice::new()));
    drivers.register_driver("NULL", Box::new(drivers::null::NullDevice::new()));
    drivers.register_driver("COM1", Box::new(drivers::com::ComDevice::new(0x3f8)));
  }
}

pub fn get_device_number_by_name(filename: &[u8; 8]) -> Option<usize> {
  let drivers = DEV.read();
  drivers.get_device_number_by_name(filename)
}

pub fn get_driver_for_device(number: usize) -> Option<Arc<Box<drivers::DriverType>>> {
  let drivers = DEV.read();
  match drivers.get_device(number) {
    Some(driver) => Some(driver.clone()),
    None => None,
  }
}