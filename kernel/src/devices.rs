use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::drivers::{self, com::serial::SerialPort};
use crate::hardware::{dma, floppy, pic, pit, rtc};
use crate::hardware::vga::text_mode;
use crate::memory::address::VirtualAddress;
use crate::tty;
use spin::{Mutex, RwLock};

pub static mut PIC: pic::PIC = pic::PIC::new();
pub static mut PIT: pit::PIT = pit::PIT::new();
pub static RTC: rtc::RTC = rtc::RTC::new();
pub static mut VGA_TEXT: text_mode::TextMode = text_mode::TextMode::new(VirtualAddress::new(0xc00b8000));

pub static mut KEYBOARD: Option<Arc<Mutex<drivers::keyboard::Keyboard>>> = None;
pub static COM1: SerialPort = SerialPort::new(0x3f8);
static mut COM1_DIRECT: SerialPort = SerialPort::new(0x3f8);

pub static DMA: dma::DMA = dma::DMA::new();
pub static FLOPPY: floppy::FloppyController = floppy::FloppyController::new();

pub static DEV: RwLock<drivers::DeviceDrivers> = RwLock::new(drivers::DeviceDrivers::new());

pub unsafe fn init() {
  PIC.init();
  PIT.set_divider(11932); // approximately 100Hz

  {
    let mut drivers = DEV.write();
    drivers.register_driver("ZERO", Arc::new(Box::new(drivers::zero::ZeroDevice::new())));
    drivers.register_driver("NULL", Arc::new(Box::new(drivers::null::NullDevice::new())));
    drivers.register_driver("COM1", Arc::new(Box::new(drivers::com::ComDevice::new(&COM1))));
    
    let kbd = Arc::new(Mutex::new(drivers::keyboard::Keyboard::new()));
    let kbd_clone = Arc::clone(&kbd);
    KEYBOARD = Some(kbd);
    drivers.register_driver("KBD", Arc::new(Box::new(drivers::keyboard::KeyboardDevice::new(kbd_clone))));

    drivers.register_driver("TTY0", Arc::new(Box::new(tty::device::TTYDevice::for_tty(0))));
    drivers.register_driver("TTY1", Arc::new(Box::new(tty::device::TTYDevice::for_tty(1))));

    COM1.init();
  }
}

pub fn get_device_number_by_name(filename: &[u8; 8]) -> Option<usize> {
  let drivers = DEV.read();
  drivers.get_device_number_by_name(filename)
}

pub unsafe fn get_raw_serial() -> &'static mut SerialPort {
  &mut COM1_DIRECT
}

pub fn get_driver_for_device(number: usize) -> Option<Arc<Box<drivers::DriverType>>> {
  let drivers = DEV.read();
  match drivers.get_device(number) {
    Some(driver) => Some(driver.clone()),
    None => None,
  }
}