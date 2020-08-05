use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::devices;
use crate::drivers::driver::DeviceDriver;
use crate::files::cursor::SeekMethod;
use crate::files::handle::{Handle, HandleAllocator, LocalHandle};
use crate::memory::address::VirtualAddress;
use spin::RwLock;
use super::directory;
use super::disk::{BiosParamBlock, Cluster, DiskConfig};
use super::super::filesystem::FileSystem;
use syscall::files::DirEntryInfo;

struct OpenFile {
  pub cursor: usize,
  pub clusters: Vec<Cluster>,
}

pub struct Fat12FileSystem {
  handle_allocator: HandleAllocator<LocalHandle>,
  open_files: RwLock<BTreeMap<LocalHandle, OpenFile>>,

  drive_number: usize,
  config: DiskConfig,
  io_buffer: RwLock<Vec<u8>>,
}

impl Fat12FileSystem {
  pub fn new(drive_number: usize) -> Fat12FileSystem {
    let mut io_buffer = Vec::with_capacity(512);
    for _ in 0..512 {
      io_buffer.push(0);
    }
    Fat12FileSystem {
      handle_allocator: HandleAllocator::new(),
      open_files: RwLock::new(BTreeMap::new()),

      drive_number,
      config: DiskConfig::empty(),
      io_buffer: RwLock::new(io_buffer),
    }
  }

  pub fn init(&mut self) -> Result<(), ()> {
    let handle = self.handle_allocator.get_next();
    let driver = devices::get_driver_for_device(self.drive_number).ok_or(())?;
    driver.open(handle)?;
    driver.seek(handle, SeekMethod::Absolute(0x0b))?;
    let mut bpb = BiosParamBlock::empty();
    driver.read(handle, bpb.as_buffer())?;
    self.config.from_bpb(&bpb);
    Ok(())
  }

  fn get_io_buffer_address(&self) -> VirtualAddress {
    VirtualAddress::new(self.io_buffer.read().as_ptr() as usize)
  }
}

impl FileSystem for Fat12FileSystem {
  fn open(&self, path: &str) -> Result<LocalHandle, ()> {
    let handle = self.handle_allocator.get_next();
    let driver = devices::get_driver_for_device(self.drive_number).ok_or(())?;
    driver.open(handle)?;
    driver.seek(handle, SeekMethod::Absolute(0x2600))?;
    {
      let mut buffer = self.io_buffer.write();
      driver.read(handle, buffer.as_mut_slice())?;
    }

    let buffer_addr = self.get_io_buffer_address();
    let iter = directory::DirectoryEntryIterator::new(buffer_addr, 16);
    crate::tty::console_write(format_args!("Root Directory:\n"));
    for entry in iter {
      let (name, ext) = unsafe {
        (
          core::str::from_utf8_unchecked(entry.get_name()),
          core::str::from_utf8_unchecked(entry.get_ext()),
        )
      };
      crate::tty::console_write(format_args!("  {}.{}, {} bytes\n", name, ext, entry.get_byte_size()));
    }

    Err(())
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    Err(())
  }

  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Err(())
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    Err(())
  }

  fn dup(&self, handle: LocalHandle) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()> {
    Err(())
  }

  fn open_dir(&self, path: &str) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn read_dir(&self, handle: LocalHandle, index: usize, info: &mut DirEntryInfo) -> Result<(), ()> {
    Err(())
  }
}