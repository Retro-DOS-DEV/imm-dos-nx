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
    0x0 => { // exit
      let code = registers.ebx;
      exec::exit(code);
    },
    0x1 => { // fork
      let pid = exec::fork();
      registers.eax = pid;
    },
    0x2 => { // exec
      let path_str_ptr = &*(registers.ebx as *const syscall::StringPtr);
      let path_str = path_str_ptr.as_str();
      let arg_str = if registers.ecx == 0 {
        ""
      } else {
        let arg_str_ptr = &*(registers.ecx as *const syscall::StringPtr);
        arg_str_ptr.as_str()
      };
      match exec::exec_path(path_str, arg_str, registers.edx) {
        Ok(_) => (),
        Err(_) => registers.eax = 0xffffffff,
      }
    },
    0x3 => { // get_pid
      let pid = exec::get_pid();
      registers.eax = pid;
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
    0x7 => { // raise

    },
    0x8 => { // send_signal
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
    0x1d => { // dup
      let to_duplicate = registers.ebx;
      let to_replace = registers.ecx;
      let new_handle = match file::dup(to_duplicate, to_replace) {
        Ok(handle) => handle,
        Err(_) => 0xffffffff,
      };
      registers.eax = new_handle;
    },

    // filesystem
    0x30 => { // mount

    },
    0x31 => { // unmount

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