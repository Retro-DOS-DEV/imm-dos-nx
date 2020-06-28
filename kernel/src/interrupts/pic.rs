use crate::{devices, process, time};
use super::stack;

pub extern "x86-interrupt" fn pit(_frame: &stack::StackFrame) {
  time::increment_offset(time::HUNDRED_NS_PER_TICK);
  process::send_tick();
  
  unsafe {
    devices::PIC.acknowledge_interrupt(0);
  }
}
