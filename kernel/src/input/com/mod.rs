pub mod device;
pub mod serial;

use crate::memory::address::VirtualAddress;
use crate::task::id::ProcessID;

pub fn init() {
  let com1 = device::ComDevice::new(0x3f8);
  com1.init();
  let com2 = device::ComDevice::new(0x2f8);
  com2.init();
  unsafe {
    device::COM_DEVICES[0] = Some(com1);
    device::COM_DEVICES[1] = Some(com2);
  }

  crate::kprintln!("Install COM handlers");

  let install_result = crate::interrupts::handlers::install_handler(
    4,
    ProcessID::new(0),
    VirtualAddress::new(int_com1 as *const fn () -> () as usize),
    VirtualAddress::new(0),
  );
  if let Err(_) = install_result {
    crate::kprintln!("Failed to install IRQ4");
  }
}

pub extern "C" fn int_com1() {
  handle_interrupt(0);
  crate::interrupts::handlers::return_from_handler(4);
}

pub extern "C" fn int_com2() {
  handle_interrupt(1);
  crate::interrupts::handlers::return_from_handler(3);
}

pub fn handle_interrupt(index: usize) {
  use crate::devices::queue::QueuedIO;

  let driver = unsafe {
    &device::COM_DEVICES[index]
  };
  if let Some(com) = driver {
    let interrupt_info = com.get_interrupt_info();
    if interrupt_info & 4 != 0 { // Received data available
      com.wake_front();
    }
  }
}

pub fn get_device(index: usize) -> &'static device::ComDevice {
  unsafe {
    &device::COM_DEVICES[index].as_ref().unwrap()
  }
}