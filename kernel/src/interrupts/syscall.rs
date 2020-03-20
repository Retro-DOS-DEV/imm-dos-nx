use crate::kprintln;
use super::stack;

#[repr(C, packed)]
struct SavedRegisters {
  edi: u32,
  esi: u32,
  ebp: u32,
  ebx: u32,
  edx: u32,
  ecx: u32,
  eax: u32,
}

#[naked]
pub unsafe extern "x86-interrupt" fn syscall_handler(_frame: &stack::StackFrame) {
  asm!("push eax
        push ecx
        push edx
        push ebx
        push ebp
        push esi
        push edi" : : :
        "esp" :
        "intel", "volatile"
  );

  asm!("mov ebx, esp; push ebx; call syscall_inner" : : : "ebx" : "intel", "volatile");

  asm!("add esp, 4
        pop edi
        pop esi
        pop ebp
        pop ebx
        pop edx
        pop ecx
        pop eax" : : : :
        "intel", "volatile"
  );
}

#[no_mangle]
unsafe extern "C" fn syscall_inner(esp: u32) {
  let registers: &SavedRegisters = &*(esp as *const SavedRegisters);
  kprintln!("Syscall Handler. {:#x} {:#x} {:#x} {:#x} {:#x}", registers.eax, registers.ebx, registers.ecx, registers.edx, registers.edi);
}