use crate::{devices, kprint, time};
use super::stack;

pub extern "x86-interrupt" fn pit(_frame: &stack::StackFrame) {
  let prev = time::get_offset_seconds();
  time::increment_offset(100002);
  let updated = time::get_offset_seconds();
  if prev != updated {
    kprint!("{:x}", _frame.cs);
  }

  unsafe {
    devices::PIC.acknowledge_interrupt(0);
  }
}
