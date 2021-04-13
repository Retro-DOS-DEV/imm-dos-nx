use core::fmt;

/// Each interrupt and exception places this structure on the stack, so that the
/// previously running code can be re-entered when the interrupt ends.
#[repr(C, packed)]
pub struct StackFrame {
  pub eip: u32,
  pub cs: u32,
  pub eflags: u32,
}

impl fmt::Debug for StackFrame {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let eip = self.eip;
    let cs = self.cs;
    let eflags = self.eflags;
    write!(
      f,
      "StackFrame {{\n  eip: {:#x}\n  cs: {:#x}\n  eflags: {:b}\n}}\n",
      eip,
      cs,
      eflags,
    )
  }
}
