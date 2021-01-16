//! A "Drive" in DOS-speak is a mounted disk, real or virtual.
//! Drives are identified externally by a unique string of one or more letters
//! followed by a colon, ie "C:"
//! IMM-DOS extends the original notion of a single-letter drive, supporting up
//! to eight alphanumeric characters.
//! Behind the scenes, each drive has a numeric ID for easier comparison and
//! lookup. When a drive is mounted, a new ID is reserved, and will not be used
//! again by the system.
//! Each mounted drive is powered by a filesystem driver that responds to a
//! standard set of IO methods.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::cmp::{Ord, PartialOrd};
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::RwLock;
use super::filesystem::{FileSystemCategory, FileSystemInstance, FileSystemType};

/// A DriveID is a unique numeric reference to a drive. Drive names shouldn't be
/// used as references within the kernel.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DriveID(usize);

impl DriveID {
  pub fn new(id: usize) -> DriveID {
    DriveID(id)
  }
}

pub struct DriveMap {
  next_id: AtomicUsize,
  drives: RwLock<BTreeMap<DriveID, FileSystemInstance>>,
}

impl DriveMap {
  pub const fn new() -> DriveMap {
    DriveMap {
      next_id: AtomicUsize::new(0),
      drives: RwLock::new(BTreeMap::new()),
    }
  }

  fn next_drive_id(&self) -> DriveID {
    let id = self.next_id.fetch_add(1, Ordering::SeqCst);
    DriveID::new(id)
  }

  pub fn mount_drive(&self, name: &str, category: FileSystemCategory, instance: Box<FileSystemType>) -> DriveID {
    let entry = FileSystemInstance {
      category,
      name: Box::from(name),
      instance: Arc::new(instance),
    };
    let id = self.next_drive_id();
    self.drives.write().insert(id, entry);

    id
  }

  pub fn get_drive_number(&self, name: &str) -> Option<DriveID> {
    let drives = self.drives.read();
    for (id, instance) in drives.iter() {
      if instance.matches_name(name) {
        return Some(*id);
      }
    }
    None
  }

  pub fn get_drive_instance(&self, id: &DriveID) -> Option<(FileSystemCategory, Arc<Box<FileSystemType>>)> {
    let drives = self.drives.read();
    let entry = drives.get(id)?;
    Some((entry.get_category(), entry.get_fs()))
  }
}