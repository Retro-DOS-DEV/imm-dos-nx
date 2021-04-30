use core::fmt;
use crate::task::regs::SavedState;

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

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct FullStackFrame {
  pub eip: usize,
  pub cs: usize,
  pub eflags: usize,
  pub esp: usize,
  pub ss: usize,
}

impl FullStackFrame {
  pub fn empty() -> Self {
    Self {
      eip: 0,
      cs: 0,
      eflags: 0,
      esp: 0,
      ss: 0,
    }
  }
}

#[repr(C, packed)]
pub struct RestorationStack {
  pub regs: SavedState,
  pub frame: FullStackFrame,
}

impl RestorationStack {
  pub fn empty() -> Self {
    Self {
      regs: SavedState::empty(),
      frame: FullStackFrame::empty(),
    }
  }
}
