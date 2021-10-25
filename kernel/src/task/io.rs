use crate::files::cursor::SeekMethod;
use crate::files::filename;
use crate::files::handle::{FileHandle, LocalHandle};
use crate::fs::{DRIVES, drive::DriveID, filesystem::KernelFileSystem};
use crate::task::get_current_process;
use syscall::result::SystemError;
use super::id::ProcessID;
use super::files::{FileMap, OpenFile};

pub fn open_path(path_str: &'static str) -> Result<FileHandle, SystemError> {
  let (drive, path) = filename::string_to_drive_and_path(path_str);
  let drive_id = DRIVES.get_drive_number(drive).ok_or(SystemError::NoSuchDrive)?;
  let (_, instance) = DRIVES.get_drive_instance(&drive_id).ok_or(SystemError::NoSuchFileSystem)?;
  let local_handle = instance.open(path).map_err(|_| SystemError::NoSuchEntity)?;
  let process_handle = get_current_process().write().open_file(drive_id, local_handle);
  Ok(process_handle)
}

pub fn read_file(handle: FileHandle, buffer: &mut [u8]) -> Result<usize, SystemError> {
  let open_file_info = {
    let process_lock = get_current_process();
    let process = process_lock.read();
    let info = process
      .get_open_file_info(handle)
      .ok_or(SystemError::BadFileDescriptor)?;
    *info
  };

  let (_, instance) = DRIVES.get_drive_instance(&open_file_info.drive).ok_or(SystemError::NoSuchFileSystem)?;
  instance.read(open_file_info.local_handle, buffer).map_err(|_| SystemError::IOError)
}

pub fn write_file(handle: FileHandle, buffer: &[u8]) -> Result<usize, SystemError> {
  let open_file_info = {
    let process_lock = get_current_process();
    let process = process_lock.read();
    let info = process
      .get_open_file_info(handle)
      .ok_or(SystemError::BadFileDescriptor)?;
    *info
  };

  let (_, instance) = DRIVES.get_drive_instance(&open_file_info.drive).ok_or(SystemError::NoSuchFileSystem)?;
  instance.write(open_file_info.local_handle, buffer).map_err(|_| SystemError::IOError)
}

pub fn close_file(handle: FileHandle) -> Result<(), SystemError> {
  let open_file_info = {
    let process_lock = get_current_process();
    let process = process_lock.read();
    let info = process
      .get_open_file_info(handle)
      .ok_or(SystemError::BadFileDescriptor)?;
    *info
  };
  
  let (_, instance) = DRIVES.get_drive_instance(&open_file_info.drive).ok_or(SystemError::NoSuchFileSystem)?;
  instance.close(open_file_info.local_handle).map_err(|_| SystemError::IOError)
}

pub fn dup(from_handle: FileHandle, to_handle: Option<FileHandle>) -> Result<FileHandle, SystemError> {
  let process_lock = get_current_process();
  let mut process = process_lock.write();
  let (_, new_handle) = process.duplicate_file_descriptor(from_handle, to_handle);
  new_handle.ok_or(SystemError::BadFileDescriptor)
}

pub fn seek(handle: FileHandle, cursor: SeekMethod) -> Result<usize, SystemError> {
  let open_file_info = {
    let process_lock = get_current_process();
    let process = process_lock.read();
    let info = process
      .get_open_file_info(handle)
      .ok_or(SystemError::BadFileDescriptor)?;
    *info
  };

  let (_, instance) = DRIVES.get_drive_instance(&open_file_info.drive).ok_or(SystemError::NoSuchFileSystem)?;
  instance.seek(open_file_info.local_handle, cursor).map_err(|_| SystemError::IOError)
}

pub fn reopen_files(id: ProcessID, files: &mut FileMap) {
  files.map_in_place(|open_file| {
    match DRIVES.get_drive_instance(&open_file.drive) {
      Some((_, instance)) => match instance.reopen(open_file.local_handle, id) {
        Ok(local_handle) => {
          Some(
            OpenFile {
              drive: open_file.drive,
              local_handle,
            }
          )
        },
        Err(_) => None,
      },
      None => None,
    }
  });
}

pub fn reopen_executable(id: ProcessID, exec: Option<(DriveID, LocalHandle)>) -> Option<(DriveID, LocalHandle)> {
  let (drive_id, local_handle) = exec?;
  let (_, instance) = DRIVES.get_drive_instance(&drive_id)?;
  instance.reopen(local_handle, id).ok().map(|handle| (drive_id, handle))
}
