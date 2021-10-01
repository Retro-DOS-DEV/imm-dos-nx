use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::files::cursor::SeekMethod;
use crate::hardware::floppy::{DriveSelect, FloppyDiskController, Operation};
use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::task::id::ProcessID;
use crate::task::memory::MMapBacking;
use spin::RwLock;
use super::geometry::{Sector, SectorRange};
use super::super::driver::DeviceDriver;

static CONTROLLER: FloppyDiskController = FloppyDiskController::new();

static DMA_ADDR: RwLock<Option<(PhysicalAddress, VirtualAddress)>> = RwLock::new(None);
const DMA_SIZE: usize = 4096;

pub fn init() -> (bool, bool) {
  crate::kprintln!("Install Floppy driver");

  let install_result = crate::interrupts::handlers::install_handler(
    6,
    ProcessID::new(0),
    VirtualAddress::new(int_floppy as *const fn () -> () as usize),
    VirtualAddress::new(0),
  );
  if let Err(_) = install_result {
    crate::kprintln!("Failed to install IRQ6");
  }
  
  match CONTROLLER.init() {
    Ok(_) => crate::kprintln!("Floppy device ready"),
    Err(e) => crate::kprintln!("Failed to install Floppy driver: {:?}", e),
  }

  // Set up DMA area
  {
    let virt: VirtualAddress = crate::task::memory::kernel_mmap(None, DMA_SIZE, MMapBacking::DMA).expect("Failed to allocate kernel mmap page");
    let phys: PhysicalAddress = crate::task::paging::get_or_allocate_physical_address(virt).expect("Failed to create DMA frame");
    crate::task::paging::share_kernel_page_directory(virt);
    let mut dma_addr = DMA_ADDR.write();
    *dma_addr = Some((phys, virt));
  }

  (CONTROLLER.has_primary_drive(), CONTROLLER.has_secondary_drive())
}

fn get_dma_addresses() -> (PhysicalAddress, VirtualAddress) {
  loop {
    let addr = {
      *DMA_ADDR.read()
    };
    match addr {
      Some(pair) => return pair,
      None => crate::task::yield_coop(),
    }
  }
}

pub fn load_sectors_to_cache(drive: DriveSelect, sectors: &SectorRange, dma_mode: u8) -> Result<VirtualAddress, ()> {
  let (dma_phys, dma_virt) = get_dma_addresses();
  {
    let channel = super::super::DMA.get_channel(2);
    channel.set_address(dma_phys);
    channel.set_count(sectors.byte_length() - 1);
    channel.set_mode(dma_mode);
  }
  let (c, h, s) = sectors.get_first_sector().to_chs();
  CONTROLLER.add_operation(Operation::Read(drive, c, h, s));
  Ok(dma_virt)
}

pub extern "C" fn int_floppy() {
  CONTROLLER.handle_interrupt();
  crate::interrupts::handlers::return_from_handler(6);
}

pub struct OpenInstance {
  cursor: usize,
}

impl OpenInstance {
  pub fn new() -> Self {
    Self {
      cursor: 0,
    }
  }
}

/// Device driver for interacting with data on a floppy disk. It exposes the
/// floppy disk as a byte stream, and can be used by a filesystem implementation
/// to actually read data on a disk.
/// The floppy driver allows artibrary reads and writes, but the floppy
/// controller only operates at a sector granularity. To accomodate this, the
/// driver maintains an internal LRU cache of sectors that have been read from
/// the disk. Byte-level data can be copied from this in-memory cache.
pub struct FloppyDriver {
  drive_select: DriveSelect,
  next_handle: AtomicUsize,
  open_handles: RwLock<BTreeMap<usize, OpenInstance>>,
}

impl FloppyDriver {
  pub fn new(drive_select: DriveSelect) -> Self {
    Self {
      drive_select,
      next_handle: AtomicUsize::new(0),
      open_handles: RwLock::new(BTreeMap::new()),
    }
  }
}

impl DeviceDriver for FloppyDriver {
  fn open(&self) -> Result<usize, ()> {
    let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
    self.open_handles.write().insert(handle, OpenInstance::new());
    Ok(handle)
  }

  fn close(&self, index: usize) -> Result<(), ()> {
    self.open_handles.write().remove(&index);
    Ok(())
  }

  fn read(&self, index: usize, buffer: &mut [u8]) -> Result<usize, ()> {
    let cursor = match self.open_handles.read().get(&index) {
      Some(open_handle) => Ok(open_handle.cursor),
      None => Err(())
    }?;

    let length = buffer.len();
    let sectors = SectorRange::for_byte_range(cursor, length);

    let dma_src = load_sectors_to_cache(self.drive_select, &sectors, 0x56)?;
    let local_offset = sectors.get_local_offset(cursor);
    let dma_src_ptr = (dma_src.as_usize() + local_offset) as *const u8;
    for i in 0..length {
      unsafe {
        buffer[i] = *dma_src_ptr.offset(i as isize);
      }
    }

    match self.open_handles.write().get_mut(&index) {
      Some(open_file) => {
        open_file.cursor += length;
        Ok(length)
      },
      None => Err(()),
    }
  }

  fn write(&self, index: usize, buffer: &[u8]) -> Result<usize, ()> {
    Ok(0)
  }

  fn seek(&self, index: usize, offset: SeekMethod) -> Result<usize, ()> {
    match self.open_handles.write().get_mut(&index) {
      Some(open_handle) => {
        let next_cursor = offset.from_current_position(open_handle.cursor);
        open_handle.cursor = next_cursor;
        Ok(next_cursor)
      },
      None => Err(())
    }
  }
}
