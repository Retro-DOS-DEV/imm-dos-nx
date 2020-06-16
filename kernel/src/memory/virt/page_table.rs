use super::page_directory;
use super::page_entry::{self, PageTableEntry};
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

  pub fn get(&self, index: usize) -> &PageTableEntry {
    &self.0[index & 0x3ff]
  }

  pub fn get_mut(&mut self, index: usize) -> &mut PageTableEntry {
    &mut self.0[index & 0x3ff]
  }
}

pub struct PageTableReference {
  address: PhysicalAddress,
}

impl PageTableReference {
  pub fn current() -> PageTableReference {
    let address = page_directory::get_current_pagedir();
    PageTableReference {
      address,
    }
  }

  pub fn make_active(&self) {
    page_directory::set_current_pagedir(self.address)
  }

  pub fn is_active(&self) -> bool {
    let current = page_directory::get_current_pagedir();
    self.address == current
  }

  /**
   * Make it possible to edit this page table, and return the starting virtual
   * address of the 4MiB area it has been mapped to.
   * If this page is active, its last entry is mapped to itself, so it is
   * readable in the last 4KiB of memory.
   * If it is inactive, we reserve a 4MiB spot immediately below the
   * self-mapped area for editing inactive page tables. It will map the
   * second-to-last entry of the active page table to itself, making it visible.
   * Because of the self-mapping, the page itself will be available in the last
   * 4 KiB of the 4 MiB section returned.
   */
  pub fn make_editable(&self) -> VirtualAddress {
    if self.is_active() {
      VirtualAddress::new(0xffc00000)
    } else {
      // map it into the active table
      let active_table = PageTable::at_address(VirtualAddress::new(0xfffff000));
      let temp_space = active_table.get_mut(TEMP_REFERENCE_INDEX);
      temp_space.set_address(self.address);
      temp_space.set_flags(page_entry::ENTRY_PRESENT);
      // flush all 1024 nested tables
      for i in 0..1024 {
        let addr: usize = 0xff800000 + (0x1000 * i);
        // flush(addr);
      }
      VirtualAddress::new(0xff800000)
    }
  }
}
