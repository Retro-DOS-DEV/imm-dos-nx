use super::super::address::{PhysicalAddress, VirtualAddress};
use super::super::physical::frame::Frame;
use super::super::physical::allocate_frame;
use super::page_table::PageTable;

pub trait PageDirectory {
  fn map(&self, frame: Frame, vaddr: VirtualAddress /* needs flags */);
}

pub struct CurrentPageDirectory {
}

impl CurrentPageDirectory {
  pub fn get() -> CurrentPageDirectory {
    CurrentPageDirectory {}
  }
}

impl PageDirectory for CurrentPageDirectory {
  fn map(&self, frame: Frame, vaddr: VirtualAddress) {
    let paddr = frame.get_address();
    let dir_index = vaddr.get_page_directory_index();
    let table_index = vaddr.get_page_table_index();
    let top_page = PageTable::at_address(VirtualAddress::new(0xfffff000));
    // Address for the nested page table
    let table_address = VirtualAddress::new(0xffc00000 + (dir_index * 0x1000));

    let entry = top_page.get_mut(dir_index);
    if !entry.is_present() {
      // Create a page table
      let table_frame = allocate_frame().unwrap();
      entry.set_address(table_frame.get_address());
      entry.set_present();
      let table = PageTable::at_address(table_address);
      table.zero();
      table.get_mut(table_index).set_address(paddr);
      table.get_mut(table_index).set_present();
    } else {
      let table = PageTable::at_address(table_address);
      let needs_invalidation = table.get(table_index).is_present();
      table.get_mut(table_index).set_address(paddr);
      table.get_mut(table_index).set_present();
      if needs_invalidation {
        invalidate_page(vaddr);
      }
    }
  }
}

#[cfg(not(test))]
pub fn set_current_pagedir(addr: PhysicalAddress) {
  crate::x86::registers::set_cr3(addr.as_u32());
}

#[cfg(not(test))]
pub fn get_current_pagedir() -> PhysicalAddress {
  let cr3 = crate::x86::registers::get_cr3();
  PhysicalAddress::new(cr3 as usize)
}

#[cfg(not(test))]
pub fn invalidate_page(addr: VirtualAddress) {
  unsafe {
    llvm_asm!("invlpg ($0)" : : "r"(addr.as_u32()) : "memory");
  }
}

pub fn get_temporary_page_address() -> VirtualAddress {
  VirtualAddress::new(0xffbff000)
}

pub fn get_current_page_address() -> VirtualAddress {
  VirtualAddress::new(0xfffff000)
}

pub fn map_frame_to_temporary_page(frame: Frame) {
  // The temporary page is located in the last slot of the second-to-last page
  // table. Assuming the current pagedir is mapped to its own last slot, this
  // means the entry we want to edit is the one just prior to the last 4KiB of
  // virtual memory.
  let last_table = PageTable::at_address(VirtualAddress::new(0xffffe000));
  last_table.get_mut(1023).set_address(frame.get_address());
  last_table.get_mut(1023).set_present();
  invalidate_page(get_temporary_page_address());
}

// For testing:
#[cfg(test)]
static mut MOCK_CR3: u32 = 0;

#[cfg(test)]
pub fn set_current_pagedir(addr: PhysicalAddress) {
  unsafe {
    MOCK_CR3 = addr.as_u32();
  }
}

#[cfg(test)]
pub fn get_current_pagedir() -> PhysicalAddress {
  let cr3 = unsafe { MOCK_CR3 };
  PhysicalAddress::new(cr3 as usize)
}

#[cfg(test)]
pub fn invalidate_page(_addr: VirtualAddress) {
  // no-op in tests
}
