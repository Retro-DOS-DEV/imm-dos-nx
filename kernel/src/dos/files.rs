use super::memory::SegmentedAddress;

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
