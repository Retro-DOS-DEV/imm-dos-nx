/// Split a path into its drive and local path components
pub fn string_to_drive_and_path(raw: &str) -> (&str, &str) {
  let mut drive_split = raw.splitn(2, ':');
  let drive = match drive_split.next() {
    Some(d) => d,
    None => {
      return (&raw[0..0], &raw[0..0]);
    },
  };
  let path = drive_split.next();
  match path {
    None => {
      // There was no colon present in the path
      // Treat this situation as having no drive
      (&raw[0..0], drive)
    },
    Some(p) => {
      (drive, p)
    },
  }
}

pub fn get_extension<'a>(raw: &'a str) -> Option<&'a str> {
  let bytes = raw.as_bytes();
  let mut cur = bytes.len();
  while cur > 0 {
    cur -= 1;
    let ch = bytes[cur];
    if ch == b'/' || ch == b'\\' {
      return None;
    }
    if ch == b'.' {
      if cur == bytes.len() - 1 {
        // filename that ends in .
        return None;
      }
      let ext_range = &bytes[cur + 1..];
      return Some(unsafe { core::str::from_utf8_unchecked(ext_range) });
    } 
  }
  None
}

pub fn copy_filename_to_dos_style(filename: &[u8], dos_name: &mut [u8; 8], dos_ext: &mut [u8; 3]) {
  let mut i = 0;
  while i < 8 && i < filename.len() && filename[i] != b'.' {
    dos_name[i] = filename[i];
    i += 1;
  }
  for fill in i..8 {
    dos_name[fill] = 0x20;
  }
  while filename[i] != b'.' {
    i += 1;
  }
  i += 1;
  let ext_offset = i;
  while i - ext_offset < 3 && i < filename.len() {
    dos_ext[i - ext_offset] = filename[i];
    i += 1;
  }
  for fill in (i - ext_offset)..3 {
    dos_ext[fill] = 0x20;
    i += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::{get_extension, string_to_drive_and_path, copy_filename_to_dos_style};

  #[test]
  fn drive_and_path() {
    assert_eq!(
      string_to_drive_and_path("C:\\dir\\subdir\\file.ext"),
      ("C", "\\dir\\subdir\\file.ext"),
    );
  }

  #[test]
  fn extension() {
    assert_eq!(
      get_extension("C:\\dir\\prog.exe"),
      Some("exe"),
    );
    assert_eq!(
      get_extension("C:\\file.longextension"),
      Some("longextension"),
    );
    assert_eq!(
      get_extension("C:\\dir\\otherdir\\file"),
      None,
    );
    assert_eq!(
      get_extension("C:\\somedir\\program."),
      None,
    );
  }

  #[test]
  fn dos_filename_copy() {
    {
      let mut name: [u8; 8] = [0; 8];
      let mut ext: [u8; 3] = [0; 3];
      copy_filename_to_dos_style("myfile.com".as_bytes(), &mut name, &mut ext);
      assert_eq!(name, [b'm', b'y', b'f', b'i', b'l', b'e', 0x20, 0x20]);
      assert_eq!(ext, [b'c', b'o', b'm']);
    }
  }
}