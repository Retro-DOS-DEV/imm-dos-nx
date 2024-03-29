use crate::kprintln;
use crate::syscalls::{exec, file, fs, hardware};
use super::stack;
use syscall::result::SystemError;

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

impl core::fmt::Debug for SavedRegisters {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let eax = self.eax;
    let ebx = self.ebx;
    let ecx = self.ecx;
    let edx = self.edx;
    let ebp = self.ebp;
    let esi = self.esi;
    let edi = self.edi;
    f.debug_struct("Saved Registers")
      .field("eax", &format_args!("{:#010x}", eax))
      .field("ebx", &format_args!("{:#010x}", ebx))
      .field("ecx", &format_args!("{:#010x}", ecx))
      .field("edx", &format_args!("{:#010x}", edx))
      .field("ebp", &format_args!("{:#010x}", ebp))
      .field("esi", &format_args!("{:#010x}", esi))
      .field("edi", &format_args!("{:#010x}", edi))
      .finish()
  }
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _syscall_inner(_frame: &stack::StackFrame, registers: &mut SavedRegisters) {
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
      let result = match exec::exec_path(path_str, arg_str, registers.edx) {
        Ok(_) => 0,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x3 => { // get_pid
      let pid = exec::get_pid();
      registers.eax = pid;
    },
    0x4 => { // brk / sbrk
      let method = registers.ebx;
      let offset = registers.ecx;
      let result = match exec::brk(method, offset) {
        Ok(new_cursor) => new_cursor,
        Err(_) => SystemError::Unknown.to_code(),
      };
      registers.eax = result;
    },
    0x5 => { // sleep
      let time = registers.ebx;
      exec::sleep(time);
    },
    0x6 => { // yield
      exec::yield_coop();
    },
    0x7 => {
      
    },
    0x8 => {
      
    },
    0x09 => { // wait_pid
      let wait_id = registers.ebx;
      let (pid, code) = exec::wait_pid(wait_id);
      let status_ptr = registers.ecx as *mut u32;
      *status_ptr = code;
      registers.eax = pid;
    },

    // files
    0x10 => { // open
      let path_str_ptr = &*(registers.ebx as *const syscall::StringPtr);
      let path_str = path_str_ptr.as_str();
      let result = match file::open_path(path_str) {
        Ok(handle) => handle,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x11 => { // close
      let handle = registers.eax;
      let result = match file::close(handle) {
        Ok(_) => 0,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x12 => { // read
      let handle = registers.ebx;
      let dest_addr = registers.ecx as *mut u8;
      let length = registers.edx as usize;
      let result = match file::read(handle, dest_addr, length) {
        Ok(bytes_read) => bytes_read as u32,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x13 => { // write
      let handle = registers.ebx;
      let src_addr = registers.ecx as *const u8;
      let length = registers.edx as usize;
      let result = match file::write(handle, src_addr, length) {
        Ok(bytes_written) => bytes_written as u32,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
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
    0x1a => { // opendir
      let path_str_ptr = &*(registers.ebx as *const syscall::StringPtr);
      let path_str = path_str_ptr.as_str();
      let result = match file::open_dir(path_str) {
        Ok(handle) => handle,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x1b => { // readdir
      let handle = registers.ebx;
      let info_ptr = registers.ecx as *mut syscall::files::DirEntryInfo;
      let result = match file::read_dir(handle, info_ptr) {
        Ok(has_more) => has_more,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x1c => { // closedir

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
    0x1e => { // ioctl
      let handle = registers.ebx;
      let command = registers.ecx;
      let arg = registers.edx;
      let result = match file::ioctl(handle, command, arg) {
        Ok(value) => value,
        Err(_) => SystemError::UnsupportedCommand.to_code(),
      };
      registers.eax = result;
    },
    0x1f => { // pipe
      let read_handle_ptr = registers.ebx as *mut usize;
      let write_handle_ptr = registers.ecx as *mut usize;
      let result = match file::pipe() {
        Ok((read, write)) => {
          *read_handle_ptr = read as usize;
          *write_handle_ptr = write as usize;
          0
        },
        Err(_) => 0xffffffff,
      };
      registers.eax = result;
    },
    0x20 => { // seek
      let handle = registers.ebx;
      let method = registers.ecx;
      let cursor = registers.edx;
      let result = match file::seek(handle, method, cursor) {
        Ok(new_cursor) => new_cursor,
        Err(_) => SystemError::BadFileDescriptor.to_code(),
      };
      registers.eax = result;
    },
    0x21 => { // set current drive
      let name_str_ptr = &*(registers.ebx as *const syscall::StringPtr);
      let name_str = name_str_ptr.as_str();
      let result = match fs::set_current_drive(name_str) {
        Ok(number) => number,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x22 => { // get current drive name
      let buffer_ptr = registers.ebx as *mut u8;
      let buffer = core::slice::from_raw_parts_mut(buffer_ptr, 8);
      let result = match fs::get_current_drive_name(buffer) {
        Ok(len) => len,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x23 => { // get current drive number
      let result = match fs::get_current_drive_number() {
        Ok(number) => number,
        Err(e) => e.to_code(),
      };
      registers.eax = result;
    },
    0x24 => { // chdir for drive number
    },
    0x25 => { // get cwd for drive number
    },

    // filesystem
    0x30 => { // register

    },
    0x31 => { // unregister

    },
    0x32 => { // mount
    },
    0x33 => { // unmount
    },

    0x40 => { // install interrupt handler
      let irq = registers.ebx;
      let address = registers.ecx;
      let stack_top = registers.edx;
      let result = match exec::install_interrupt_handler(irq, address, stack_top) {
        Ok(_) => 0,
        Err(_) => 0xff,
      };
      registers.eax = result;
    },

    0x50 => { // change video mode
      let mode = registers.ebx;
      hardware::change_video_mode(mode as u8);
    },

    // misc
    0xffff => { // debug
      kprintln!("SYSCALL!");
      registers.eax = 0;
    },
    _ => {
      // unknown syscall
      registers.eax = SystemError::Unknown.to_code();
    },
  }
}