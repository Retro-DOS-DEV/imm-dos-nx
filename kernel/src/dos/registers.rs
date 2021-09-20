use core::fmt;

#[repr(C, packed)]
pub struct DosApiRegisters {
  pub ax: u32,
  pub bx: u32,
  pub cx: u32,
  pub dx: u32,

  pub si: u32,
  pub di: u32,
  pub bp: u32,
}

impl DosApiRegisters {
  pub fn empty() -> DosApiRegisters {
    DosApiRegisters {
      ax: 0,
      bx: 0,
      cx: 0,
      dx: 0,

      si: 0,
      di: 0,
      bp: 0,
    }
  }

  pub fn ah(&self) -> u8 {
    ((self.ax & 0xff00) >> 8) as u8
  }

  pub fn al(&self) -> u8 {
    (self.ax & 0xff) as u8
  }

  pub fn set_al(&mut self, value: u8) {
    self.ax &= 0xff00;
    self.ax |= value as u32;
  }
}

/// When an interrupt occurs in VM86 mode, the stack pointer and segment
/// registers are pushed onto the stack before the typical stack frame.
#[repr(C, packed)]
pub struct VM86Frame {
  pub sp: u32,
  pub ss: u32,
  pub es: u32,
  pub ds: u32,
  pub fs: u32,
  pub gs: u32,
}

impl fmt::Debug for VM86Frame {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let sp = self.sp;
    let ss = self.ss;
    let es = self.es;
    let ds = self.ds;
    let fs = self.fs;
    let gs = self.gs;
    write!(
      f,
      "VM86Frame {{\n  sp: {:#x}\n  ss: {:#x}\n  es: {:#x}\n  ds: {:#x}\n  fs: {:#x}\n  gs: {:#x}\n}}\n",
      sp,
      ss,
      es,
      ds,
      fs,
      gs,
    )
  }
}