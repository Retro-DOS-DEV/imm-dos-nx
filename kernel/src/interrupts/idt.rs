use core::mem;
use crate::x86::segments::SegmentSelector;
use super::{exceptions, handlers, pic, stack};

// To maintain full control of the stack when entering / exiting a syscall, we
// use a bit of statically-linked assembly code to handle this interrupt
#[link(name="libsyscall", kind="static")]
extern "x86-interrupt" {
  fn syscall_handler(frame: &stack::StackFrame) -> ();
}

// Flags used in IDT entries
pub const IDT_PRESENT: u8 = 1 << 7;
pub const IDT_DESCRIPTOR_RING_0: u8 = 0;
pub const IDT_DESCRIPTOR_RING_1: u8 = 1 << 5;
pub const IDT_DESCRIPTOR_RING_2: u8 = 2 << 5;
pub const IDT_DESCRIPTOR_RING_3: u8 = 3 << 5;
pub const IDT_GATE_TYPE_TASK_32: u8 = 0x5;
pub const IDT_GATE_TYPE_INT_16: u8 = 0x6;
pub const IDT_GATE_TYPE_TRAP_16: u8 = 0x7;
pub const IDT_GATE_TYPE_INT_32: u8 = 0xe;
pub const IDT_GATE_TYPE_TRAP_32: u8 = 0xf;

/// Used to specify whether an interrupt handler should be implemented as a trap
/// or interrupt. The key difference is whether interrupts are disabled upon
/// entry.
pub enum GateType {
  Interrupt,
  Trap,
}

impl GateType {
  pub fn as_flag(&self) -> u8 {
    match self {
      GateType::Interrupt => IDT_GATE_TYPE_INT_32,
      GateType::Trap => IDT_GATE_TYPE_TRAP_32,
    }
  }
}

/// A function implementing the x86-interrupt "ABI" that will receive a stack
/// frame as its only argument.
pub type HandlerFunction = unsafe extern "x86-interrupt" fn(&stack::StackFrame);
/// A function implementing the x86-interrupt "ABI" that also expects an error
/// code. Some processor exceptions will push an extra 32-bit error code onto
/// the stack which needs to be handled and popped before returning.
pub type HandlerFunctionWithErrorCode = unsafe extern "x86-interrupt" fn(&stack::StackFrame, u32);

/// An IDT Entry tells a x86 CPU how to handle an interrupt.
/// The entry attributes determine how the interrupt is entered, what permission
/// ring and memory selector to use, and which address to enter.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IDTEntry {
  pub offset_low: u16,
  pub selector: SegmentSelector,
  pub _zero: u8,
  pub type_and_attributes: u8,
  pub offset_high: u16,
}

impl IDTEntry {
  pub const fn new() -> IDTEntry {
    IDTEntry {
      offset_low: 0,
      selector: SegmentSelector::new(1, 0),
      _zero: 0,
      type_and_attributes: 0,
      offset_high: 0,
    }
  }

  /// Set the handler function for this entry. When this interrupt occurs, the
  /// CPU will attempt to enter this function using the method described by
  /// the gate type.
  /// 
  /// Various attempts were made to control how the stack was handled upon entry
  /// and exit from the handler. Even with naked functions, there was a lot of
  /// hackiness to fight the LLVM compiler. The introduction of the
  /// x86-interrupt ABI makes this a lot easier to manage.
  pub fn set_handler(&mut self, func: HandlerFunction, gate_type: GateType) {
    let offset = func as *const () as usize;
    self.set_handler_at_offset(offset, gate_type);
  }

  /// Similar to `set_handler`, but used for exceptions that push an extra error
  /// code onto the stack.
  pub fn set_handler_with_error(&mut self, func: HandlerFunctionWithErrorCode, gate_type: GateType) {
    let offset = func as *const () as usize;
    self.set_handler_at_offset(offset, gate_type);
  }

  /// Actual implementation of setting the handler
  fn set_handler_at_offset(&mut self, offset: usize, gate_type: GateType) {
    self.offset_low = offset as u16;
    self.offset_high = (offset >> 16) as u16;
    // Mark the gate as present, and to require ring-0
    self.type_and_attributes = IDT_PRESENT | gate_type.as_flag();
  }

  /// Allow the interrupt to be called from Ring 3. This is necessary for any
  /// syscalls.
  pub fn make_usermode_accessible(&mut self) {
    self.type_and_attributes |= IDT_DESCRIPTOR_RING_3;
  }
}

/// Tell the CPU to change its internal pointer to the IDT Descriptor
#[inline]
pub unsafe fn lidt(desc: &IDTDescriptor) {
  asm!(
    "lidt [{0}]",
    in(reg) (desc as *const IDTDescriptor as usize),
  );
}

// ================
// Below are the actual in-memory data structures used to tell the CPU how to
// handle each interrupt. These will be loaded into the CPU once virtual memory
// has been initialized, but before we need to actually start handling
// interrupts.

/// The IDT itself is a table of 256 interrupt handler entries.
static mut IDT: [IDTEntry; 256] = [IDTEntry::new(); 256];

/// The IDT descriptor is a special in-memory data structure that tells the CPU
/// how to find the actual IDT table. Because the CPU needs to know how many
/// valid entries exist in the table, it requires this extra layer of
/// indirection.
#[repr(C, packed)]
pub struct IDTDescriptor {
  /// The size field contains the byte length, minus one. This is a common
  /// notation type on the x86 CPU, since we can assume that there are more than
  /// zero entries, and because it's designed to support the full range of bytes
  /// stored in a 16-bit unsigned number.
  pub size: u16,
  /// The physical address
  pub offset: u32,
}

