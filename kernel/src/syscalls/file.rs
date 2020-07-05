use crate::files::filename;
use crate::files::handle::{FileHandle, Handle};
use crate::filesystems;
use super::current_process;

pub enum FileError {
  DriveDoesNotExist,
  UnknownFileSystem,
  FileDoesNotExist,
  HandleNotOpen,
  ReadError,
  WriteError,
}

pub fn open_path(path_str: &'static str) -> Result<u32, FileError> {
  let (drive, path) = filename::string_to_drive_and_path(path_str);
  let number = filesystems::get_fs_number(drive).ok_or(FileError::DriveDoesNotExist)?;
  let fs = filesystems::get_fs(number).ok_or(FileError::UnknownFileSystem)?;
  let local_handle = fs.open(path).map_err(|_| FileError::FileDoesNotExist)?;
  Ok(current_process().open_file(number, local_handle).as_u32())
}

pub fn close(handle: u32) {
  current_process().close_file(FileHandle::new(handle))
}

pub unsafe fn read(handle: u32, dest: *mut u8, length: usize) -> Result<usize, FileError> {
  let drive_and_handle = current_process()
    .get_open_file_info(FileHandle::new(handle))
    .ok_or(FileError::HandleNotOpen)?;

  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(FileError::UnknownFileSystem)?;
  let buffer = core::slice::from_raw_parts_mut(dest, length);
  fs.read(drive_and_handle.1, buffer).map_err(|_| FileError::ReadError)
}

pub unsafe fn write(handle: u32, src: *const u8, length: usize) -> Result<usize, FileError> {
  let drive_and_handle = current_process()
    .get_open_file_info(FileHandle::new(handle))
    .ok_or(FileError::HandleNotOpen)?;

  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(FileError::UnknownFileSystem)?;
  let buffer = core::slice::from_raw_parts(src, length);
  fs.write(drive_and_handle.1, buffer).map_err(|_| FileError::WriteError)
}
