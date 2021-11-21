#[repr(u8)]
pub enum DirEntryType {
  Empty = 0,
  Directory = 1,
  File = 2,
}

#[repr(C, packed)]
pub struct DirEntryInfo {
  pub file_name: [u8; 8],
  pub file_ext: [u8; 3],
  pub entry_type: DirEntryType,
  pub byte_size: usize,
}

impl DirEntryInfo {
  pub fn empty() -> DirEntryInfo {
    DirEntryInfo {
      file_name: [0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20],
      file_ext: [0x20, 0x20, 0x20],
      entry_type: DirEntryType::Empty,
      byte_size: 0,
    }
  }

  pub fn is_empty(&self) -> bool {
    match self.entry_type {
      DirEntryType::Empty => true,
      _ => false,
    }
  }
}

#[repr(C, packed)]
pub struct FileStatus {
  pub byte_size: usize,
}

impl FileStatus {
  pub fn empty() -> Self {
    Self {
      byte_size: 0,
    }
  }
}
