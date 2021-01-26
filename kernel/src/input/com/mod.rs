pub mod device;
pub mod serial;

pub fn init() {
  let com1 = device::ComDriver::new(0x3f8);
  com1.init();
  let com2 = device::ComDriver::new(0x2f8);
  com2.init();
  unsafe {
    device::COM_DEVICES[0] = Some(com1);
    device::COM_DEVICES[1] = Some(com2);
  }
}

pub fn handle_interrupt(index: usize) {
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

pub fn get_device(index: usize) -> &'static device::ComDriver {
  unsafe {
    &device::COM_DEVICES[index].as_ref().unwrap()
  }
}