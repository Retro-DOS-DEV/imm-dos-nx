use crate::memory::address::VirtualAddress;
use crate::task::id::ProcessID;
use spin::RwLock;

#[derive(Copy, Clone)]
pub struct InterruptHandler {
  process: ProcessID,
  function: VirtualAddress,
  stack_top: VirtualAddress,
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

pub fn install_handler(irq: usize, process: ProcessID, function: VirtualAddress, stack_top: VirtualAddress) -> Result<(), ()> {
  if irq >= 16 {
    return Err(());
  }
  if stack_top.as_usize() < core::mem::size_of::<usize>() {
    // Need enough space on the stack to push the return address
    return Err(());
  }
  match INSTALLED[irq].try_write() {
    Some(mut handler) => {
      *handler = Some(
        InterruptHandler {
          process,
          function,
          stack_top,
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
  match INSTALLED[irq].try_read() {
    Some(inner) => *inner,
    None => None,
  }
}

pub fn enter_handler(handler: InterruptHandler, irq: usize) {
  let process_lock = match crate::task::switching::get_process(&handler.process) {
    Some(p) => p,
    None => return,
  };

  crate::kprintln!("GOT HANDLER");
  // Switch to the process

  // Modify the interrupt stack to enable returning from the handler
  // For now, we require interrupt and signal handlers to register an explicit
  // stack location that will be used when they execute. Since only one
  // interrupt can happen at a time, a program can safely share a stack between
  // all of its handlers.
  let mut sp = handler.stack_top.as_usize();
  unsafe {
    sp -= 4;
    (sp as *mut usize).write(0xc0000000 + irq);
  }

  // Enter the process with IRET
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

pub fn return_from_handler(irq: usize) {
  crate::kprintln!("Return from IRQ {}", irq);

  // We need to unwind whatever happened when the original hardware interrupt
  // occurred. At the very least, we need to be able to restore all registers
  // and return to the instruction / permission level that was interrupted.
  // This could be accommodated by ALWAYS storing ALL registers on the kernel
  // stack when entering an interrupt. The interrupt itself will push
  // IP/CS/FLAGS/SP/SS/etc... After that, we can push all registers, and store
  // the stack pointer in a value on the task state. To restore, we only need to
  // update the stack pointer to that value, pop the registers, and call IRETD. 

  loop {}
}
