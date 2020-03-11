use core::mem;

use crate::{interrupt, interrupt_with_error};
use crate::interrupts;
use crate::x86::segments::SegmentSelector;

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

  pub fn set_handler(&mut self, func: unsafe extern fn() -> !) {
    let offset = func as *const () as usize;
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
  asm!("lidt [$0]" : : "r"(desc as *const IDTDescriptor as usize) : : "intel", "volatile");
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
  IDT[0].set_handler(interrupt!(interrupts::exceptions::divide_by_zero));

  IDT[8].set_handler(interrupt!(interrupts::exceptions::double_fault));

  IDT[0xd].set_handler(interrupt_with_error!(interrupts::exceptions::gpf));
  IDT[0xe].set_handler(interrupt_with_error!(interrupts::exceptions::page_fault));

  IDT[0x21].set_handler(interrupt!(interrupts::syscall_legacy::dos_api));
  
  IDT[0x2b].set_handler(interrupt!(interrupts::syscall::syscall_handler));

  lidt(&IDTR);
}
