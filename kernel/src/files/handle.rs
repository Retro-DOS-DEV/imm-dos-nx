use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

pub trait Handle {
  fn new(handle: u32) -> Self;
  fn as_u32(&self) -> u32;
  fn as_usize(&self) -> usize;
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
pub struct DeviceHandlePair(pub usize, pub LocalHandle);

/**
 * Map a process's file handles to the filesystem and fs-specific handle behind
 * each one.
 */
pub struct FileHandleMap {
  map: Vec<Option<DeviceHandlePair>>,
}

impl FileHandleMap {
  pub const fn new() -> FileHandleMap {
    FileHandleMap {
      map: Vec::new(),
    }
  }

  pub fn open_handle(&mut self, drive: usize, local: LocalHandle) -> FileHandle {
    let pair = DeviceHandlePair(drive, local);
    self.map.push(Some(pair));
    let index = self.map.len() - 1;
    FileHandle::new(index as u32)
  }

  pub fn get_drive_and_handle(&self, handle: FileHandle) -> Option<DeviceHandlePair> {
    let index = handle.as_usize();
    match self.map.get(index) {
      Some(pair) => *pair,
      None => None,
    }
  }
}
