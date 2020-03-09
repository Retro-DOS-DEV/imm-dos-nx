use super::entry::PageTableEntry;
use super::super::address::{PhysicalAddress, VirtualAddress};
use super::super::frame::Frame;

pub struct PageTable {
  pub physical_location: PhysicalAddress,
  pub virtual_location: VirtualAddress,
}

impl PageTable {
  /**
   * Initialize a Page Table within a physical memory frame, using the
   * provided virtual address for access.
   */
  pub fn at_mapped_frame(frame: Frame, vaddr: VirtualAddress) -> PageTable {
    PageTable {
      physical_location: frame.get_address(),
      virtual_location: vaddr,
    }
  }

  fn get_first_entry_mut_ptr(&self) -> *mut PageTableEntry {
    self.virtual_location.as_usize() as *mut PageTableEntry
  }

  pub fn get_table_entry(&self, index: usize) -> PageTableEntry {
    if index >= 1024 {
      panic!("PageTable index out of bounds");
    }
    unsafe {
      let entry_ptr = self.get_first_entry_mut_ptr().offset(index as isize);
      *entry_ptr
    }
  }

  pub fn set_table_entry(&mut self, index: usize, entry: PageTableEntry) {
    if index >= 1024 {
      panic!("PageTable index out of bounds");
    }
    unsafe {
      let entry_ptr = self.get_first_entry_mut_ptr().offset(index as isize);
      *entry_ptr = entry;
    }
  }

  pub fn get_entry_for_address(&self, vaddr: VirtualAddress) -> PageTableEntry {
    let index = (vaddr.as_usize() >> 12) & 0x3ff;
    self.get_table_entry(index)
  }
}
