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

pub extern "C" fn gpf(_stack_frame: &StackFrame, error: u32) -> ! {
  kprintln!("\nERR: General Protection Fault, code {}", error);
  loop {}
}

pub extern "C" fn page_fault(stack_frame: &StackFrame, error: u32) {
  let address: usize;
  unsafe {
    asm!("mov $0, cr2" : "=r"(address) : : : "intel", "volatile");
  }
  kprintln!("\nPage Fault at {:#010x} {:x}:", address, error);
  if error & 1 == 0 {
    kprintln!("  PAGE NOT PRESENT");
  }
  if error & 2 == 2 {
    kprintln!("  WRITE ATTEMPTED");
  } else {
    kprintln!("  READ ATTEMPTED");
  }
  if error & 4 == 4 {
    kprintln!("  AT RING 3");
  }
  if error & 16 == 16 {
    kprintln!("  INSTRUCTION FETCH");
  }
  kprintln!("{:?}", stack_frame);
  loop {}
}
