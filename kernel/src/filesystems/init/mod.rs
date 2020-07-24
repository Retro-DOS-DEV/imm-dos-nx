use alloc::collections::BTreeMap;
use crate::files::handle::{HandleAllocator, LocalHandle};
use crate::memory::address::VirtualAddress;
use spin::RwLock;
use super::filesystem::FileSystem;

struct OpenFile {
  pub cursor: usize,
  pub length: usize,
  pub start: usize,
}

pub struct InitFileSystem {
  handle_allocator: HandleAllocator<LocalHandle>,
  cpio_archive_address: VirtualAddress,
  open_files: RwLock<BTreeMap<LocalHandle, OpenFile>>,
}

impl InitFileSystem {
  pub fn new(addr: VirtualAddress) -> InitFileSystem {
    InitFileSystem {
      handle_allocator: HandleAllocator::<LocalHandle>::new(),
      cpio_archive_address: addr,
      open_files: RwLock::new(BTreeMap::new()),
    }
  }
}

impl FileSystem for InitFileSystem {
  fn open(&self, path: &str) -> Result<LocalHandle, ()> {
    let local_path = if path.starts_with('\\') {
      &path[1..]
    } else {
      path
    };

    let iter = CpioIterator::new(self.cpio_archive_address.as_usize());
    for entry in iter {
      if entry.get_filename_str() == local_path {
        let handle = self.handle_allocator.get_next();
        let open_file = OpenFile {
          start: entry.get_content_ptr() as usize,
          length: entry.get_file_size(),
          cursor: 0,
        };
        self.open_files.write().insert(handle, open_file);
        return Ok(handle);
      }
    }

    Err(())
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let file_ptr = match self.open_files.write().get_mut(&handle) {
      Some(open_file) => {
        let mut to_read = buffer.len();
        let bytes_left_in_file = open_file.length - open_file.cursor;
        if bytes_left_in_file < to_read {
          to_read = bytes_left_in_file;
        }
        let prev_cursor = open_file.cursor;
        open_file.cursor += to_read;
        Some((open_file.start + prev_cursor, to_read))
      },
      None => None,
    };

    match file_ptr {
      Some((address, to_read)) => unsafe {
        let start = address as *const u8;
        for offset in 0..to_read {
          let ptr = start.offset(offset as isize);
          buffer[offset] = *ptr;
        }
        Ok(to_read)
      },
      None => Err(())
    }
  }

  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(0)
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    Err(())
  }

  fn dup(&self, handle: LocalHandle) -> Result<LocalHandle, ()> {
    Err(())
  }
}

const TRAILER: &[u8] = "TRAILER!!!".as_bytes();

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
