use crate::{devices, input, process, time, x86};
use super::stack;

pub extern "x86-interrupt" fn pit(_frame: &stack::StackFrame) {
  time::system::increment_offset(time::system::HUNDRED_NS_PER_TICK);
  process::send_tick();
  
  unsafe {
    devices::PIC.acknowledge_interrupt(0);
  }
}

static KEYBOARD_PORT: x86::io::Port = x86::io::Port::new(0x60);

pub extern "x86-interrupt" fn keyboard(_frame: &stack::StackFrame) {
  unsafe {
    let mut data: [u8; 1] = [0; 1];
    data[0] = KEYBOARD_PORT.read_u8();
    input::INPUT_EVENTS.write(&data);
    input::wake_thread();

    devices::PIC.acknowledge_interrupt(1);
  }
}
