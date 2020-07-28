use crate::devices;
use crate::files::handle::LocalHandle;
use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::process;
use spin::RwLock;
use super::driver::DeviceDriver;

/// Device driver for interacting with data on a floppy disk. It exposes the
/// floppy disk as a byte stream, and can be used by a filesystem implementation
/// to actually read data on a disk.
/// The floppy driver allows artibrary reads and writes, but the floppy
/// controller only operates at a sector granularity. To accomodate this, the
/// driver maintains an internal LRU cache of sectors that have been read from
/// the disk. Byte-level data can be copied from this in-memory cache.
pub struct FloppyDevice {
  drive_number: usize,
}

impl FloppyDevice {
  pub fn new(drive_number: usize) -> FloppyDevice {
    FloppyDevice {
      drive_number,
    }
  }
}

impl DeviceDriver for FloppyDevice {
  fn open(&self, _handle: LocalHandle) -> Result<(), ()> {
    Ok(())
  }

  fn close(&self, _handle: LocalHandle) -> Result<(), ()> {
    Ok(())
  }

  fn read(&self, _handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let (dma_phys, dma_virt) = get_dma_addresses();
    {
      let channel = devices::DMA.get_channel(2);
      channel.set_address(dma_phys);
      // Round up to the next sector
      let mut length = (buffer.len() / 512) * 512;
      if buffer.len() & 511 > 0 {
        length += 512;
      }
      channel.set_count(length);
      channel.set_mode(0x56);
    }
    crate::kprintln!("DMA READY");
    devices::FLOPPY.read(0, 0, 1).map_err(|_| ())?;
    crate::kprintln!("READ DONE");
    let dma_src = dma_virt.as_usize() as *const u8;
    for i in 0..buffer.len() {
      unsafe {
        buffer[i] = *dma_src.offset(i as isize);
      }
    }

    Ok(buffer.len())
  }

  fn write(&self, _handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(0)
  }
}

static DMA_ADDR: RwLock<Option<(PhysicalAddress, VirtualAddress)>> = RwLock::new(None);

const DMA_SIZE: usize = 4096;

pub fn init_dma() {
  let address_pair = process::current_process().unwrap().kernel_mmap_dma(DMA_SIZE);
  let mut dma_addr = DMA_ADDR.write();
  *dma_addr = Some(address_pair);
  crate::kprintln!("Floppy DMA at {:?}/{:?}", address_pair.0, address_pair.1);
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
