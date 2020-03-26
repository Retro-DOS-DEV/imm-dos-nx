use crate::files::filename;
use crate::files::handle::{FileHandle, FileHandleMap, Handle, LocalHandle};
use crate::filesystems;
use spin::RwLock;

// temporary until we implement processes
static HANDLES: RwLock<FileHandleMap> = RwLock::new(FileHandleMap::new());

fn open_file_handle(drive: usize, local: LocalHandle) -> FileHandle {
  let mut handles = HANDLES.write();
  let handle = handles.open_handle(drive, local);
  handle
}

fn close_file_handle(handle: FileHandle) {

}

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
  Ok(open_file_handle(number, local_handle).as_u32())
}

pub fn close(handle: u32) {
  close_file_handle(FileHandle::new(handle))
}

pub unsafe fn read(handle: u32, dest: *mut u8, length: usize) -> Result<usize, FileError> {
  let drive_and_handle = {
    let handles = HANDLES.read();
    handles.get_drive_and_handle(FileHandle::new(handle))
  }.ok_or(FileError::HandleNotOpen)?;

  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(FileError::UnknownFileSystem)?;
  let buffer = core::slice::from_raw_parts_mut(dest, length);
  fs.read(drive_and_handle.1, buffer).map_err(|_| FileError::ReadError)
}

pub unsafe fn write(handle: u32, src: *const u8, length: usize) -> Result<usize, FileError> {
  let drive_and_handle = {
    let handles = HANDLES.read();
    handles.get_drive_and_handle(FileHandle::new(handle))
  }.ok_or(FileError::HandleNotOpen)?;

  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(FileError::UnknownFileSystem)?;
  let buffer = core::slice::from_raw_parts(src, length);
  fs.write(drive_and_handle.1, buffer).map_err(|_| FileError::WriteError)
}
