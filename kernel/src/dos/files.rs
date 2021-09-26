use super::execution::{PSP, get_current_psp_segment};
use super::memory::{SegmentedAddress, get_asciiz_string};
use super::registers::{DosApiRegisters, VM86Frame};
use crate::files::handle::{FileHandle, Handle};
use crate::interrupts::stack::StackFrame;
use crate::task::io;

#[repr(C, packed)]
pub struct FileControlBlock {
  drive_number: u8,
  filename: [u8; 8],
  extension: [u8; 3],
  current_block: u16,
  record_size: u16,
  file_size: u32,
  file_date: FileDate,
  file_time: FileTime,
  reserved_attributes: [u8; 8],
  char_device_header: u32,
  reserved_share: [u8; 2],
  relative_record_number: u8,
  absolute_record_number: u32,
}

#[repr(transparent)]
pub struct FileDate(u16);

#[repr(transparent)]
pub struct FileTime(u16);

pub fn open_file(regs: &mut DosApiRegisters, segments: &mut VM86Frame, stack_frame: &StackFrame) {
  // TODO: use this?
  let _mode = regs.al();

  let filename_ptr = SegmentedAddress { segment: segments.ds as u16, offset: regs.dx as u16 };
  let path = unsafe { get_asciiz_string(filename_ptr) };
  let handle: FileHandle = match io::open_path(path) {
    Ok(handle) => handle,
    Err(_) => {
      unsafe { stack_frame.set_carry_flag() };
      regs.ax = 2;
      return;
    },
  };

  let psp = match get_current_psp_segment() {
    Some(p) => unsafe { PSP::at_segment(p) },
    None => return,
  };
  let handle_index = match psp.find_empty_file_handle() {
    Some(index) => index,
    None => return,
  };
  psp.file_handles[handle_index] = handle.as_u32() as u8;
  regs.ax = handle_index as u32;
}

pub fn read_file(regs: &mut DosApiRegisters, segments: &mut VM86Frame) {
  let psp = match get_current_psp_segment() {
    Some(p) => unsafe { PSP::at_segment(p) },
    None => return,
  };
  let handle_index = regs.bx as usize;
  let raw_handle = psp.file_handles[handle_index];
  if raw_handle == 0xff {
    return;
  }
  let handle = FileHandle::new(raw_handle as u32);
  let to_read = regs.cx as usize;
  let dest_ptr = SegmentedAddress { segment: segments.ds as u16, offset: regs.dx as u16 };
  let dest_addr = dest_ptr.as_address();
  let dest_slice = unsafe { core::slice::from_raw_parts_mut(dest_addr as *mut u8, to_read) };
  let bytes_written: usize = match io::read_file(handle, dest_slice) {
    Ok(bytes) => bytes,
    Err(_) => return,
  };

  regs.ax = bytes_written as u32;
}
