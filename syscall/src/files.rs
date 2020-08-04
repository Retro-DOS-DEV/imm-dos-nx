#[repr(u8)]
pub enum DirEntryType {
  File = 0,
  Directory = 1,
}

pub struct DirEntryInfo {
  pub file_name: [u8; 8],
  pub file_ext: [u8; 3],
  pub entry_type: DirEntryType,
  pub byte_size: usize,
}