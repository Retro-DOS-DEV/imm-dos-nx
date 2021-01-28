use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::hardware::{dma, floppy, pic, pit, rtc};
use crate::hardware::vga::text_mode;
use crate::memory::address::VirtualAddress;
use spin::RwLock;

pub mod driver;
pub mod installed;

use installed::InstalledDevices;

pub static DEVICES: RwLock<InstalledDevices> = RwLock::new(InstalledDevices::new());

/// The PIC handles hardware interrupts and connects them to the CPU
pub static mut PIC: pic::PIC = pic::PIC::new();
/// The PIT is a configurable timer chip
pub static mut PIT: pit::PIT = pit::PIT::new();
/// The RTC is a real-time clock
pub static RTC: rtc::RTC = rtc::RTC::new();
/// The DMA controller configures direct access to memory for ISA devices
pub static DMA: dma::DMA = dma::DMA::new();
/// The floppy controller configures the disk drive
pub static FLOPPY: floppy::FloppyController = floppy::FloppyController::new();

pub static mut VGA_TEXT: text_mode::TextMode = text_mode::TextMode::new(VirtualAddress::new(0xc00b8000));

pub fn get_device_number_by_name(filename: &str) -> Option<usize> {
  let devices = DEVICES.read();
  devices.get_device_number_by_name(filename)
}

pub fn get_driver_for_device(number: usize) -> Option<Arc<Box<driver::DeviceDriverType>>> {
  let devices = DEVICES.read();
  match devices.get_device(number) {
    Some(driver) => Some(driver.clone()),
    None => None,
  }
}

pub fn init() {
  unsafe {
    PIC.init();
    PIT.set_divider(11932); // approximately 100Hz
  }

  {
    let mut all_devices = DEVICES.write();
    all_devices.register_driver("KBD", Arc::new(Box::new(crate::input::keyboard::device::KeyboardDriver {})));
    crate::input::com::init();

    // ZERO, NULL, COM, TTY, FD0
  }
}
