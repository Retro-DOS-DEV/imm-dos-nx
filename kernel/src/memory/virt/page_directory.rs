use crate::memory::physical::allocated_frame::AllocatedFrame;

use super::super::address::{PhysicalAddress, VirtualAddress};
use super::super::physical::frame::Frame;
use super::super::physical::{allocate_frame, free_frame};
use super::page_entry::PageTableEntry;
use super::page_table::PageTable;
use super::region::VirtualMemoryRegion;

pub struct PermissionFlags(u8);
impl PermissionFlags {
  pub const USER_ACCESS: u8 = 1;
  pub const WRITE_ACCESS: u8 = 2;
  pub const NO_RECLAIM: u8 = 4;

  pub fn new(flags: u8) -> PermissionFlags {
    PermissionFlags(flags)
  }

  pub fn empty() -> PermissionFlags {
    PermissionFlags(0)
  }

  pub fn as_u8(&self) -> u8 {
    self.0
  }
}

/// CurrentPageDirectory modifies the page dir of the active process. Nearly all
/// changes to page tables happen in the current process, so this is the only
/// struct we need.
pub struct CurrentPageDirectory {
}

impl CurrentPageDirectory {
  /// Get a new instance of the Current Page Directory
  pub fn get() -> CurrentPageDirectory {
    CurrentPageDirectory {}
  }

  /// Unmap the page containing a specific address. If a page table entry
  /// existed for that address, it is returned.
  pub fn unmap(&self, vaddr: VirtualAddress) -> Option<(AllocatedFrame, PageTableEntry)> {
    let dir_index = vaddr.get_page_directory_index();
    let table_index = vaddr.get_page_table_index();
    let directory = PageTable::at_address(VirtualAddress::new(0xfffff000));
    if !directory.get(dir_index).is_present() {
      return None;
    }
    let table = PageTable::at_address(VirtualAddress::new(
      0xffc00000 + 0x1000 * dir_index,
    ));
    let entry = *table.get(table_index);
    if !entry.is_present() {
      return None;
    }
    table.get_mut(table_index).zero();
    invalidate_page(vaddr);

    let frame = AllocatedFrame::new(entry.get_address());
    Some((frame, entry))
  }

  pub fn unmap_region(&self, region: VirtualMemoryRegion) {
    let mut page_start = VirtualAddress::new(region.get_starting_address_as_usize());
    while region.contains_address(page_start) {
      match self.unmap(page_start) {
        Some((frame, entry)) => {
          if entry.should_reclaim() {
            free_frame(frame).unwrap();
          }
        },
        None => (),
      }
      page_start = page_start.offset(0x1000);
    }
  }

  pub fn get_table_entry_for(&mut self, address: VirtualAddress) -> Option<&mut PageTableEntry> {
    let dir_index = address.get_page_directory_index();
    let table_index = address.get_page_table_index();
    let top_page = PageTable::at_address(VirtualAddress::new(0xfffff000));
    let entry = top_page.get_mut(dir_index);
    if !entry.is_present() {
      // table doesn't exist, so there is no entry for the address
      return None;
    }

    let table_address = VirtualAddress::new(0xffc00000 + (dir_index * 0x1000));
    let table = PageTable::at_address(table_address);
    Some(table.get_mut(table_index))
  }

  /// Map an allocated block of physical memory to a specific virtual address.
  /// This is the method that should always be used to create
  /// virtual-to-physical mappings in the page table.
  pub fn map(&self, allocated_frame: AllocatedFrame, vaddr: VirtualAddress, flags: PermissionFlags) {
    // Consume the AllocatedFrame without dropping it
    let frame = allocated_frame.to_frame();
    self.map_explicit(frame.get_address(), vaddr, flags)
  }

  /// Map an explicit address to a virtual location. This should only be used
  /// for peripherals that are on the memory bus, NOT for allocated RAM.
  pub fn map_explicit(&self, paddr: PhysicalAddress, vaddr: VirtualAddress, flags: PermissionFlags) {
    let dir_index = vaddr.get_page_directory_index();
    let table_index = vaddr.get_page_table_index();
    let top_page = PageTable::at_address(VirtualAddress::new(0xfffff000));
    // Address for the nested page table
    let table_address = VirtualAddress::new(0xffc00000 + (dir_index * 0x1000));

    let entry = top_page.get_mut(dir_index);
    if !entry.is_present() {
      // Create a page table
      let table_frame = allocate_frame().unwrap().to_frame();
      entry.set_address(table_frame.get_address());
      entry.set_present();
      if dir_index < 768 {
        entry.set_user_access();
        entry.set_write_access();
      }
      let table = PageTable::at_address(table_address);
      table.zero();
      table.get_mut(table_index).set_address(paddr);
      table.get_mut(table_index).set_present();
      if flags.as_u8() & PermissionFlags::WRITE_ACCESS != 0 {
        table.get_mut(table_index).set_write_access();
      }
      if flags.as_u8() & PermissionFlags::USER_ACCESS != 0 {
        table.get_mut(table_index).set_user_access();
      }
      if flags.as_u8() & PermissionFlags::NO_RECLAIM != 0 {
        table.get_mut(table_index).set_no_reclaim();
      }
    } else {
      let table = PageTable::at_address(table_address);
      let needs_invalidation = table.get(table_index).is_present();
      table.get_mut(table_index).set_address(paddr);
      table.get_mut(table_index).set_present();
      if flags.as_u8() & PermissionFlags::WRITE_ACCESS != 0 {
        table.get_mut(table_index).set_write_access();
      }
      if flags.as_u8() & PermissionFlags::USER_ACCESS != 0 {
        table.get_mut(table_index).set_user_access();
      }
      if flags.as_u8() & PermissionFlags::NO_RECLAIM != 0 {
        table.get_mut(table_index).set_no_reclaim();
      }
      if needs_invalidation {
        invalidate_page(vaddr);
      }
    }
  }

  pub fn get_physical_address(&self, vaddr: VirtualAddress) -> Option<PhysicalAddress> {
    // this could be supported, we just don't need it
    if !vaddr.is_page_aligned() {
      return None;
    }

    let dir_index = vaddr.get_page_directory_index();
    let table_index = vaddr.get_page_table_index();
    let top_page = PageTable::at_address(VirtualAddress::new(0xfffff000));
    let entry = top_page.get_mut(dir_index);
    if !entry.is_present() {
      return None;
    }
    // Address for the nested page table
    let table_address = VirtualAddress::new(0xffc00000 + (dir_index * 0x1000));
    let table = PageTable::at_address(table_address);
    let row = table.get(table_index);
    if row.is_present() {
      return Some(row.get_address());
    }
    Some(row.get_address())
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

pub fn get_current_page_address() -> VirtualAddress {
  VirtualAddress::new(0xfffff000)
}

/// Get the second-to-last entry in the self-mapped page directory. This is the
/// table containing most of the kernel stacks, and the scratch space for
/// unmapped pages.
pub fn get_last_page_table() -> &'static mut PageTable {
  PageTable::at_address(VirtualAddress::new(0xffffe000))
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
