/// Stores the emulated state of a DOS VM
pub struct VMState {
  pub current_psp: u16,
}

impl VMState {
  pub fn new() -> Self {

    Self {
      current_psp: 0,
    }
  }
}