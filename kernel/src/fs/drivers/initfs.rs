//! InitFS is a simple in-memory file archive based on the CPIO format.
//! Files are read-only and stored linearly. Finding a file within the archive
//! is an O(n) operation that traverses each item until a matching filename is
//! found.

use crate::collections::SlotList;
use crate::files::{cursor::SeekMethod, handle::{Handle, LocalHandle}};
use crate::memory::address::VirtualAddress;
use spin::RwLock;
use crate::fs::KernelFileSystem;
use crate::task::id::ProcessID;
use syscall::files::{DirEntryInfo, FileStatus};

#[derive(Clone)]
struct OpenFile {
  pub cursor: usize,
  pub length: usize,
  pub header_start: usize,
  pub file_start: usize,
}

pub struct InitFileSystem {
  cpio_archive_address: VirtualAddress,
  open_files: RwLock<SlotList<OpenFile>>,
}

impl InitFileSystem {
  /// Create an instance of an in-memory filesystem at a specific address. The
  /// filesystem will read entries until it reaches the "TRAILER" entry at the
  /// end of the archive.
  pub fn new(addr: VirtualAddress) -> InitFileSystem {
    InitFileSystem {
      cpio_archive_address: addr,
      open_files: RwLock::new(SlotList::new()),
    }
  }
}

impl KernelFileSystem for InitFileSystem {
  fn open(&self, path: &str) -> Result<LocalHandle, ()> {
    let local_path = if path.starts_with('\\') {
      &path[1..]
    } else {
      path
    };

    let iter = CpioIterator::new(self.cpio_archive_address.as_usize());
    for entry in iter {
      if entry.get_filename_str() == local_path {
        let open_file = OpenFile {
          header_start: entry as *const CpioHeader as usize,
          file_start: entry.get_content_ptr() as usize,
          length: entry.get_file_size(),
          cursor: 0,
        };
        let index = self.open_files.write().insert(open_file);
        return Ok(LocalHandle::new(index as u32));
      }
    }

    Err(())
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let (address, to_read) = match self.open_files.write().get_mut(handle.as_usize()) {
      Some(open_file) => {
        let mut to_read = buffer.len();
        let bytes_left_in_file = open_file.length - open_file.cursor;
        if bytes_left_in_file < to_read {
          to_read = bytes_left_in_file;
        }
        let prev_cursor = open_file.cursor;
        open_file.cursor += to_read;
        Ok((open_file.file_start + prev_cursor, to_read))
      },
      None => Err(()),
    }?;

    let start = address as *const u8;
    unsafe {
      for offset in 0..to_read {
        let ptr = start.offset(offset as isize);
        buffer[offset] = *ptr;
      }
    }
    Ok(to_read)
  }

  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(0)
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    let index = handle.as_usize();
    self.open_files
      .write()
      .remove(index)
      .map_or(Err(()), |_| Ok(()))
  }

  fn reopen(&self, handle: LocalHandle, id: ProcessID) -> Result<LocalHandle, ()> {
    let reopened_file= match self.open_files.write().get_mut(handle.as_usize()) {
      Some(open_file) => Ok(open_file.clone()),
      None => Err(()),
    }?;
    let index = self.open_files.write().insert(reopened_file);
    Ok(LocalHandle::new(index as u32))
  }

  fn ioctl(&self, handle: LocalHandle, command: u32, arg: u32) -> Result<u32, ()> {
    Err(())
  }

  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()> {
    match self.open_files.write().get_mut(handle.as_usize()) {
      Some(open_file) => {
        let new_cursor = offset.from_current_position(open_file.cursor);
        open_file.cursor = new_cursor;
        Ok(new_cursor)
      },
      None => Err(())
    }
  }

  fn open_dir(&self, path: &str) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn read_dir(&self, handle: LocalHandle, index: usize, info: &mut DirEntryInfo) -> Result<bool, ()> {
    Err(())
  }

  fn stat(&self, handle: LocalHandle, status: &mut FileStatus) -> Result<(), ()> {
    let start = match self.open_files.read().get(handle.as_usize()) {
      Some(open_file) => Ok(open_file.header_start),
      None => Err(()),
    }?;
    let header: &CpioHeader = unsafe { &*(start as *const CpioHeader) };
    status.byte_size = header.get_file_size();
    Ok(())
  }
}

const TRAILER: &[u8] = "TRAILER!!!".as_bytes();

/// CPIO archives consist of a series of files with headers using this format.
#[repr(packed)]
pub struct CpioHeader {
  pub magic: u16,
  _device: u16,
  _inode: u16,
  file_mode: u16,
  _owner_uid: u16,
  _owner_gid: u16,
  _link_count: u16,
  _device_no: u16,
  modification_time: u32,
  pub name_size: u16,
  file_size_high: u16,
  file_size_low: u16,
}

impl CpioHeader {
  pub fn at_offset(addr: usize) -> &'static CpioHeader {
    unsafe {
      &*(addr as *const CpioHeader)
    }
  }

  pub fn is_valid(&self) -> bool {
    self.magic == 0x71c7
  }

  fn get_header_ptr(&self) -> *const u8 {
    self as *const CpioHeader as *const u8
  }

  pub fn get_filename_ptr(&self) -> *const u8 {
    unsafe { self.get_header_ptr().offset(26) }
  }

  pub fn get_file_size(&self) -> usize {
    ((self.file_size_high as usize) << 16) | (self.file_size_low as usize)
  }

  pub fn get_content_ptr(&self) -> *const u8 {
    let header_ptr = self.get_header_ptr();
    let filename_ptr = self.get_filename_ptr();
    let mut file_start = unsafe {
      filename_ptr.offset(self.name_size as isize)
    };
    // File must start on a 2-byte barrier
    if ((file_start as usize) - (header_ptr as usize)) & 1 != 0 {
      file_start = unsafe { file_start.offset(1) };
    }
    file_start
  }

  pub fn get_filename(&self) -> &[u8] {
    unsafe {
      core::slice::from_raw_parts(self.get_filename_ptr(), self.name_size as usize - 1)
    }
  }

  pub fn get_filename_str(&self) -> &str {
    core::str::from_utf8(self.get_filename()).unwrap()
  }

  pub fn is_trailer(&self) -> bool {
    let filename = unsafe {
      core::slice::from_raw_parts(self.get_filename_ptr(), TRAILER.len())
    };
    filename == TRAILER
  }

  pub fn length(&self) -> usize {
    let mut filename_length = self.name_size as usize;
    if filename_length & 1 != 0 {
      filename_length += 1;
    }
    let mut file_length = self.get_file_size();
    if file_length & 1 != 0 {
      file_length += 1;
    }
    26 + filename_length + file_length
  }
}

pub struct CpioIterator {
  address: usize,
}

impl CpioIterator {
  pub fn new(address: usize) -> CpioIterator {
    CpioIterator {
      address
    }
  }
}

impl Iterator for CpioIterator {
  type Item = &'static CpioHeader;

  fn next(&mut self) -> Option<Self::Item> {
    let entry = CpioHeader::at_offset(self.address);
    if entry.is_trailer() {
      None
    } else {
      self.address += entry.length();
      Some(entry)
    }
  }
}