/// The structure pointing to the IDT. It needs to be initialized to zero and
/// set up at runtime because Rust doesn't currently support pointer-to-int
/// casting in a constexpr.
static mut IDTR: IDTDescriptor = IDTDescriptor {
  size: 0,
  offset: 0,
};

pub unsafe fn init() {
  IDTR.size = (IDT.len() * mem::size_of::<IDTEntry>() - 1) as u16;
  IDTR.offset = IDT.as_ptr() as *const IDTEntry as u32;

  // Set exception handlers. Right now we mark these all as Interrupt types,
  // which means interrupts are disabled while running them. We do this because
  // the kernel is not guaranteed to be interrupt-safe. Later, we can come back
  // and update one trap at a time until they are all correctly flagged.
  
  // Exception triggered when dividing by zero, or when the result is too large
  // to fit into the destination. If the error occurred in a usermode process,
  // an arithetic error signal will be sent to the program to be optionally
  // handled.
  IDT[0x00].set_handler(exceptions::divide_by_zero, GateType::Interrupt);

  // Exception intentionally triggered by a breakpoint command. The kernel will
  // send a breakpoint trap signal to the process. Any other processes tracing
  // that process (like a debugger) will also be able to intercept that signal
  // and handle it appropriately.
  IDT[0x03].set_handler(exceptions::breakpoint, GateType::Interrupt);

  // Exception triggered when the CPU attempts to execute an invalid instruction
  IDT[0x06].set_handler(exceptions::invalid_opcode, GateType::Interrupt);

  // Exception triggered in a double-fault case. This occurs when an exception
  // can't be handled, often because another exception arose when trying to
  // handle the first exception.
  // Technically it includes an error code, but it is always set to zero.
  IDT[0x08].set_handler_with_error(exceptions::double_fault, GateType::Interrupt);

  // Exception triggered when a selector in the TSS points to an invalid entry.
  IDT[0x0a].set_handler_with_error(exceptions::invalid_tss, GateType::Interrupt);

  // Exception triggered when the CPU attempts to access a segment that does not
  // have a "present" flag set. The error code will be set to the segment index.
  IDT[0x0b].set_handler_with_error(exceptions::segment_not_present, GateType::Interrupt);

  // Exception triggered when the CPU attempts to access a stack segment that
  // does not exist. The error code will be set to the segment index.
  IDT[0x0c].set_handler_with_error(exceptions::stack_segment_fault, GateType::Interrupt);

  // Catch-all exception for privilege errors. The kernel uses this to handle
  // attempts in usermode to run privileged instructions, as well as various
  // actions related to running DOS programs in the 8086 VM.
  IDT[0xd].set_handler_with_error(exceptions::gpf, GateType::Interrupt);

  // Exception that occurs when memory access leads to a page fault. The error
  // value encodes the behavior that caused the fault.
  IDT[0xe].set_handler_with_error(exceptions::page_fault, GateType::Interrupt);

  // Interrupts through 0x1f represent exceptions that we don't handle, usually
  // because they are deprecated or represent hardware functions unsupported by
  // the kernel.

  // Interrupts 0x20-0x2f are reserved to potentially implement their DOS
  // counterparts. The only one used here is 0x2b, which is the entrypoint for
  // user-mode programs to make a syscall.
  IDT[0x2b].set_handler(syscall_handler, GateType::Interrupt);
  IDT[0x2b].make_usermode_accessible();
  
  // Interrupts 0x30-0x3f are reserved for PIC hardware interrupts.
  // This is where we begin to allow processes to install their own interrupt
  // handlers. For example, a COM driver would want to listen to interrupt 0x34.
  // To accommodate this, these interrupts have a handler that runs through a
  // vector of installed hooks before returning to whatever code was running
  // before the interrupt.

  // IRQ 0 is always guaranteed to be the PIT timer chip
  IDT[0x30].set_handler(pic::pit, GateType::Interrupt);
  // IRQ 1 is also guaranteed to be the keyboard PS/2 controller
  IDT[0x31].set_handler(pic::keyboard, GateType::Interrupt);
  // IRQ 2 is the cascade from the second PIC, and unused
  // The rest of the PIC IRQs are a mix of standard connections and ISA
  // interrupts. When PCI devices are available, their interrupts are exposed on
  // unused lines using the Programmable Interrupt Router.
  IDT[0x33].set_handler(irq_3, GateType::Interrupt);
  IDT[0x34].set_handler(pic::com1, GateType::Interrupt);
  //IDT[0x35].set_handler(irq_5, GateType::Interrupt);
  IDT[0x36].set_handler(pic::floppy, GateType::Interrupt);
  //IDT[0x37].set_handler(pic::lpt, GateType::Interrupt);
  //IDT[0x38].set_handler(pic::rtc, GateType::Interrupt);
  //IDT[0x39].set_handler(irq_9, GateType::Interrupt);
  //IDT[0x3a].set_handler(irq_10, GateType::Interrupt);
  //IDT[0x3b].set_handler(irq_11, GateType::Interrupt);
  //IDT[0x3c].set_handler(pic::mouse, GateType::Interrupt);
  //IDT[0x3d].set_handler(pic::fpu, GateType::Interrupt);
  //IDT[0x3e].set_handler(pic::ata_primary, GateType::Interrupt);
  //IDT[0x3f].set_handler(pic::ata_secondary, GateType::Interrupt);

  // With the table initialized, tell the CPU where it is
  lidt(&IDTR);
}

pub extern "x86-interrupt" fn irq_3(_frame: &stack::StackFrame) {
  let handler = match handlers::try_get_installed_handler(3) {
    Some(handler) => handler,
    None => return,
  };
  handlers::enter_handler(handler);
}
