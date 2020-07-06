use alloc::vec::Vec;
use core::cmp;
use core::sync::atomic::{AtomicU32, Ordering};

pub trait Handle {
  fn new(handle: u32) -> Self;
  fn as_u32(&self) -> u32;
  fn as_usize(&self) -> usize;
}

#[derive(Copy, Clone, Debug)]
pub struct LocalHandle(u32);

impl Handle for LocalHandle {
  fn new(handle: u32) -> LocalHandle {
    LocalHandle(handle)
  }

  fn as_u32(&self) -> u32 {
    self.0
  }

  fn as_usize(&self) -> usize {
    self.0 as usize
  }
}

impl cmp::Ord for LocalHandle {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for LocalHandle {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for LocalHandle {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Eq for LocalHandle {}

#[derive(Copy, Clone)]
pub struct FileHandle(u32);

impl Handle for FileHandle {
  fn new(handle: u32) -> FileHandle {
    FileHandle(handle)
  }

  fn as_u32(&self) -> u32 {
    self.0
  }

  fn as_usize(&self) -> usize {
    self.0 as usize
  }
}

pub struct HandleAllocator<T: Handle> {
  next_id: AtomicU32,
  _phantom: core::marker::PhantomData<T>,
}

impl<T: Handle> HandleAllocator<T> {
  pub const fn new() -> HandleAllocator<T> {
    HandleAllocator {
      next_id: AtomicU32::new(1),
      _phantom: core::marker::PhantomData,
    }
  }

  pub fn get_next(&self) -> T {
    let handle = self.next_id.fetch_add(1, Ordering::SeqCst);
    T::new(handle)
  }
}

#[derive(Copy, Clone, Debug)]
pub struct DriveHandlePair(pub usize, pub LocalHandle);

impl PartialEq for DriveHandlePair {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0 && self.1 == other.1
  }
}

impl Eq for DriveHandlePair {}

const MAX_OPEN_FILES: usize = 4096;

/**
 * Map a process's file handles to the filesystem and fs-specific handle behind
 * each one.
 */
pub struct FileHandleMap {
  map: Vec<Option<DriveHandlePair>>,
}

impl FileHandleMap {
  pub const fn new() -> FileHandleMap {
    FileHandleMap {
      map: Vec::new(),
    }
  }

  pub fn open_handle(&mut self, drive: usize, local: LocalHandle) -> Option<FileHandle> {
    let handle = self.get_next_available_handle()?;
    self.set_handle_directly(handle, drive, local);
    Some(handle)
  }

  pub fn set_handle_directly(&mut self, handle: FileHandle, drive: usize, local: LocalHandle) -> Option<DriveHandlePair> {
    let pair = DriveHandlePair(drive, local);
    while self.map.len() <= handle.as_usize() {
      self.map.push(None);
    }
    let prev = self.map[handle.as_usize()];
    self.map[handle.as_usize()] = Some(pair);
    prev
  }

  pub fn close_handle(&mut self, handle: FileHandle) -> Option<DriveHandlePair> {
    let entry = self.map.get_mut(handle.as_usize());
    match entry {
      Some(e) => {
        let prev = *e;
        *e = None;
        return prev;
      },
      None => (),
    }
    None
  }

  pub fn get_next_available_handle(&mut self) -> Option<FileHandle> {
    for (index, item) in self.map.iter().enumerate() {
      match item {
        None => return Some(FileHandle::new(index as u32)),
        _ => (),
      }
    }
    if self.map.len() < MAX_OPEN_FILES {
      let index = self.map.len();
      self.map.push(None);
      return Some(FileHandle::new(index as u32));
    }
    None
  }

  pub fn references_drive_and_handle(&self, drive: usize, local: LocalHandle) -> bool {
    let seek = DriveHandlePair(drive, local);

    for item in self.map.iter() {
      match item {
        Some(pair) => if pair == &seek {
          return true;
        },
        None => (),
      }
    }
    false
  }

  pub fn get_drive_and_handle(&self, handle: FileHandle) -> Option<DriveHandlePair> {
    let index = handle.as_usize();
    match self.map.get(index) {
      Some(pair) => *pair,
      None => None,
    }
  }
}

impl core::fmt::Debug for FileHandleMap {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_list().entries(self.map.iter()).finish()
  }
}
