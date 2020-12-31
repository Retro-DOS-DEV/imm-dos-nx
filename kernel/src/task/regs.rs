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
