use crate::files::filename;
use crate::filesystems;
use crate::process;
use super::file::FileError;

pub fn yield_coop() {
  process::yield_coop();
}

pub fn sleep(ms: u32) {
  process::sleep(ms as usize)
}

pub fn fork() -> u32 {
  process::fork()
}

pub fn exec_path(path_str: &'static str) -> Result<(), FileError> {
  let (drive, path) = filename::string_to_drive_and_path(path_str);
  let number = filesystems::get_fs_number(drive).ok_or(FileError::DriveDoesNotExist)?;
  let fs = filesystems::get_fs(number).ok_or(FileError::UnknownFileSystem)?;
  let local_handle = fs.open(path).map_err(|_| FileError::FileDoesNotExist)?;
  process::exec(number, local_handle, process::exec::InterpretationMode::Detect);
  Ok(())
}

pub fn exit(code: u32) {
  process::exit(code);
}