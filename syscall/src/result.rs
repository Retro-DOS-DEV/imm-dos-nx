/// Error codes that can be returned from syscalls
/// They do not correspond to POSIX error numbers, but they can be mapped
/// to POSIX values for compatibility.
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum SystemError {
  /// Used when the error doesn't match a known code
  Unknown = 0,
  /// File operation was performed on an unopened or bad descriptor
  BadFileDescriptor = 1,
  /// A drive does not exist with the specified name
  NoSuchDrive = 2,
  /// The selected drive uses an unregistered filesystem
  NoSuchFileSystem = 3,
  /// File or directory path does not exist
  NoSuchEntity = 4,
  /// Directory operation was performed on a non-directory path or descriptor
  NotDirectory = 5,
  /// Directory was not empty
  NotEmpty = 6,
  /// Pipe was closed at the other end
  BrokenPipe = 7,
  /// File does not support seek operation
  InvalidSeek = 8,
  /// Unsupported IOCTL command
  UnsupportedCommand = 9,
  /// An error occurred while reading / writing
  IOError = 10,
  /// The process cannot open any more file handles
  MaxFilesExceeded = 11,
}

impl SystemError {
  /// Extract the SystemError value from a numeric code
  pub fn from_code(code: u32) -> SystemError {
    match code & 0xffff {
      1 => SystemError::BadFileDescriptor,
      2 => SystemError::NoSuchDrive,
      3 => SystemError::NoSuchFileSystem,
      4 => SystemError::NoSuchEntity,
      5 => SystemError::NotDirectory,
      6 => SystemError::NotEmpty,
      7 => SystemError::BrokenPipe,
      8 => SystemError::InvalidSeek,
      9 => SystemError::UnsupportedCommand,
      10 => SystemError::IOError,
      11 => SystemError::MaxFilesExceeded,

      _ => SystemError::Unknown,
    }
  }

  /// Convert a SystemError to be sent as a number in a register
  pub fn to_code(&self) -> u32 {
    0x80000000 | (*self as u32)
  }
}

pub fn result_from_code(code: u32) -> Result<u32, SystemError> {
  if code & 0x80000000 == 0 {
    Ok(code & 0x7fffffff)
  } else {
    Err(SystemError::from_code(code))
  }
}