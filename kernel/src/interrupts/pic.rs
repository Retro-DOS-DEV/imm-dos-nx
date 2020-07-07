use crate::{devices, process, time};
use super::stack;

pub extern "x86-interrupt" fn pit(_frame: &stack::StackFrame) {
  time::increment_offset(time::HUNDRED_NS_PER_TICK);
  process::send_tick();
  
  unsafe {
    devices::PIC.acknowledge_interrupt(0);
  }
}

pub extern "x86-interrupt" fn keyboard(_frame: &stack::StackFrame) {
  unsafe {
    match &devices::KEYBOARD {
      Some(keyboard) => {
        keyboard.lock().handle_interrupt();
      },
      None => (),
    }
    devices::PIC.acknowledge_interrupt(1);
  }
}
