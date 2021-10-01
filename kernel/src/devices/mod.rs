use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::hardware::{dma, floppy, pic, pit, rtc};
use crate::hardware::vga::text_mode;
use crate::memory::address::VirtualAddress;
use spin::RwLock;

pub mod block;
pub mod driver;
pub mod installed;
pub mod null;
pub mod zero;

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
    all_devices.register_driver("COM1", Arc::new(Box::new(crate::input::com::device::ComDriver::new(0))));
    all_devices.register_driver("COM2", Arc::new(Box::new(crate::input::com::device::ComDriver::new(0))));
    all_devices.register_driver("NULL", Arc::new(Box::new(null::NullDriver::new())));
    all_devices.register_driver("ZERO", Arc::new(Box::new(zero::ZeroDriver::new())));

    let (has_primary_floppy, has_secondary_floppy) = block::floppy::init();
    if has_primary_floppy {
      all_devices.register_driver("FD1", Arc::new(Box::new(block::FloppyDriver::new(floppy::DriveSelect::Primary))));
    }
    if has_secondary_floppy {
      all_devices.register_driver("FD2", Arc::new(Box::new(block::FloppyDriver::new(floppy::DriveSelect::Secondary))));
    }
  }
}

pub fn create_tty(index: usize) {
  let mut all_devices = DEVICES.write();
  let name: alloc::string::String = alloc::format!("TTY{}", index);
  all_devices.register_driver(&name, Arc::new(Box::new(crate::tty::device::TTYDevice::for_tty(index))));
}
