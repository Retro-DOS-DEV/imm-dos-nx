use crate::kprintln;

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

#[repr(C, packed)]
pub struct VM8086Frame {
  pub sp: u32,
  pub ss: u32,
  pub es: u32,
  pub ds: u32,
  pub fs: u32,
  pub gs: u32,
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
}

/**
 * Interrupts to support legacy DOS API calls
 */
pub fn dos_api(regs: &mut DosApiRegisters, frame: &mut VM8086Frame) {
  match (regs.ax & 0xff00) >> 8 {
    0x02 => {
      // print char to stdout
      kprintln!("PRINTDOS");
      regs.ax = (regs.ax & 0xff00) | (regs.dx & 0xff);
    },
    _ => (),
  }
}