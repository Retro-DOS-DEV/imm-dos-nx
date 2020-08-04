#![feature(core_intrinsics)]
#![feature(llvm_asm)]

#![no_std]

pub mod data;
pub mod files;
pub mod flags;
pub mod result;
pub mod signals;

pub use data::*;

pub fn syscall_inner(method: u32, arg0: u32, arg1: u32, arg2: u32) -> u32 {
  let result: u32;
  unsafe {
    llvm_asm!("int 0x2b" :
          "={eax}"(result) :
          "{eax}"(method), "{ebx}"(arg0), "{ecx}"(arg1), "{edx}"(arg2) :
          "eax", "ebx", "ecx", "edx" :
          "intel", "volatile"
    );
  }
  result
}

pub fn debug() -> u32 {
  syscall_inner(0xffff, 0, 0, 0)
}

pub fn open(path: &'static str) -> u32 {
  let path_ptr = StringPtr::from_str(path);
  syscall_inner(0x10, &path_ptr as *const StringPtr as u32, 0, 0)
}

pub fn read(handle: u32, buffer: *mut u8, length: usize) -> usize {
  syscall_inner(0x12, handle, buffer as u32, length as u32) as usize
}

pub fn write(handle: u32, buffer: *const u8, length: usize) -> usize {
  syscall_inner(0x13, handle, buffer as u32, length as u32) as usize
}

pub fn dup(handle: u32) -> u32 {
  syscall_inner(0x1d, handle, 0xffffffff, 0)
}

pub fn dup2(handle: u32, replace: u32) -> u32 {
  syscall_inner(0x1d, handle, replace, 0)
}

pub fn ioctl(handle: u32, command: u32, arg: u32) -> u32 {
  syscall_inner(0x1e, handle, command, arg)
}

pub fn pipe(handles: &[u32; 2]) -> u32 {
  syscall_inner(0x1f, &handles[0] as *const u32 as u32, &handles[1] as *const u32 as u32, 0)
}

pub fn seek(handle: u32, position: u32) {
  syscall_inner(0x20, handle, 0, position);
}

pub fn seek_relative(handle: u32, offset: i32) -> u32 {
  syscall_inner(0x20, handle, 1, offset as u32)
}

pub fn fork() -> u32 {
  syscall_inner(0x01, 0, 0, 0)
}

pub fn exec(path: &'static str) {
  let path_ptr = StringPtr::from_str(path);
  syscall_inner(0x02, &path_ptr as *const StringPtr as u32, 0, 0);
}

pub fn execv(path: &'static str, args: &'static str) {
  let path_ptr = StringPtr::from_str(path);
  let arg_ptr = StringPtr::from_str(args);
  syscall_inner(0x02, &path_ptr as *const StringPtr as u32, &arg_ptr as *const StringPtr as u32, 0);
}

pub fn exec_format(path: &'static str, format: u32) {
  let path_ptr = StringPtr::from_str(path);
  syscall_inner(0x02, &path_ptr as *const StringPtr as u32, 0, format);
}

pub fn brk(addr: u32) -> u32 {
  syscall_inner(0x04, 0, addr, 0)
}

pub fn sbrk(delta: i32) -> u32 {
  syscall_inner(0x04, 1, delta as u32, 0)
}

pub fn yield_coop() {
  syscall_inner(0x06, 0, 0, 0);
}

pub fn sleep(ms: u32) {
  syscall_inner(0x05, ms, 0, 0);
}

pub fn exit(code: u32) -> ! {
  syscall_inner(0, code, 0, 0);
  unsafe { core::intrinsics::unreachable() }
}

pub fn get_pid() -> u32 {
  syscall_inner(0x03, 0, 0, 0)
}

pub fn wait_pid(id: u32) -> (u32, u32) {
  let mut status = 0;
  let pid = syscall_inner(0x09, id, &mut status as *mut u32 as u32, 0);
  (pid, status)
}

/**
 * Send a signal to a specific thread, equivalent to POSIX `kill`
 */
pub fn send_signal(pid: u32, signal: u32) {
  syscall_inner(0x8, pid, signal, 0);
}

/**
 * Send a signal to the current thread
 */
pub fn raise(signal: u32) {
  syscall_inner(0x7, signal, 0, 0);
}

