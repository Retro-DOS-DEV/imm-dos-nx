pub mod page_directory;
pub mod page_entry;
pub mod page_table;
pub mod region;

use page_table::{PageTable, PageTableReference};
use super::address::{PhysicalAddress, VirtualAddress};
use super::physical::{self, frame::Frame};

#[cfg(not(test))]
use crate::x86;

pub const STACK_START: VirtualAddress = VirtualAddress::new(0xffbfd000);

/**
 * Create the initial Page Directory, before paging has been enabled.
 */
pub fn create_initial_pagedir() -> PageTableReference {
  let dir_frame = physical::allocate_frame().unwrap();
  unsafe { dir_frame.zero_memory(); }
  let dir_address = dir_frame.get_address();
  let dir = PageTable::at_address(VirtualAddress::new(dir_address.as_usize()));
  // Point second-to-last entry to an empty pagetable
  // The last entry of this pagetable is used as temporary frame editing space
  let last_table_frame = physical::allocate_frame().unwrap();
  unsafe { last_table_frame.zero_memory(); }
  let last_table_address = last_table_frame.get_address();
  dir.get_mut(1022).set_address(last_table_address);
  dir.get_mut(1022).set_present();
  // Point last entry to self, so it and its tables are always editable
  dir.get_mut(1023).set_address(dir_address);
  dir.get_mut(1023).set_present();

  PageTableReference::new(dir_address)
}

pub struct KernelDataBounds {
  pub ro_start: PhysicalAddress,

  pub rw_end: PhysicalAddress,
  pub stack_start: PhysicalAddress,
}

pub fn map_kernel(directory_ref: PageTableReference, bounds: KernelDataBounds) {
  // Mark the kernel's occupied frames as allocated
  let kernel_range = physical::frame_range::FrameRange::new(
    bounds.ro_start.as_usize(),
    bounds.rw_end.as_usize() - bounds.ro_start.as_usize() - 1,
  );
  physical::allocate_range(kernel_range).unwrap();
  // For now, just identity-map the first 4MiB
  let table_zero_frame = physical::allocate_frame().unwrap();
  unsafe { table_zero_frame.zero_memory() };
  let dir = PageTable::at_address(VirtualAddress::new(directory_ref.get_address().as_usize()));
  dir.get_mut(0).set_address(table_zero_frame.get_address());
  dir.get_mut(0).set_present();
  let table_zero = PageTable::at_address(VirtualAddress::new(table_zero_frame.get_address().as_usize()));
  for index in 0..1024 {
    table_zero.get_mut(index).set_address(PhysicalAddress::new(0x1000 * index));
    table_zero.get_mut(index).set_present();
  }
  // Also, map it to highmem at 0xc0000000
  dir.get_mut(0x300).set_address(table_zero_frame.get_address());
  dir.get_mut(0x300).set_present();
  // Finally, move the stack to the top of memory, just below the temp page
  let last_page_addr = dir.get(1022).get_address();
  let last_page = PageTable::at_address(VirtualAddress::new(last_page_addr.as_usize()));
  last_page.get_mut(1022).set_address(bounds.stack_start);
  last_page.get_mut(1022).set_present();
}

pub fn enable_paging() {
  #[cfg(not(test))]
  {
    x86::registers::enable_paging();
  }
}
