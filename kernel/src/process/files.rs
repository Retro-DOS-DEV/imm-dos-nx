use crate::files::handle::{DriveHandlePair, FileHandle, LocalHandle};
use super::process_state::ProcessState;

impl ProcessState {
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
}