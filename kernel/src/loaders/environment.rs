use alloc::vec::Vec;
use crate::task::memory::{ExecutionSegment, Relocation};

pub struct InitialRegisters {
  pub eax: Option<u32>,
  
  pub eip: Option<u32>,
  pub esp: Option<u32>,

  pub cs: Option<u32>,
  pub ds: Option<u32>,
  pub es: Option<u32>,
  pub ss: Option<u32>,
}

pub struct ExecutionEnvironment {
  pub segments: Vec<ExecutionSegment>,
  pub relocations: Vec<Relocation>,
  pub registers: InitialRegisters,
  pub require_vm: bool,
}