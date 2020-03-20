use crate::kprintln;
use super::stack;

#[repr(C, packed)]
pub struct SavedRegisters {
  edi: u32,
  esi: u32,
  ebp: u32,
  ebx: u32,
  edx: u32,
  ecx: u32,
  eax: u32,
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _syscall_inner(frame: &stack::StackFrame, registers: &mut SavedRegisters) {
  let eax = registers.eax;
  kprintln!("{:x} {:x} {:x} {:x} {:x}", registers.eax, registers.ebx, registers.ecx, registers.edx, registers.edi);
  kprintln!("{:x} {:x} {:x}", frame.eip, frame.cs, frame.eflags);
  match eax {
    // execution
    0x0 => { // terminate

    },
    0x1 => { // fork

    },
    0x2 => { // exec

    },
    0x3 => { // getpid

    },
    0x4 => { // brk

    },
    0x5 => { // sleep

    },
    0x6 => { // yield

    },

    // files
    0x10 => { // open

    },
    0x11 => { // close

    },
    0x12 => { // read

    },
    0x13 => { // write

    },
    0x14 => { // unlink

    },
    0x15 => { // seek

    },
    0x16 => { // stat

    },
    0x17 => { // fstat

    },
    0x18 => { // mkdir

    },
    0x19 => { // rmdir

    },
    0x1a => { // chdir

    },

    // filesystem
    0x20 => { // mount

    },
    0x21 => { // unmount

    },

    0xffff => { // debug
      kprintln!("SYSCALL!");
      registers.eax = 0;
    },
    _ => {
      // unknown syscall
    },
  }
}