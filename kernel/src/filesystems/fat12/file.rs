#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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

/// Check if a filename character matches a character in a search string
pub fn name_character_matches(a: u8, b: u8) -> bool {
  if a == b {
    return true;
  }
  if b == b'?' {
    return true;
  }
  if a > 64 && a < 91 {
    if a + 32 == b {
      return true;
    }
  }
  if b > 64 && b < 91 {
    if b + 32 == a {
      return true;
    }
  }
  false
}

pub fn file_name_components_from_string(s: &str) -> ([u8; 8], [u8; 3]) {
  let mut name: [u8; 8] = [0x20; 8];
  let mut ext: [u8; 3] = [0x20; 3];
  let mut index = 0;
  let mut on_extension = false;
  for ch in s.as_bytes().iter() {
    match ch {
      b'.' => {
        if on_extension {
          return (name, ext);
        }
        index = 0;
        on_extension = true;
      },
      b'/' => {
        return (name, ext);
      },
      _ => {
        if on_extension {
          if index >= 3 {
            return (name, ext);
          }
          ext[index] = *ch;
        } else {
          if index >= 8 {
            return (name, ext);
          }
          name[index] = *ch;
        }

        index += 1;
      }
    }
  }

  return (name, ext);
}

#[cfg(test)]
mod tests {
  use super::file_name_components_from_string;

  #[test]
  fn file_name_from_string() {
    assert_eq!(
      file_name_components_from_string("hello.txt"),
      ([b'h', b'e', b'l', b'l', b'o', b' ', b' ', b' '], [b't', b'x', b't'])
    );

    assert_eq!(
      file_name_components_from_string("longfile.bmp"),
      ([b'l', b'o', b'n', b'g', b'f', b'i', b'l', b'e'], [b'b', b'm', b'p'])
    );

    assert_eq!(
      file_name_components_from_string("a.z"),
      ([b'a', b' ', b' ', b' ', b' ', b' ', b' ', b' '], [b'z', b' ', b' '])
    );

    assert_eq!(
      file_name_components_from_string("toolongtoparse.abc"),
      ([b't', b'o', b'o', b'l', b'o', b'n', b'g', b't'], [b' ', b' ', b' '])
    );

    assert_eq!(
      file_name_components_from_string("longext.abc123"),
      ([b'l', b'o', b'n', b'g', b'e', b'x', b't', b' '], [b'a', b'b', b'c'])
    );
  }
}
