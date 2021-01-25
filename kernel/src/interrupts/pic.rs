use crate::{devices, input, task, time, x86};
use super::stack;

pub extern "x86-interrupt" fn pit(_frame: &stack::StackFrame) {
  time::system::increment_offset(time::system::HUNDRED_NS_PER_TICK);
  task::switching::update_timeouts(time::system::MS_PER_TICK);

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

    devices::PIC.acknowledge_interrupt(1);
  }
}

pub extern "x86-interrupt" fn com1(_frame: &stack::StackFrame) {
  unsafe {
    devices::COM1.handle_interrupt();
    devices::PIC.acknowledge_interrupt(4);
  }
}



pub extern "x86-interrupt" fn floppy(_frame: &stack::StackFrame) {
  unsafe {
    devices::FLOPPY.handle_int6();
    devices::PIC.acknowledge_interrupt(6);
  }
}
