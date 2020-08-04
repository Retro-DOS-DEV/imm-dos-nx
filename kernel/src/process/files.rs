use crate::files::handle::{DriveHandlePair, FileHandle, FileHandleMap, LocalHandle};
use super::process_state::ProcessState;
use syscall::result::SystemError;

impl ProcessState {
  // Files:

  pub fn open_file(&self, drive: usize, local: LocalHandle) -> FileHandle {
    let mut files = self.get_open_files().write();
    match files.open_handle(drive, local) {
      Some(handle) => handle,
      None => panic!("Max open files exceeded"),
    }
  }

  pub fn close_file(&self, handle: FileHandle) -> Option<DriveHandlePair> {
    let mut files = self.get_open_files().write();
    files.close_handle(handle)
  }

  pub fn get_open_file_info(&self, handle: FileHandle) -> Option<DriveHandlePair> {
    let files = self.get_open_files().read();
    files.get_drive_and_handle(handle)
  }

  pub fn references_drive_and_handle(&self, drive: usize, local: LocalHandle) -> bool {
    let files = self.get_open_files().read();
    files.references_drive_and_handle(drive, local)
  }

  pub fn fork_file_map(&self) -> FileHandleMap {
    let mut forked = FileHandleMap::new();
    for (handle, pair) in self.get_open_files().read().iter() {
      forked.set_handle_directly(handle, pair.0, pair.1);
    }
    forked
  }

  // Directories:

  pub fn open_directory(&self, drive: usize, local: LocalHandle) -> Result<FileHandle, SystemError> {
    let mut dirs = self.get_open_directories().write();
    match dirs.open_handle(drive, local) {
      Some(handle) => Ok(handle),
      None => Err(SystemError::MaxFilesExceeded),
    }
  }

  pub fn close_directory(&self, handle: FileHandle) -> Result<DriveHandlePair, SystemError> {
    let mut dirs = self.get_open_directories().write();
    dirs.close_handle(handle).ok_or(SystemError::BadFileDescriptor)
  }

  pub fn get_open_dir_info(&self, handle: FileHandle) -> Option<DriveHandlePair> {
    let dirs = self.get_open_directories().read();
    dirs.get_drive_and_handle(handle)
  }

  pub fn fork_directory_map(&self) -> FileHandleMap {
    let mut forked = FileHandleMap::new();
    for (handle, pair) in self.get_open_directories().read().iter() {
      forked.set_handle_directly(handle, pair.0, pair.1);
    }
    forked
  }
}