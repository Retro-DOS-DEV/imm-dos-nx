pub type DriveName = [u8; 8];
pub type FileName = [u8; 8];
pub type FileExtension = [u8; 3];

pub struct FullFileName {
  name: FileName,
  extension: FileExtension,
}

pub struct Path<'orig> {
  original: &'orig str,
  pub path_start: usize,
  pub drive: DriveName,
  pub filename: FileName,
  pub extension: FileExtension,
}

pub enum PathError {
  InvalidDrive,
  InvalidPath,
  InvalidExtension,
}

enum ParseState {
  Drive,
  PathStart,
  DirectoryName,
  FileName,
}

pub fn is_valid_filename_character(ch: u8) -> bool {
  match ch {
    0x30..=0x39 => true, // number
    0x41..=0x5a => true, // uppercase letter
    0x61..=0x7a => true, // lowercase letter

    0x20 => true, // space
    0x21 => true, // !
    0x23..=0x29 => true, // #, $, %, &, ', (, )
    0x2d => true, // -
    0x40 => true, // @
    0x5e..=0x60 => true, // ^, _, `
    0x7b => true, // {
    0x7d => true, // }
    0x7e => true, // ~
    _ => false
  }
}

impl<'orig> Path<'orig> {
  pub fn from_str(s: &'orig str) -> Result<Path<'orig>, PathError> {
    let mut path = Path {
      original: s,
      path_start: 0,
      drive: [0x20; 8],
      filename: [0x20; 8],
      extension: [0x20; 3],
    };
    let bytes = s.as_bytes();
    let mut index = 0;
    let mut fill = 0;
    let mut state = ParseState::Drive;
    while index < bytes.len() {
      let cur = bytes[index];
      match state {
        ParseState::Drive => {
          if is_valid_filename_character(cur) {
            if fill > 7 {
              return Err(PathError::InvalidDrive);
            }
            path.drive[fill] = cur;
            fill += 1;
          } else if cur == b':' {
            fill = 0;
            state = ParseState::PathStart;
          } else {
            return Err(PathError::InvalidDrive);
          }
        },
        ParseState::PathStart => {
          if cur == b'\\' || cur == b'/' {
            path.path_start = index;
            state = ParseState::DirectoryName;
          } else {
            return Err(PathError::InvalidPath);
          }
        },
        ParseState::DirectoryName => {
          if is_valid_filename_character(cur) {
            if fill > 7 {
              return Err(PathError::InvalidPath);
            }
            path.filename[fill] = cur;
            fill += 1;
          } else if cur == b'\\' || cur == b'/' {
            for i in 0..8 {
              path.filename[i] = 0x20;
            }
            fill = 0;
          } else if cur == b'.' {
            fill = 0;
            state = ParseState::FileName;
          } else {
            return Err(PathError::InvalidPath);
          }
        },
        ParseState::FileName => {
          if is_valid_filename_character(cur) {
            if fill > 2 {
              return Err(PathError::InvalidExtension);
            }
            path.extension[fill] = cur;
            fill += 1;
          } else {
            return Err(PathError::InvalidExtension);
          }
        },
      }
      index += 1;
    }
    Ok(path)
  }
}

// New filename handling

/**
 * Split a path into its drive and local path components
 */
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
