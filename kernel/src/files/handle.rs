use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

pub trait Handle {
  fn new(handle: u32) -> Self;
  fn as_u32(&self) -> u32;
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

struct DeviceHandlePair(pub usize, pub LocalHandle);

/**
 * Map a process's file handles to the filesystem and fs-specific handle behind
 * each one.
 */
pub struct FileHandleMap {
  map: Vec<Option<DeviceHandlePair>>,
}
