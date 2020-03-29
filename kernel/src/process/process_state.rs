use core::cmp;
use crate::files::handle::{DeviceHandlePair, FileHandle, FileHandleMap, LocalHandle};
use crate::memory::address::VirtualAddress;
use spin::RwLock;

#[derive(Copy, Clone, Eq)]
#[repr(transparent)]
pub struct ProcessID(u32);

impl ProcessID {
  pub fn new(id: u32) -> ProcessID {
    ProcessID(id)
  }
}

impl cmp::Ord for ProcessID {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for ProcessID {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for ProcessID {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

pub struct ProcessState {
  pid: ProcessID,
  kernel_stack: VirtualAddress,

  open_files: RwLock<FileHandleMap>,
}

impl ProcessState {
  pub fn new(pid: ProcessID) -> ProcessState {
    ProcessState {
      pid,
      kernel_stack: VirtualAddress::new(0),
      open_files: RwLock::new(FileHandleMap::new()),
    }
  }

  pub fn open_file(&self, drive: usize, local: LocalHandle) -> FileHandle {
    let mut files = self.open_files.write();
    files.open_handle(drive, local)
  }

  pub fn close_file(&self, handle: FileHandle) {
    let mut files = self.open_files.write();
    files.close_handle(handle)
  }

  pub fn get_open_file_info(&self, handle: FileHandle) -> Option<DeviceHandlePair> {
    let files = self.open_files.read();
    files.get_drive_and_handle(handle)
  }
}