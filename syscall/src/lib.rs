#![feature(asm)]

#![no_std]

pub fn syscall_inner(method: u32, arg0: u32, arg1: u32) -> u32 {
  let result: u32;
  unsafe {
    asm!("int 0x2b" :
          "={eax}"(result) :
          "{eax}"(method), "{ebx}"(arg0), "{ecx}"(arg1) :
          "eax", "ebx", "ecx" :
          "intel", "volatile"
    );
  }
  result
}

pub fn debug() -> u32 {
  syscall_inner(0xffff, 0, 0)
}