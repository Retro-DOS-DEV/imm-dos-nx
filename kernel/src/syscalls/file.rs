use crate::files::cursor::SeekMethod;
use crate::files::filename;
use crate::files::handle::{FileHandle, Handle};
use crate::filesystems;
use crate::pipes;
use super::current_process;
use syscall::files::{DirEntryInfo, DirEntryType};
use syscall::result::SystemError;

pub fn open_path(path_str: &'static str) -> Result<u32, SystemError> {
  let (drive, path) = filename::string_to_drive_and_path(path_str);
  let number = filesystems::get_fs_number(drive).ok_or(SystemError::NoSuchDrive)?;
  let fs = filesystems::get_fs(number).ok_or(SystemError::NoSuchFileSystem)?;
  let local_handle = fs.open(path).map_err(|_| SystemError::NoSuchEntity)?;
  Ok(current_process().open_file(number, local_handle).as_u32())
}

pub fn close(handle: u32) -> Result<(), SystemError> {
  let pair_to_close = {
    let cur = current_process();
    let prev = cur.close_file(FileHandle::new(handle));
    match prev {
      Some(pair) => if !current_process().references_drive_and_handle(pair.0, pair.1) {
        Some(pair)
      } else {
        // Another handle in this process references the same file descriptor
        None
      },
      None => None,
    }
  };

  let pair = pair_to_close.ok_or(SystemError::BadFileDescriptor)?;
  match filesystems::get_fs(pair.0) {
    Some(fs) => fs.close(pair.1).map_err(|_| SystemError::IOError),
    None => Err(SystemError::NoSuchFileSystem),
  }
}

pub unsafe fn read(handle: u32, dest: *mut u8, length: usize) -> Result<usize, SystemError> {
  let drive_and_handle = current_process()
    .get_open_file_info(FileHandle::new(handle))
    .ok_or(SystemError::BadFileDescriptor)?;

  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(SystemError::NoSuchFileSystem)?;
  let buffer = core::slice::from_raw_parts_mut(dest, length);
  fs.read(drive_and_handle.1, buffer).map_err(|_| SystemError::IOError)
}

pub unsafe fn write(handle: u32, src: *const u8, length: usize) -> Result<usize, SystemError> {
  let drive_and_handle = current_process()
    .get_open_file_info(FileHandle::new(handle))
    .ok_or(SystemError::BadFileDescriptor)?;

  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(SystemError::NoSuchFileSystem)?;
  let buffer = core::slice::from_raw_parts(src, length);
  fs.write(drive_and_handle.1, buffer).map_err(|_| SystemError::IOError)
}

pub fn ioctl(handle: u32, command: u32, arg: u32) -> Result<u32, SystemError> {
  let drive_and_handle = current_process()
    .get_open_file_info(FileHandle::new(handle))
    .ok_or(SystemError::BadFileDescriptor)?;

  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(SystemError::NoSuchFileSystem)?;
  fs.ioctl(drive_and_handle.1, command, arg).map_err(|_| SystemError::IOError)
}

pub fn dup(to_duplicate: u32, to_replace: u32) -> Result<u32, SystemError> {
  let drive_and_handle = current_process()
    .get_open_file_info(FileHandle::new(to_duplicate))
    .ok_or(SystemError::BadFileDescriptor)?;

  let (handle, pair_to_close) = {
    let cur = current_process();
    let mut files = cur.get_open_files().write();
    let handle = if to_replace == 0xffffffff {
      files.get_next_available_handle().ok_or(SystemError::MaxFilesExceeded)?
    } else {
      FileHandle::new(to_replace)
    };

    let prev = files.set_handle_directly(handle, drive_and_handle.0, drive_and_handle.1);
    match prev {
      Some(pair) => if !current_process().references_drive_and_handle(pair.0, pair.1) {
        (handle, Some(pair))
      } else {
        // Another handle in this process references the same file descriptor
        (handle, None)
      },
      None => (handle, None),
    }
  };

  let pair = pair_to_close.ok_or(SystemError::BadFileDescriptor)?;
  match filesystems::get_fs(pair.0) {
    Some(fs) => {
      fs.close(pair.1).map_err(|_| SystemError::IOError)?;
      Ok(handle.as_u32())
    },
    None => Err(SystemError::NoSuchFileSystem),
  }
}

pub fn pipe() -> Result<(u32, u32), SystemError> {
  let (read_local, write_local) = pipes::create_pipe().map_err(|_| SystemError::Unknown)?;
  let (read, write) = {
    let current = current_process();
    let fs_number = unsafe { filesystems::PIPE_FS };
    let read = current.open_file(fs_number, read_local).as_u32();
    let write = current.open_file(fs_number, write_local).as_u32();
    (read, write)
  };
  Ok((read, write))
}

pub fn seek(handle: u32, method: u32, cursor: u32) -> Result<u32, SystemError> {
  let seek_method = match method {
    1 => SeekMethod::Relative(cursor as i32 as isize),
    _ => SeekMethod::Absolute(cursor as usize),
  };
  let drive_and_handle = current_process()
    .get_open_file_info(FileHandle::new(handle))
    .ok_or(SystemError::BadFileDescriptor)?;

  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(SystemError::NoSuchFileSystem)?;
  fs.seek(drive_and_handle.1, seek_method)
    .map(|new_cursor| new_cursor as u32)
    .map_err(|_| SystemError::IOError)
}

pub fn open_dir(path_str: &'static str) -> Result<u32, SystemError> {
  let (drive, path) = filename::string_to_drive_and_path(path_str);
  let number = filesystems::get_fs_number(drive).ok_or(SystemError::NoSuchDrive)?;
  let fs = filesystems::get_fs(number).ok_or(SystemError::NoSuchFileSystem)?;
  let local_handle = fs.open_dir(path).map_err(|_| SystemError::NoSuchEntity)?;
  current_process().open_directory(number, local_handle).map(|handle| handle.as_u32())
}

pub fn read_dir(handle: u32, index: usize, info: *mut DirEntryInfo) -> Result<(), SystemError> {
  let drive_and_handle = current_process()
    .get_open_dir_info(FileHandle::new(handle))
    .ok_or(SystemError::BadFileDescriptor)?;
  let fs = filesystems::get_fs(drive_and_handle.0).ok_or(SystemError::NoSuchFileSystem)?;
  let entry = unsafe { &mut *info };
  fs.read_dir(drive_and_handle.1, index, entry).map_err(|_| SystemError::NoSuchEntity)
}
