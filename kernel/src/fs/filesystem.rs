//! Filesystems allow the kernel to interact with files in a standardized way.
//! They may represent real files on a disk, like a FAT12 filesystem, or they
//! may be virtual and represent resources belonging to processes or devices
//! attached to the system ("PROC:" and "DEV:", respectively).
//! 
//! There are three different types of filesystems: Kernel Sync, Kernel Async,
//! and Userspace.
//! 
//! Kernel Sync is the simplest form of filesystem, and is used for information
//! that is immediately available. They exist only in the kernel, and don't
//! require any context switching. When a file IO syscall arrives, the kernel
//! calls into the filesystem's handler, copies the data, and immediately
//! returns. This is only possible for filesystems that report system data like
//! PROC, or return in-memory values like INIT.
//! 
//! Kernel Async filesystems are used anytime a data read or write could be
//! blocking. They exist in standalone kernel-level processes, and are able to
//! talk to devices and wake up external threads as needed. When a process tries
//! to read or write data, it is marked as blocked on IO and yields. The
//! filesystem process the request, and wakes up the caller when it's complete.
//! 
//! Userspace filesystems run outside the kernel, and can therefore be executed
//! from disk. Supporting a new format works by implementing a standalone driver
//! and telling the OS to use it when the disk is mounted. They communicate with
//! the kernel through an Arbiter, which is essentially another Kernel Async
//! filesystem that farms out requests to different processes. Any time a
//! program makes a file operation to a drive backed by a userspace filesystem,
//! the request is enqueued in the Arbiter, which determines which process to
//! talk to and sends the request via IPC. Reading / writing the originating
//! process's memory is done via shared memory grants. When the operation is
//! complete, the filesystem sends a response back to the Arbiter via IPC, which
//! will wake the caller and let it continue execution.
//! Although it wasn't intentionally used as a model, this is pretty much the
//! same approach taken by Filesystem in Userspace (FUSE) on Unix systems.

use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::files::cursor::SeekMethod;
use crate::files::handle::LocalHandle;
use syscall::files::{DirEntryInfo, FileStatus};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FileSystemCategory {
  KernelSync,
  KernelAsync,
  Userspace,
}

/// All filesystems compiled into the kernel need to implement this trait to
/// support the standard set of file operations.
pub trait KernelFileSystem {
  #![allow(unused_variables)]

  /// Create a new reference to a file. If it exists, it will return a unique
  /// reference to this instance of the open file. Any further operations on the
  /// file depend upon this handle.
  fn open(&self, path: &str) -> Result<LocalHandle, ()>;

  /// Copy bytes from the file to a local buffer. On success, it will return the
  /// number of bytes copied.
  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()>;

  /// Copy bytes from a local buffer into a file. On success, it will return the
  /// number of bytes copied.
  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()>;

  /// Close out a reference to a file. The handle will no longer be usable.
  fn close(&self, handle: LocalHandle) -> Result<(), ()>;

  /// Create a duplicate reference to an existing handle.
  fn dup(&self, handle: LocalHandle) -> Result<LocalHandle, ()>;

  /// Update the cursor that determines the starting point for reads and writes.
  /// On success, it returns the new cursor location.
  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()>;
  
  /// Create a new reference to a directory. Dir references contain their own
  /// internal cursor as they are read, so successive calls to read_dir iterate
  /// through the entries.
  fn open_dir(&self, path: &str) -> Result<LocalHandle, ()>;

  /// Read information about the next entry in an open directory. Fields are
  /// copied into a DirEntryInfo struct. If the entry was copied, the method
  /// resolves with `true`. If there are no more entries, the method resolves
  /// with `false`. Any errors while fetching the entry will return an Err
  /// value instead.
  fn read_dir(&self, handle: LocalHandle, index: usize, info: &mut DirEntryInfo) -> Result<bool, ()>;

  /// Perform a unique FS operation on a file. IOCTL command numbers depend on
  /// the device and FS.label_ro_physical_start
  fn ioctl(&self, handle: LocalHandle, command: u32, arg: u32) -> Result<u32, ()> {
    Err(())
  }

  /// Fetch status information about an open file. If successful, the data will
  /// be copied into a FileStatus struct.
  fn stat(&self, handle: LocalHandle, status: &mut FileStatus) -> Result<(), ()>;
}

pub type FileSystemType = dyn KernelFileSystem + Send + Sync;

pub struct FileSystemInstance {
  pub category: FileSystemCategory,
  pub name: Box<str>,
  pub instance: Arc<Box<FileSystemType>>,
}

impl FileSystemInstance {
  pub fn matches_name(&self, name: &str) -> bool {
    self.name.as_ref() == name
  }

  pub fn get_category(&self) -> FileSystemCategory {
    self.category
  }

  pub fn get_fs(&self) -> Arc<Box<FileSystemType>> {
    self.instance.clone()
  }
}
