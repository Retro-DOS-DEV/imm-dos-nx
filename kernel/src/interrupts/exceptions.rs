use crate::kprintln;
use super::stack::StackFrame;

pub extern "C" fn divide_by_zero(stack_frame: &StackFrame) -> ! {
  kprintln!("\nERR: Divide By Zero\n{:?}", stack_frame);
  loop {}
}

pub extern "C" fn double_fault(stack_frame: &StackFrame) -> ! {
  kprintln!("\nERR: Double Fault\n{:?}", stack_frame);
  loop {}
}

pub extern "C" fn gpf(_stack_frame: &StackFrame) -> ! {
  kprintln!("\nERR: General Protection Fault");
  loop {}
}
