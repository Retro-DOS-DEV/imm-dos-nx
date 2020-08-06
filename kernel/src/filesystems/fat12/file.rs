#[repr(transparent)]
pub struct FileTime(u16);

impl FileTime {
  pub fn get_hours(&self) -> u16 {
    self.0 >> 11
  }

  pub fn get_minutes(&self) -> u16 {
    (self.0 >> 5) & 0x3f
  }

  pub fn get_seconds(&self) -> u16 {
    (self.0 & 0x1f) << 1
  }
}

#[repr(transparent)]
pub struct FileDate(u16);

impl FileDate {
  pub fn get_year(&self) -> usize {
    ((self.0 >> 9) & 0x7f) as usize + 1980
  }

  pub fn get_month(&self) -> u16 {
    (self.0 >> 5) & 0xf
  }

  pub fn get_day(&self) -> u16 {
    self.0 & 0x1f
  }
}

/// Directory entries can represent a number of real or virtual items
pub enum FileType {
  File,
  Directory,
  VolumeLabel,
}

impl FileType {
  pub fn is_file(&self) -> bool {
    match self {
      FileType::File => true,
      _ => false,
    }
  }

  pub fn is_directory(&self) -> bool {
    match self {
      FileType::Directory => true,
      _ => false,
    }
  }
}
