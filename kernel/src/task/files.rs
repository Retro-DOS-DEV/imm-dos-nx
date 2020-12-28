use crate::collections::SlotList;
use crate::files::handle::LocalHandle;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DriveID(usize);

impl DriveID {
  pub fn new(id: usize) -> DriveID {
    DriveID(id)
  }
}

/// An open file contains a reference to a drive, and the handle local to that
/// drive that can be used to access the file.
#[derive(Copy, Clone)]
pub struct OpenFile {
  pub drive: DriveID,
  pub local_handle: LocalHandle,
}

/// A file map contains slots to open files. A FileHandle represents an index
/// into this data structure.
pub type FileMap = SlotList<OpenFile>;
