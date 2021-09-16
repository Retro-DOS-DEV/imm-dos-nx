
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