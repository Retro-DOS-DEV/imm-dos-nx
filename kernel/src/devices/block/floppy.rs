use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::hardware::floppy::{FloppyDiskController};
use crate::memory::address::VirtualAddress;
use crate::task::id::ProcessID;
use spin::RwLock;
use super::super::driver::DeviceDriver;

static CONTROLLER: FloppyDiskController = FloppyDiskController::new();

pub fn init() {
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

pub struct FloppyDriver {
  drive_number: usize,
  next_handle: AtomicUsize,
  open_handles: RwLock<BTreeMap<usize, OpenInstance>>,
}

impl FloppyDriver {
  pub fn new(drive_number: usize) -> Self {
    Self {
      drive_number,
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
    Ok(0)
  }

  fn write(&self, index: usize, buffer: &[u8]) -> Result<usize, ()> {
    Ok(0)
  }
}
