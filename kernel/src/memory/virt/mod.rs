pub mod page_directory;
pub mod page_entry;
pub mod page_table;
pub mod region;

use page_directory::{CurrentPageDirectory, PageDirectory};
use page_table::{PageTable, PageTableReference};
use super::address::{PhysicalAddress, VirtualAddress};
use super::physical;

#[cfg(not(test))]
use crate::x86;

/// Create the initial Page Directory, before paging has been enabled
pub fn create_initial_pagedir() -> PageTableReference {
  let dir_frame = physical::allocate_frame().unwrap().to_frame();
  unsafe { dir_frame.zero_memory(); }
  let dir_address = dir_frame.get_address();
  let dir = PageTable::at_address(VirtualAddress::new(dir_address.as_usize()));
  // Point second-to-last directory entry to an empty pagetable.
  // The last few entries of that pagetable are used as temporary frame editing
  // space. Below that are the kernel stacks for processes, starting with the
  // stack for the bootstrapping process.
  let last_table_frame = physical::allocate_frame().unwrap().to_frame();
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

/// Before enabling paging, we need to make sure all kernel internals are
/// mapped at the appropriate addresses, or the entire thing will crash.
pub fn map_kernel(directory_ref: PageTableReference, bounds: &KernelDataBounds) {
  // Mark the kernel's occupied frames as allocated
  let kernel_range = physical::frame_range::FrameRange::new(
    bounds.ro_start.as_usize(),
    bounds.rw_end.as_usize() - bounds.ro_start.as_usize() - 1,
  );
  physical::allocate_range(kernel_range).unwrap();
  // For now, just identity-map the first 4MiB
  let table_zero_frame = physical::allocate_frame().unwrap().to_frame();
  unsafe { table_zero_frame.zero_memory() };
  let dir = PageTable::at_address(VirtualAddress::new(directory_ref.get_address().as_usize()));
  dir.get_mut(0).set_address(table_zero_frame.get_address());
  dir.get_mut(0).set_present();
  // Used for access to testing methods in userspace. Can be removed when those
  // no longer exist.
  dir.get_mut(0).set_user_access();

  
  let table_zero = PageTable::at_address(VirtualAddress::new(table_zero_frame.get_address().as_usize()));
  for index in 0..1024 {
    table_zero.get_mut(index).set_address(PhysicalAddress::new(0x1000 * index));
    table_zero.get_mut(index).set_present();
    table_zero.get_mut(index).set_user_access();
  }
  // Also, map it to highmem at 0xc0000000
  dir.get_mut(0x300).set_address(table_zero_frame.get_address());
  dir.get_mut(0x300).set_present();
  // Finally, move the stack to the top of memory, just below the temp page
  let last_page_addr = dir.get(1022).get_address();
  let last_page = PageTable::at_address(VirtualAddress::new(last_page_addr.as_usize()));
  let temp_area_page_count = crate::task::stack::STACK_SIZE / 0x1000;
  let stack_top_page_index = 1023 - temp_area_page_count;
  last_page.get_mut(stack_top_page_index).set_address(bounds.stack_start);
  last_page.get_mut(stack_top_page_index).set_present();
  // Each process expects its stack to be fully mapped (including one invalid
  // guard page) before it runs. If a page fault occurs in a kernel stack, there
  // will be nowhere to push the exception details and the processor will
  // double-fault.
  // The guard page will remain unmapped. When a page fault occurs, the
  // exception handler can check the alignment of the faulted page and determine
  // if it represents a stack guard.
  let mut extra_stack_pages = (crate::task::stack::STACK_SIZE / 0x1000) - 2;
  while extra_stack_pages > 0 {
    let stack_frame = physical::allocate_frame().unwrap().to_frame();
    let index = stack_top_page_index - extra_stack_pages;
    last_page.get_mut(index).set_address(stack_frame.get_address());
    last_page.get_mut(index).set_present();
    extra_stack_pages -= 1;
  }
}

/// Create mappings for a new kernel stack. Each time a process is created, a
/// stack is allocated. All pages except for the guard at the bottom need frames
/// allocated in physical memory, and mappings to that physical memory need to
/// be created.
pub fn map_kernel_stack(stack_range: core::ops::Range<VirtualAddress>) {
  if stack_range.start < VirtualAddress::new(0xff800000) {
    panic!("Creating new stack page directories isn't currently supported");
  }
  let mut stack_pages = (crate::task::stack::STACK_SIZE / 0x1000) - 1;
  while stack_pages > 0 {
    let stack_frame = physical::allocate_frame().unwrap();
    let address = stack_range.end - (0x1000 * stack_pages);
    CurrentPageDirectory::get().map(stack_frame.to_frame(), address, page_directory::PermissionFlags::empty());
    stack_pages -= 1;
  }
}

pub fn enable_paging() {
  #[cfg(not(test))]
  {
    x86::registers::enable_paging();
  }
}
