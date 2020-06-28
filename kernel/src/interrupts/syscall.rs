use crate::kprintln;
use crate::syscalls::{exec, file, fs};
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
pub unsafe extern "C" fn _syscall_inner(frame: &stack::StackFrame, registers: &mut SavedRegisters) {
  let eax = registers.eax;
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
      let time = registers.ebx;
      exec::sleep(time);
    },
    0x6 => { // yield
      exec::yield_coop();
    },

    // files
    0x10 => { // open
      let path_str_ptr = &*(registers.ebx as *const syscall::StringPtr);
      let path_str = path_str_ptr.as_str();
      match file::open_path(path_str) {
        Ok(handle) => registers.eax = handle,
        Err(_) => registers.eax = 0xffffffff,
      }
    },
    0x11 => { // close
      let handle = registers.eax;
      file::close(handle);
      registers.eax = 0;
    },
    0x12 => { // read
      let handle = registers.ebx;
      let dest_addr = registers.ecx as *mut u8;
      let length = registers.edx as usize;
      match file::read(handle, dest_addr, length) {
        Ok(bytes_read) => registers.eax = bytes_read as u32,
        Err(_) => (),
      }
    },
    0x13 => { // write
      let handle = registers.ebx;
      let src_addr = registers.ecx as *const u8;
      let length = registers.edx as usize;
      match file::write(handle, src_addr, length) {
        Ok(bytes_written) => registers.eax = bytes_written as u32,
        Err(_) => (),
      }
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
    0x1a => { // readdir

    },
    0x1b => { // chdir

    },
    0x1c => { // getcwd

    },

    // filesystem
    0x20 => { // mount

    },
    0x21 => { // unmount

    },

    // misc
    0xffff => { // debug
      kprintln!("SYSCALL!");
      registers.eax = 0;
    },
    _ => {
      // unknown syscall
    },
  }
}