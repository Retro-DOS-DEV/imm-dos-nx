
pub enum Subsystem {
  Native,
  DOS(DosSubsystemMetadata),
}

pub struct DosSubsystemMetadata {
  // Real-mode segments
  ds: usize,
  es: usize,
  fs: usize,
  gs: usize,
  ss: usize,

  interrupts_enabled: bool,
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
