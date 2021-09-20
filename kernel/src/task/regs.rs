/// SavedState stashes the running state of a task when it is interrupted.
/// Restoring these would allow the CPU to return to its pre-interrupt state
/// without the task ever knowing.
/// When an interrupt actually occurs, the handler immediately pushes these
/// values onto the kernel stack. However, we can't keep them there. If a task
/// were to execute a custom interrupt handler and, during that, encounter an
/// exception, the original values on the stack would be clobbered.
/// To ensure that we always have a safe set of values to return to, each
/// interrupt handler copies the stack-stored values to a SavedState on the
/// task's state object.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SavedState {
  edi: usize,
  esi: usize,
  ebp: usize,
  ebx: usize,
  edx: usize,
  ecx: usize,
  eax: usize,
}

impl SavedState {
  pub const fn empty() -> Self {
    Self {
      edi: 0,
      esi: 0,
      ebp: 0,
      ebx: 0,
      edx: 0,
      ecx: 0,
      eax: 0,
    }
  }
}

impl core::fmt::Debug for SavedState {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let eax = self.eax;
    let ebx = self.ebx;
    let ecx = self.ecx;
    let edx = self.edx;
    let ebp = self.ebp;
    let esi = self.esi;
    let edi = self.edi;
    f.debug_struct("Saved Registers")
      .field("eax", &format_args!("{:#010x}", eax))
      .field("ebx", &format_args!("{:#010x}", ebx))
      .field("ecx", &format_args!("{:#010x}", ecx))
      .field("edx", &format_args!("{:#010x}", edx))
      .field("ebp", &format_args!("{:#010x}", ebp))
      .field("esi", &format_args!("{:#010x}", esi))
      .field("edi", &format_args!("{:#010x}", edi))
      .finish()
  }
}

#[repr(C, packed)]
pub struct EnvironmentRegisters {
  pub eax: u32,
  pub ecx: u32,
  pub edx: u32,
  pub ebx: u32,
  pub ebp: u32,
  pub esi: u32,
  pub edi: u32,

  // Registers that get popped by IRETD
  pub eip: u32,
  pub cs: u32,
  pub flags: u32,
  pub esp: u32,
  pub ss: u32,

  pub es: u32,
  pub ds: u32,
  pub fs: u32,
  pub gs: u32,
}
