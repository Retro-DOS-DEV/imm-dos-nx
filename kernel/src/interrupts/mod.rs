pub mod exceptions;
pub mod stack;

pub fn cli() {
  unsafe {
    asm!("cli" : : : : "volatile");
  }
}

pub fn sti() {
  unsafe {
    asm!("sti" : : : : "volatile");
  }
}

#[macro_export]
macro_rules! interrupt {
  ($method: expr) => {
    {
      #[naked]
      unsafe extern "C" fn wrap_interrupt() -> ! {
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
        asm!("mov ebx, esp
              add ebx, 28
              push ebx
              call $0" : :
              "{eax}"($method as usize) :
              "ebx", "esp" :
              "intel", "volatile"
        );
        // return from the interrupt
        asm!("add esp, 4" : : : "esp" : "intel", "volatile");
        asm!("pop edi
              pop esi
              pop ebp
              pop ebx
              pop edx
              pop ecx
              pop eax
              iretd" : : :
              "esp" :
              "intel", "volatile"
        );

        core::intrinsics::unreachable();
      }
      wrap_interrupt
    }
  }
}

#[macro_export]
macro_rules! interrupt_with_error {
  ($method: expr) => {
    {
      #[naked]
      unsafe extern "C" fn wrap_interrupt() -> ! {
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
        asm!("mov ebx, esp
              add ebx, 28
              push [ebx]
              add ebx, 4
              push ebx
              call $0" : :
              "{eax}"($method as usize) :
              "ebx", "esp" :
              "intel", "volatile"
        );
        // return from the interrupt
        asm!("add esp, 8" : : : "esp" : "intel", "volatile");
        asm!("pop edi
              pop esi
              pop ebp
              pop ebx
              pop edx
              pop ecx
              pop eax
              add esp, 4
              iretd" : : :
              "esp" :
              "intel", "volatile"
        );

        core::intrinsics::unreachable();
      }
      wrap_interrupt
    }
  }
}