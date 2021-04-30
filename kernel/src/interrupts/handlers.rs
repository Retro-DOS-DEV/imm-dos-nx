use crate::memory::address::VirtualAddress;
use crate::task::{id::ProcessID, regs::SavedState};
use spin::RwLock;
use super::stack::{FullStackFrame, RestorationStack};

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

/// Install a user-mode handler for a hardware interrupt.
/// At this point only one interrupt handler can be installed for each IRQ
/// number. That's all that should be necessary to implement drivers --
/// conflicting handlers would need to interop with each other anyways.
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

/// Attempt to fetch an installed handler function for an IRQ number.
/// If the fetch fails (the data structure is locked?) or no handler is
/// installed, it will return None.
pub fn try_get_installed_handler(irq: usize) -> Option<InterruptHandler> {
  match INSTALLED[irq].try_read() {
    Some(inner) => *inner,
    None => None,
  }
}

/// InterruptReturnPoint tells the kernel how to resume execution at the point
/// where the interrupt occurred. It tells us which process was executing, and
/// where the instruction and stack pointer were located.
/// Using this information, the kernel can look up the saved registers that need
/// to be restored, and set up the stack for an IRET instruction.
#[derive(Clone, Copy)]
pub struct InterruptReturnPoint {
  pub process: ProcessID,
  pub frame: FullStackFrame,
}

/// Stores the return info for the interrupt currently being executed. It should
/// only be written when an interrupt occurs, and read when that interrupt ends.
/// Since hardware interrupts are exclusive, this should never block.
pub static CURRENT_INTERRUPT: RwLock<Option<InterruptReturnPoint>> = RwLock::new(None);

/// Instruct the kernel to temporarily enter a userspace interrupt handler. When
/// that handler returns, the kernel will be able to properly restore execution
/// state to the point before the interrupt occurred.
pub fn enter_handler(handler: InterruptHandler, irq: usize, registers: &SavedState, frame: &FullStackFrame) {
  let current_id: ProcessID = {
    // Store the stack-saved registers in the current process, so that they can
    // be restored when the interrupt ends. As long as all process access is
    // marked as interrupt-unsafe, this shouldn't block.
    let current_proc_lock = crate::task::switching::get_current_process();
    let mut current_proc = current_proc_lock.write();
    current_proc.save_state(registers);
    *current_proc.get_id()
  };

  crate::kprintln!("GOT HANDLER");

  let mut interrupt_frame = FullStackFrame::empty();
  interrupt_frame.eip = frame.eip;
  interrupt_frame.cs = frame.cs;
  interrupt_frame.eflags = frame.eflags;
  if interrupt_frame.cs & 3 != 0 {
    // Came from a different CPL, need to also store the ESP and SS
    // It should be safe to assume that these exist / are on the stack
    interrupt_frame.esp = frame.esp;
    interrupt_frame.ss = frame.ss;
  }

  crate::kprintln!("When we're done, return to {:x}:{:#010x}", interrupt_frame.cs, interrupt_frame.eip);

  match CURRENT_INTERRUPT.try_write() {
    Some(mut inner) => {
      inner.replace(
        InterruptReturnPoint {
          process: current_id,
          frame: interrupt_frame,
        }
      );
    },
    None => return,
  }
  
  // Switch to the handling process
  // TODO!

  // Modify the interrupt stack to enable returning from the handler
  // For now, we require interrupt and signal handlers to register an explicit
  // stack location that will be used when they execute. Since only one
  // interrupt can happen at a time, a program can safely share a stack between
  // all of its handlers.
  let mut sp = handler.stack_top.as_usize();
  unsafe {
    sp -= 4;
    // The magic number for execution on return is 0xC000000X, where X is the
    // IRQ number.
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

  unreachable!("End of enter_handler");
}

pub fn return_from_handler(irq: usize) {
  crate::kprintln!("Return from IRQ {}", irq);

  // In order to unwind to the original interrupt entry point, we return to the
  // process that was interrupted. Then, we update the stack pointer to the
  // values that were pushed when the irq_core handler called into Rust.

  let return_point: InterruptReturnPoint = match CURRENT_INTERRUPT.try_read() {
    Some(inner) => match *inner {
      Some(point) => point,
      None => panic!("Attempted to return from an IRQ, but none was running"),
    },
    None => panic!("Attempted to return from an IRQ, but return info was locked"),
  };

  // Return to the memory space of the originating process
  // TODO!

  // Set up the stack to restore register state and return to the originating
  // instruction.

  let mut restoration_stack = RestorationStack::empty();
  {
    // TODO: should this cleanly handle a process that was cleaned up while the
    // interrupt was running? Is that even possible?
    let proc_lock = crate::task::switching::get_process(&return_point.process)
      .expect("Tried to return from interrupt into an unknown process");
    let proc = proc_lock.read();
    proc.restore_state(&mut restoration_stack.regs);
  }
  restoration_stack.frame = return_point.frame;

  // Set the stack pointer to the bottom of restoration stack. After this, we'll
  // pop all registers and attempt an IRET.
  let esp = &restoration_stack as *const RestorationStack as usize;
  
  unsafe {
    asm!(
      "mov esp, {esp}
      pop edi
      pop esi
      pop ebp
      pop ebx
      pop edx
      pop ecx
      pop eax
      iretd",

      esp = in(reg) esp,
    );
  }

  unreachable!();
}
