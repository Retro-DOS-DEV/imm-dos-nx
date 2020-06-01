use core::mem;

use crate::interrupts;
use crate::x86::segments::SegmentSelector;

#[link(name="libsyscall", kind="static")]
extern "x86-interrupt" {
  fn syscall_handler(frame: &interrupts::stack::StackFrame) -> ();
}

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

  /**
   * Previously, this was able to implement handlers as naked cdecl ABI methods.
   * Since LLVM changed to no longer mark function pointers as constants in
   * inline assembly, the only way to guarantee the registers touched in a
   * handler are properly restored at the end is to use the x86-interrupt ABI,
   * which is a bit black-box-y, but gets the job done.
   */
  pub fn set_handler(&mut self, func: unsafe extern "x86-interrupt" fn(&interrupts::stack::StackFrame)) {
    let offset = func as *const () as usize;
    self.set_handler_at_offset(offset);
  }

  pub fn set_handler_with_error(&mut self, func: unsafe extern "x86-interrupt" fn(&interrupts::stack::StackFrame, u32)) {
    let offset = func as *const () as usize;
    self.set_handler_at_offset(offset);
  }

  fn set_handler_at_offset(&mut self, offset: usize) {
    self.offset_low = offset as u16;
    self.offset_high = (offset >> 16) as u16;
    self.type_and_attributes = IDT_PRESENT | IDT_DESCRIPTOR_RING_0 | IDT_GATE_TYPE_INT_32;
  }
}

#[repr(C, packed)]
pub struct IDTDescriptor {
  pub size: u16,
  pub offset: u32,
}

#[inline]
pub unsafe fn lidt(desc: &IDTDescriptor) {
  llvm_asm!("lidt [$0]" : : "r"(desc as *const IDTDescriptor as usize) : : "intel", "volatile");
}

// Global Tables:

static mut IDTR: IDTDescriptor = IDTDescriptor {
  size: 0,
  offset: 0,
};

static mut IDT: [IDTEntry; 256] = [IDTEntry::new(); 256];

pub unsafe fn init() {
  IDTR.size = (IDT.len() * mem::size_of::<IDTEntry>() - 1) as u16;
  IDTR.offset = IDT.as_ptr() as *const IDTEntry as u32;

  // Set exception handlers
  IDT[0].set_handler(interrupts::exceptions::divide_by_zero);

  IDT[8].set_handler(interrupts::exceptions::double_fault);

  IDT[0xd].set_handler_with_error(interrupts::exceptions::gpf);
  IDT[0xe].set_handler_with_error(interrupts::exceptions::page_fault);

  //IDT[0x21].set_handler(interrupts::syscall_legacy::dos_api);
  
  IDT[0x2b].set_handler(syscall_handler);

  IDT[0x30].set_handler(interrupts::pic::pit);

  lidt(&IDTR);
}
