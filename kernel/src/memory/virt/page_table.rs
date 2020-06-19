use super::page_directory;
use super::page_entry::PageTableEntry;
use super::super::address::{PhysicalAddress, VirtualAddress};

pub const TABLE_ENTRY_COUNT: usize = 1024;

pub const SELF_REFERENCE_INDEX: usize = 1023;
pub const TEMP_REFERENCE_INDEX: usize = 1022;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PageTable([PageTableEntry; TABLE_ENTRY_COUNT]);

impl PageTable {
  pub fn at_address(addr: VirtualAddress) -> &'static mut PageTable {
    let ptr = addr.as_usize() as *mut PageTable;
    unsafe { &mut *ptr }
  }

  pub fn zero(&mut self) {
    for index in 0..1024 {
      self.0[index].zero();
    }
  }

  pub fn get(&self, index: usize) -> &PageTableEntry {
    &self.0[index & 0x3ff]
  }

  pub fn get_mut(&mut self, index: usize) -> &mut PageTableEntry {
    &mut self.0[index & 0x3ff]
  }
}

#[derive(Copy, Clone)]
pub struct PageTableReference {
  address: PhysicalAddress,
}

impl PageTableReference {
  pub fn new(address: PhysicalAddress) -> PageTableReference {
    PageTableReference {
      address
    }
  }

  pub fn current() -> PageTableReference {
    let address = page_directory::get_current_pagedir();
    PageTableReference {
      address,
    }
  }

  pub fn get_address(&self) -> PhysicalAddress {
    self.address
  }

  pub fn make_active(&self) {
    page_directory::set_current_pagedir(self.address)
  }

  pub fn is_active(&self) -> bool {
    let current = page_directory::get_current_pagedir();
    self.address == current
  }
}
