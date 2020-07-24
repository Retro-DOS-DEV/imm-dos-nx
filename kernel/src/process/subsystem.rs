use super::process_state::ProcessState;

#[derive(Copy, Clone)]
pub enum Subsystem {
  Native,
  DOS(DosSubsystemMetadata),
}

#[derive(Copy, Clone)]
pub struct DosSubsystemMetadata {
  // Real-mode segments
  pub ds: usize,
  pub es: usize,
  pub fs: usize,
  pub gs: usize,
  pub ss: usize,

  pub interrupts_enabled: bool,
}

impl DosSubsystemMetadata {
  pub const fn new() -> DosSubsystemMetadata {
    DosSubsystemMetadata {
      ds: 0,
      es: 0,
      fs: 0,
      gs: 0,
      ss: 0,
      interrupts_enabled: false,
    }
  }
}

impl ProcessState {
  pub fn is_vm8086(&self) -> bool {
    if let Subsystem::DOS(_) = *self.get_subsystem().read() {
      true
    } else {
      false
    }
  }

  pub fn get_vm8086_metadata(&self) -> Option<DosSubsystemMetadata> {
    if let Subsystem::DOS(meta) = *self.get_subsystem().read() {
      Some(meta)
    } else {
      None
    }
  }
}
