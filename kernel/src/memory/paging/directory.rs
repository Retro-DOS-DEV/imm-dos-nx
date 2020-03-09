use crate::x86::registers::{get_cr3, set_cr3};
use super::entry::PageTableEntry;
use super::super::address::{PhysicalAddress, VirtualAddress};
use super::super::frame::Frame;

/**
 * We need to know the physical address of each page directory in order to
 * enable it, and the virtual address in order to read / write it.
 */
pub struct PageDirectory {
  pub physical_location: PhysicalAddress,
  pub virtual_location: VirtualAddress,
}

impl PageDirectory {
  /**
   * Initialize a Page Directory within a physical memory frame, using the
   * provided virtual address for access.
   */
  pub fn at_mapped_frame(frame: Frame, vaddr: VirtualAddress) -> PageDirectory {
    PageDirectory {
      physical_location: frame.get_address(),
      virtual_location: vaddr,
    }
  }

  /**
   * In order to make currently-active tables readable/writable, we set the last
   * entry to itself, making the last 4KiB of memory point to the directory.
   */
  pub fn set_last_entry_to_self(&mut self) {
    let mut entry = PageTableEntry::new();
    entry.set_address(self.physical_location);
    entry.set_present();
    self.set_directory_entry(1023, entry);
  }

  pub fn set_table_at_entry(&mut self, table_addr: PhysicalAddress, index: usize) {
    let mut entry = self.get_directory_entry(index);
    entry.set_address(table_addr);
    entry.set_present();
    self.set_directory_entry(index, entry);
  }

  fn get_first_entry_mut_ptr(&self) -> *mut PageTableEntry {
    self.virtual_location.as_usize() as *mut PageTableEntry
  }

  pub fn get_directory_entry(&self, index: usize) -> PageTableEntry {
    if index >= 1024 {
      panic!("PageDirectory index out of bounds");
    }
    unsafe {
      let entry_ptr = self.get_first_entry_mut_ptr().offset(index as isize);
      *entry_ptr
    }
  }

  pub fn set_directory_entry(&mut self, index: usize, entry: PageTableEntry) {
    if index >= 1024 {
      panic!("PageDirectory index out of bounds");
    }
    unsafe {
      let entry_ptr = self.get_first_entry_mut_ptr().offset(index as isize);
      *entry_ptr = entry;
    }
  }

  pub fn get_entry_for_address(&self, vaddr: VirtualAddress) -> PageTableEntry {
    let index = vaddr.as_usize() >> 22;
    self.get_directory_entry(index)
  }

  /**
   * Set CR3 to the physical address of this page directory, making it the
   * active mapping for memory paging.
   */
  pub fn make_active(&self) {
    set_cr3(self.physical_location.as_u32());
  }
}

pub fn get_current_directory() -> PageDirectory {
  // Fetch the physical address from CR3
  let paddr = PhysicalAddress::new(get_cr3() as usize);
  // The currently-active page directory is always mapped to the last 4KiB
  let vaddr = VirtualAddress::new(0xfffff000);

  PageDirectory {
    physical_location: paddr,
    virtual_location: vaddr,
  }
}