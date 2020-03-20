pub mod exceptions;
pub mod pic;
pub mod stack;
pub mod syscall;
pub mod syscall_legacy;

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

#[inline]
pub fn is_interrupt_enabled() -> bool {
  let flags: u32;
  unsafe {
    asm!("pushfd; pop $0" : "=r"(flags) : : : "intel", "volatile");
  }
  flags & 0x200 == 0x200
}
