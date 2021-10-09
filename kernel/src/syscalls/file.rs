use crate::files::cursor::SeekMethod;
use crate::files::handle::{FileHandle, Handle};
use syscall::files::{DirEntryInfo, DirEntryType};
use syscall::result::SystemError;

pub fn open_path(path_str: &'static str) -> Result<u32, SystemError> {
  crate::task::io::open_path(path_str).map(|handle| handle.as_u32())
}

pub fn close(handle: u32) -> Result<(), SystemError> {
  crate::task::io::close_file(FileHandle::new(handle))
}

pub unsafe fn read(handle: u32, dest: *mut u8, length: usize) -> Result<usize, SystemError> {
  let buffer = core::slice::from_raw_parts_mut(dest, length);
  crate::task::io::read_file(FileHandle::new(handle), buffer)
}

pub unsafe fn write(handle: u32, src: *const u8, length: usize) -> Result<usize, SystemError> {
  let buffer = core::slice::from_raw_parts(src, length);
  crate::task::io::write_file(FileHandle::new(handle), buffer)
}

pub fn ioctl(handle: u32, command: u32, arg: u32) -> Result<u32, SystemError> {
  //crate::task::io::ioctl(handle, command, arg)
  Err(SystemError::IOError)
}

pub fn dup(to_duplicate: u32, to_replace: u32) -> Result<u32, SystemError> {
  let from_handle = FileHandle::new(to_duplicate);
  let to_handle = if to_replace == 0xffffffff {
    None
  } else {
    Some(FileHandle::new(to_replace))
  };
  crate::task::io::dup(from_handle, to_handle).map(|h| h.as_u32())
}

pub fn pipe() -> Result<(u32, u32), SystemError> {
  /*
  let (read_local, write_local) = pipes::create_pipe().map_err(|_| SystemError::Unknown)?;
  let (read, write) = {
    let current = current_process();
    let fs_number = unsafe { filesystems::PIPE_FS };
    let read = current.open_file(fs_number, read_local).as_u32();
    let write = current.open_file(fs_number, write_local).as_u32();
    (read, write)
  };
  Ok((read, write))
  */
  Err(SystemError::Unknown)
}

pub fn seek(handle: u32, method: u32, cursor: u32) -> Result<u32, SystemError> {
  let seek_method = match method {
    1 => SeekMethod::Relative(cursor as i32 as isize),
    _ => SeekMethod::Absolute(cursor as usize),
  };
  crate::task::io::seek(FileHandle::new(handle), seek_method).map(|cur| cur as u32)
}

pub fn open_dir(path_str: &'static str) -> Result<u32, SystemError> {
  /*
  let (drive, path) = filename::string_to_drive_and_path(path_str);
  let number = filesystems::get_fs_number(drive).ok_or(SystemError::NoSuchDrive)?;
  let fs = filesystems::get_fs(number).ok_or(SystemError::NoSuchFileSystem)?;
  let local_handle = fs.open_dir(path).map_err(|_| SystemError::NoSuchEntity)?;
  current_process().open_directory(number, local_handle).map(|handle| handle.as_u32())
  */
  Err(SystemError::Unknown)
}

pub fn read_dir(handle: u32, index: usize, info: *mut DirEntryInfo) -> Result<(), SystemError> {
  /*
  let drive_and_handle = current_process()
    .get_open_dir_info(FileHandle::new(handle))
    .ok_or(SystemError::BadFileDescriptor)?;
  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(SystemError::NoSuchFileSystem)?;
  let entry = unsafe { &mut *info };
  fs.read_dir(drive_and_handle.1, index, entry).map_err(|_| SystemError::NoSuchEntity)
  */
  Err(SystemError::Unknown)
}
