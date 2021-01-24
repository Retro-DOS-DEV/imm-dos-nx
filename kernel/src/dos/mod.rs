//! The dos module contains all of the internal structs used by DOS APIs, as
//! well as methods to manipulate the current VM process space.

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

#[repr(C, packed)]
pub struct PSP {
  int_20: [u8; 2],
  memory_top_paragraphs: u16,
  dos_reserved: u8,
  dispatcher_long: [u8; 5],
  bytes_in_segment: u16,
  termination_vector: u32,
  control_break_vector: u32,
  critical_error_vector: u32,
  parent_segment: u16,
  file_handles: [u8; 20],
  env_segment: u16,
  stack_save: u32,
  handle_array_length: u16,
  handle_array_pointer: u32,
  previous_psp: u32,
  reserved: [u8; 20],
  dispatcher: [u8; 3],
  unused: [u8; 9],
  fcb_first: [u8; 36],
  fcb_second: [u8; 20],
  command_tail: [u8; 128],
}
