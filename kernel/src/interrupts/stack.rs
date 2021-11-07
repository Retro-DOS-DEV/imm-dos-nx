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

impl StackFrame {
  unsafe fn as_ptr(&self) -> *mut u32 {
    self as *const StackFrame as usize as *mut u32
  }

  pub unsafe fn set_eip(&self, eip: u32) {
    core::ptr::write_volatile(self.as_ptr(), eip);
  }

  pub unsafe fn add_eip(&self, delta: i32) {
    let value = (self.eip as i32 + delta) as u32;
    self.set_eip(value);
  }

  pub unsafe fn set_cs(&self, cs: u32) {
    core::ptr::write_volatile(self.as_ptr().offset(1), cs);
  }

  pub unsafe fn set_eflags(&self, flags: u32) {
    core::ptr::write_volatile(self.as_ptr().offset(2), flags);
  }

  pub unsafe fn set_carry_flag(&self) {
    let flags = core::ptr::read_volatile(self.as_ptr().offset(2));
    core::ptr::write_volatile(self.as_ptr().offset(2), flags | 1);
  }

  pub unsafe fn clear_carry_flag(&self) {
    let flags = core::ptr::read_volatile(self.as_ptr().offset(2));
    core::ptr::write_volatile(self.as_ptr().offset(2), flags & 0xfffffffe);
  }
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
