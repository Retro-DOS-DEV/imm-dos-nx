use crate::memory::address::VirtualAddress;
use crate::task::id::ProcessID;
use spin::RwLock;

#[derive(Copy, Clone)]
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

pub fn try_get_installed_handler(irq: usize) -> Option<InterruptHandler> {
  match INSTALLED[3].try_read() {
    Some(inner) => *inner,
    None => None,
  }
}

pub fn enter_handler(handler: InterruptHandler) {
  let process_lock = match crate::task::switching::get_process(&handler.process) {
    Some(p) => p,
    None => return,
  };

  crate::kprintln!("GOT HANDLER");
  // Push the return point onto the process's stack

  // Switch to the process

  // Enter the process with IRET
  let sp = 0x100;
  // IRET pops IP, CS, EFLAGS, SP, SS
  unsafe {
    asm!(
      "push 0x23
      push {esp}
      push 0x00
      push 0x1b
      push {addr}
      iretd",
      esp = in(reg) sp,
      addr = in(reg) handler.function.as_usize(),
    );
  }

  // We return to this spot
}
