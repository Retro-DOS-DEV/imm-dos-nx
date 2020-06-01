pub mod exceptions;
pub mod pic;
pub mod stack;
pub mod syscall;
pub mod syscall_legacy;

pub fn cli() {
  unsafe {
    llvm_asm!("cli" : : : : "volatile");
  }
}

pub fn sti() {
  unsafe {
    llvm_asm!("sti" : : : : "volatile");
  }
}

#[inline]
pub fn is_interrupt_enabled() -> bool {
  let flags: u32;
  unsafe {
    llvm_asm!("pushfd; pop $0" : "=r"(flags) : : : "intel", "volatile");
  }
  flags & 0x200 == 0x200
}
