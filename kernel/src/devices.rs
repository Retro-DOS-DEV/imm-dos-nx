use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::drivers;
use crate::hardware::{pic, pit, rtc};
use crate::hardware::vga::text_mode;
use crate::memory::address::VirtualAddress;
use spin::{Mutex, RwLock};

pub static mut PIC: pic::PIC = pic::PIC::new();
pub static mut PIT: pit::PIT = pit::PIT::new();
pub static RTC: rtc::RTC = rtc::RTC::new();
pub static mut VGA_TEXT: text_mode::TextMode = text_mode::TextMode::new(VirtualAddress::new(0xc00b8000));

pub static mut KEYBOARD: Option<Arc<Mutex<drivers::keyboard::Keyboard>>> = None;
static mut COM1_DIRECT: drivers::com::SerialPort = drivers::com::SerialPort::new(0x3f8);

pub static DEV: RwLock<drivers::DeviceDrivers> = RwLock::new(drivers::DeviceDrivers::new());

pub unsafe fn init() {
  PIC.init();
  PIT.set_divider(11932); // approximately 100Hz

  {
    let mut drivers = DEV.write();
    drivers.register_driver("ZERO", Arc::new(Box::new(drivers::zero::ZeroDevice::new())));
    drivers.register_driver("NULL", Arc::new(Box::new(drivers::null::NullDevice::new())));
    drivers.register_driver("COM1", Arc::new(Box::new(drivers::com::ComDevice::new(0x3f8))));
    
    let kbd = Arc::new(Mutex::new(drivers::keyboard::Keyboard::new()));
    let kbd_clone = Arc::clone(&kbd);
    KEYBOARD = Some(kbd);
    drivers.register_driver("KBD", Arc::new(Box::new(drivers::keyboard::KeyboardDevice::new(kbd_clone))));
  }
}

pub fn get_device_number_by_name(filename: &[u8; 8]) -> Option<usize> {
  let drivers = DEV.read();
  drivers.get_device_number_by_name(filename)
}

pub unsafe fn get_raw_serial() -> &'static mut drivers::com::SerialPort {
  &mut COM1_DIRECT
}

pub fn get_driver_for_device(number: usize) -> Option<Arc<Box<drivers::DriverType>>> {
  let drivers = DEV.read();
  match drivers.get_device(number) {
    Some(driver) => Some(driver.clone()),
    None => None,
  }
}