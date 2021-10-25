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

#[cfg(test)]
mod tests {
  use super::get_extension;

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
}