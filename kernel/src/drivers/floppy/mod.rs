use alloc::collections::BTreeMap;
use crate::devices;
use crate::files::cursor::SeekMethod;
use crate::files::handle::LocalHandle;
use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::process;
use spin::RwLock;
use super::driver::DeviceDriver;

pub mod sector;

use sector::{Sector, SectorRange};

/// Device driver for interacting with data on a floppy disk. It exposes the
/// floppy disk as a byte stream, and can be used by a filesystem implementation
/// to actually read data on a disk.
/// The floppy driver allows artibrary reads and writes, but the floppy
/// controller only operates at a sector granularity. To accomodate this, the
/// driver maintains an internal LRU cache of sectors that have been read from
/// the disk. Byte-level data can be copied from this in-memory cache.
pub struct FloppyDevice {
  drive_number: usize,
  open_files: RwLock<BTreeMap<LocalHandle, OpenFile>>,
}

impl FloppyDevice {
  pub fn new(drive_number: usize) -> FloppyDevice {
    FloppyDevice {
      drive_number,
      open_files: RwLock::new(BTreeMap::new()),
    }
  }
}

impl DeviceDriver for FloppyDevice {
  fn open(&self, handle: LocalHandle) -> Result<(), ()> {
    let open_file = OpenFile {
      cursor: 0,
    };
    self.open_files.write().insert(handle, open_file);
    Ok(())
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    self.open_files.write().remove(&handle);
    Ok(())
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let cursor = match self.open_files.read().get(&handle) {
      Some(open_file) => Ok(open_file.cursor),
      None => Err(())
    }?;

    let length = buffer.len();
    let sectors = SectorRange::for_byte_range(cursor, length);

    let dma_src = load_sectors_to_cache(&sectors, 0x56)?;
    let local_offset = sectors.get_local_offset(cursor);
    let dma_src_ptr = (dma_src.as_usize() + local_offset) as *const u8;
    for i in 0..length {
      unsafe {
        buffer[i] = *dma_src_ptr.offset(i as isize);
      }
    }

    match self.open_files.write().get_mut(&handle) {
      Some(open_file) => {
        open_file.cursor += length;
        Ok(length)
      },
      None => Err(()),
    }
  }

  fn write(&self, _handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(0)
  }

  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()> {
    match self.open_files.write().get_mut(&handle) {
      Some(open_file) => {
        let new_cursor = offset.from_current_position(open_file.cursor);
        open_file.cursor = new_cursor;
        Ok(new_cursor)
      },
      None => Err(())
    }
  }
}

/// Stores metadata associated with a currently open file handle
struct OpenFile {
  pub cursor: usize,
}

static DMA_ADDR: RwLock<Option<(PhysicalAddress, VirtualAddress)>> = RwLock::new(None);

const DMA_SIZE: usize = 4096;

pub fn init_dma() {
  let address_pair = process::current_process().unwrap().kernel_mmap_dma(DMA_SIZE);
  let mut dma_addr = DMA_ADDR.write();
  *dma_addr = Some(address_pair);
  crate::tty::console_write(format_args!("Floppy DMA at {:?}/{:?}\n", address_pair.0, address_pair.1));
}

pub fn get_dma_addresses() -> (PhysicalAddress, VirtualAddress) {
  loop {
    let addr = {
      *DMA_ADDR.read()
    };
    match addr {
      Some(pair) => return pair,
      None => process::yield_coop(),
    }
  }
}

pub fn load_sectors_to_cache(sectors: &SectorRange, dma_mode: u8) -> Result<VirtualAddress, ()> {
  let (dma_phys, dma_virt) = get_dma_addresses();
  {
    let channel = devices::DMA.get_channel(2);
    channel.set_address(dma_phys);
    channel.set_count(sectors.byte_length() - 1);
    channel.set_mode(dma_mode);
  }
  let (c, h, s) = sectors.get_first_sector().to_chs();
  devices::FLOPPY.read(c, h, s).map_err(|_| ())?;
  Ok(dma_virt)
}
