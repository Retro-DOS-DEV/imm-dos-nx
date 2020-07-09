#[repr(C, packed)]
pub struct FileStat {
  pub attribute: FileAttributes,
  pub create_time: FileTimestamp,
  pub modify_time: FileTimestamp,
  pub byte_size: u32,
}

pub struct FileAttributes(u32);

pub struct FileTimestamp(u32);

impl FileTimestamp {
  pub fn new(time: u32) -> FileTimestamp {
    FileTimestamp(time)
  }

  pub fn as_u32(&self) -> u32 {
    self.0
  }
}

pub struct FileLocation(u32);

impl FileLocation {
  pub fn new(loc: u32) -> FileLocation {
    FileLocation(loc)
  }

  pub fn offset(&self, delta: i32) -> FileLocation {
    let udelta = delta as u32;
    let mut sum = self.0.wrapping_add(udelta);
    if (self.0 ^ udelta) & 0x80000000 != 0 {
      if (sum ^ udelta) & 0x80000000 == 0 {
        if sum & 0x80000000 == 0 {
          sum = 0xffffffff;
        } else {
          sum = 0;
        }
      }
    }
    FileLocation(sum)
  }

  pub fn as_u32(&self) -> u32 {
    self.0
  }

  pub fn as_usize(&self) -> usize {
    self.0 as usize
  }
}

#[cfg(test)]
mod tests {
  use super::{FileLocation};

  #[test]
  fn file_location_offset() {
    let simple_sum = FileLocation::new(5).offset(20);
    assert_eq!(simple_sum.as_usize(), 25);

    let negative_delta = FileLocation::new(64).offset(-10);
    assert_eq!(negative_delta.as_usize(), 54);

    let overflow = FileLocation::new(0xffffff00).offset(0x200);
    assert_eq!(overflow.as_usize(), 0xffffffff);

    let underflow = FileLocation::new(12).offset(-15);
    assert_eq!(underflow.as_usize(), 0);

    let zero = FileLocation::new(104).offset(-104);
    assert_eq!(zero.as_usize(), 0);
  }
}