use crate::memory::address::VirtualAddress;
use crate::task::id::ProcessID;
use spin::RwLock;

pub struct InterruptHandler {
  process: ProcessID,
  function: VirtualAddress,
}

/// Store an optional installed vectors for each hardware IRQ on the PIC.
/// Some of these will be unused, but we create them all anyways for simplicity.
pub static INSTALLED: [RwLock<Option<InterruptHandler>>; 16] = [
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
  RwLock::new(None),
];

pub fn install_handler(irq: usize, process: ProcessID, function: VirtualAddress) -> Result<(), ()> {
  if irq >= 16 {
    return Err(());
  }
  match INSTALLED[irq].try_write() {
    Some(mut handler) => {
      *handler = Some(
        InterruptHandler {
          process,
          function,
        }
      );
    },
    None => {
      // The entry is locked. Are you trying to install a handler during an
      // interrupt?
      return Err(());
    },
  }
  Ok(())
}
