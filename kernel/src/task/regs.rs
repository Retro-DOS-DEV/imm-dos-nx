#[repr(C, packed)]
pub struct SavedRegisters {
  flags: u32,
  edi: u32,
  esi: u32,
  ebp: u32,
  ebx: u32,
  edx: u32,
  ecx: u32,
  eax: u32,
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
}
