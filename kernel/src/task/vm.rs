//! A process can simulate a DOS 8086 environment to execute DOS programs.
use crate::dos::state::VMState;

pub enum Subsystem {
  Native,
  DOS(VMState),
}