use crate::files::handle::{FileHandle, Handle};
use crate::task::io::{read_file, write_file};
use crate::task::switching::get_current_process;
use crate::task::vm::Subsystem;
use super::{execution::PSP, memory::SegmentedAddress, registers::{DosApiRegisters, VM86Frame}};

fn get_current_psp_segment() -> Option<u16> {
  let process_lock = get_current_process();
  let process = process_lock.read();
  match &process.subsystem {
    Subsystem::DOS(state) => Some(state.current_psp),
    Subsystem::Native => {
      return None;
    },
  }
}

pub fn read_stdin_with_echo(regs: &mut DosApiRegisters) {
  // Read from STDIN (local handle 0), write to STDOUT (local handle 1)
  let mut buffer: [u8; 1] = [0];
  let psp = match get_current_psp_segment() {
    Some(p) => unsafe { PSP::at_segment(p) },
    None => return,
  };
  let stdin_handle = FileHandle::new(psp.file_handles[0] as u32);
  let stdout_handle = FileHandle::new(psp.file_handles[1] as u32);

  let len = match read_file(stdin_handle, &mut buffer) {
    Ok(len) => len,
    Err(_) => return,
  };

  if len > 0 {
    regs.set_al(buffer[0]);
    let _ = write_file(stdout_handle, &buffer);
  }
}

pub fn read_stdaux() {

}

pub fn output_char_to_stdout(regs: &mut DosApiRegisters) {
  let psp = match get_current_psp_segment() {
    Some(p) => unsafe { PSP::at_segment(p) },
    None => return,
  };
  let stdout_handle = FileHandle::new(psp.file_handles[1] as u32);
  let buffer: [u8; 1] = [regs.dl()];
  let _ = write_file(stdout_handle, &buffer);
}

pub fn output_char_to_stdaux() {

}

pub fn console_io() {

}

pub fn print_string(regs: &mut DosApiRegisters, segments: &mut VM86Frame) {
  let psp = match get_current_psp_segment() {
    Some(p) => unsafe { PSP::at_segment(p) },
    None => return,
  };
  let stdout_handle = FileHandle::new(psp.file_handles[1] as u32);
  let string_location = SegmentedAddress {
    segment: segments.ds as u16,
    offset: regs.dx as u16,
  };
  let start = string_location.as_address() as *const u8;
  let mut length = 0;
  loop {
    if length > 255 {
      break;
    }
    let ch = unsafe { *start.offset(length) };
    if ch == b'$' {
      break;
    }
    length += 1;
  }
  if length > 0 {
    let buffer = unsafe { core::slice::from_raw_parts(start, length as usize) };
    let _ = write_file(stdout_handle, buffer);
  }
}

pub fn buffer_keyboard_input() {

}

pub fn check_stdin() {

}

pub fn get_date() {

}

pub fn set_date() {

}

pub fn get_time() {

}

pub fn set_time() {

}