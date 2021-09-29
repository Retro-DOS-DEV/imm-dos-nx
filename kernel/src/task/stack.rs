//! Each process has its own Kernel Stack to handle operations whenever it drops
//! into a syscall. Allocation and de-allocation of these stacks is handled here
//! to guarantee that no two processes accidentally claim the same stack, and to
//! ensure freed stacks can be re-used.
//! Earlier versions of the kernel used page tables to give each process a
//! unique stack at the same address location, but this created some challenges
//! when trying to manipulate an unmapped process's stack. Some quick math
//! showed that even with a generous stack size of four pages (including a guard
//! page), there would be plenty of space for tens of thousands of processes in
//! virtual memory. With that realization, we can share all stacks in the same
//! address space and simplify kernel memory mapping. It's also simple to
//! manipulate the stack of any process, making forking easier.
//! A large area of memory (stack size * max process count) is reserved just
//! below the top page directory (0xffc00000). When a stack is allocated, the
//! corresponding pages will be mapped into all kernel page tables, and a
//! pointer to the allocated stack pages will be attached to the process.
//! These stacks could be allocated on the heap, but it wouldn't be possible to
//! include a guard page to keep one stack from clobbering other heap objects.
//! This method makes allocation and deallocation a bit hackier, but it lets us
//! lean on memory management hardware to prevent any stack overflows.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use core::ops::Range;
use crate::memory::address::{PhysicalAddress, VirtualAddress};
use spin::RwLock;

pub static ALLOCATED_KERNEL_STACKS: RwLock<Vec<u8>> = RwLock::new(Vec::new());

pub const STACKS_TOP: usize = 0xffc00000;
pub const STACK_SIZE: usize = 0x4000;
pub const STACK_GUARD_SIZE: usize = 0x1000;
pub const STACK_SIZE_IN_PAGES: usize = STACK_SIZE / 0x1000;
pub const FIRST_STACK_TOP_PAGE: usize = STACKS_TOP - STACK_SIZE - 0x1000;
pub const MAX_STACK_AREA_SIZE: usize = STACK_SIZE * (super::process::MAX_PROCESS_COUNT + 1);

pub fn temporary_paging_range() -> Range<VirtualAddress> {
  VirtualAddress::new(STACKS_TOP - STACK_SIZE)..VirtualAddress::new(STACKS_TOP)
}

/// Initialize the stack allocation bitmap. The first "stack" area is actually
/// used for temporary paging, and should not be allocated. The second stack is
/// for the bootstrapping process, and will already be mapped.
pub fn allocate_initial_stacks() {
  let mut alloc_map = ALLOCATED_KERNEL_STACKS.write();
  alloc_map.push(3);
}

/// Utility to help generate a pointer from a specific stack index. This should
/// only be used to set up the initial bootstrap process.
pub fn stack_box_from_index(index: usize) -> Box<[u8]> {
  let ptr = STACKS_TOP - ((index + 1) * STACK_SIZE);
  unsafe {
    Vec::from_raw_parts(ptr as *mut u8, STACK_SIZE, STACK_SIZE).into_boxed_slice()
  }
}

/// Find a free stack area, mark it as used, and return a Box referencing the
/// new stack space. Each time a process is created, this should be used to
/// give it a kernel stack.
pub fn allocate_stack() -> Box<[u8]> {
  let index = find_free_space(&ALLOCATED_KERNEL_STACKS);
  stack_box_from_index(index)
}

fn find_free_space(stacks: &RwLock<Vec<u8>>) -> usize {
  let mut alloc_map = stacks.write();
  for (index, map) in alloc_map.iter_mut().enumerate() {
    let mut stack_index = index * 8;
    if *map != 0xff {
      let mut inv = !*map;
      let mut mask = 1;
      while inv != 0 {
        if inv & 1 != 0 {
          *map |= mask;
          return stack_index;
        }
        inv >>= 1;
        mask <<= 1;
        stack_index += 1;
      }
    }
  }
  let stack_index = alloc_map.len() * 8;
  alloc_map.push(1);
  stack_index
}

pub fn free_stack(stack: Box<[u8]>) {
  let box_ptr = Box::into_raw(stack);
  let location = box_ptr as *mut u8 as  usize;
  let offset = (STACKS_TOP - location) / STACK_SIZE;
  free_index(&ALLOCATED_KERNEL_STACKS, offset);
}

fn free_index(stacks: &RwLock<Vec<u8>>, index: usize) {
  let mut alloc_map = stacks.write();
  let byte_index = index / 8;
  let local_index = index & 7;
  match alloc_map.get_mut(byte_index) {
    Some(map) => {
      let mask = 1 << local_index;
      *map &= !mask;
    },
    None => (),
  }
}

pub fn duplicate_stack(from: &Box<[u8]>, to: &mut Box<[u8]>) {
  to[0x1000..].copy_from_slice(&from[0x1000..]);
}

static SCRATCH_PAGES: AtomicU32 = AtomicU32::new(0);

/// We use the top "stack" as scratch space for editing pages that aren't mapped
/// into the current memory space. This is typically used for creating page
/// tables for other processes.
/// To use these pages, we allocate UnmappedPage structs which mark a scratch
/// page as occupied, and release it when dropped.
pub struct UnmappedPage {
  address: PhysicalAddress,
  scratch_index: usize,
}

impl UnmappedPage {
  pub fn map(address: PhysicalAddress) -> UnmappedPage {
    let mut mask: u32 = 1;
    for i in 0..STACK_SIZE_IN_PAGES {
      let prev = SCRATCH_PAGES.fetch_or(mask, Ordering::SeqCst);
      if prev & mask == 0 {
        #[cfg(not(test))]
        {
          // write to the stack
          use crate::memory::virt;
          let top_table = virt::page_directory::get_last_page_table();
          // The highest entries in top_table are the scratch pages.
          // SCRATCH_PAGES[0] refers to entry 1023, [1] refers to entry 1022,
          // and so on.
          let entry = 1023 - i;
          top_table.get_mut(entry).set_address(address);
          top_table.get_mut(entry).set_present();
          let virtual_addr = VirtualAddress::new(STACKS_TOP - ((i + 1) * 0x1000));
          virt::page_directory::invalidate_page(virtual_addr);
        }
        return UnmappedPage {
          address,
          scratch_index: i,
        };
      }
      mask <<= 1;
    }
    panic!("There are no free unmapped page scratch pages");
  }

  pub fn virtual_address(&self) -> VirtualAddress {
    VirtualAddress::new(STACKS_TOP - ((self.scratch_index + 1) * 0x1000))
  }
}

impl Drop for UnmappedPage {
  fn drop(&mut self) {
    let mask = !(1 << self.scratch_index);
    // Mark the page as unoccupied again
    SCRATCH_PAGES.fetch_and(mask, Ordering::SeqCst);
  }
}

#[cfg(test)]
mod tests {
  use super::{
    Ordering,
    PhysicalAddress,
    RwLock,
    Vec,
    VirtualAddress,
    find_free_space,
    free_index,
    SCRATCH_PAGES,
    STACK_SIZE_IN_PAGES,
    UnmappedPage,
  };

  #[test]
  fn create_stack() {
    let stacks = RwLock::new(Vec::new());
    assert_eq!(find_free_space(&stacks), 0);
    assert_eq!(find_free_space(&stacks), 1);
    assert_eq!(find_free_space(&stacks), 2);
    assert_eq!(find_free_space(&stacks), 3);
    assert_eq!(find_free_space(&stacks), 4);
    assert_eq!(find_free_space(&stacks), 5);
    assert_eq!(find_free_space(&stacks), 6);
    assert_eq!(find_free_space(&stacks), 7);
    *(stacks.write().get_mut(0).unwrap()) = 0xbf;
    assert_eq!(find_free_space(&stacks), 6);
    assert_eq!(find_free_space(&stacks), 8);
  }

  #[test]
  fn free_allocated_stack() {
    let stacks = RwLock::new(Vec::new());
    assert_eq!(find_free_space(&stacks), 0);
    assert_eq!(find_free_space(&stacks), 1);
    assert_eq!(find_free_space(&stacks), 2);
    assert_eq!(find_free_space(&stacks), 3);
    free_index(&stacks, 1);
    assert_eq!(find_free_space(&stacks), 1);
  }

  #[test]
  fn unmapped_page() {
    // reset the map
    SCRATCH_PAGES.swap(0, Ordering::SeqCst);

    let p0 = UnmappedPage::map(PhysicalAddress::new(0x3000));
    assert_eq!(SCRATCH_PAGES.load(Ordering::SeqCst), 1);
    assert_eq!(p0.virtual_address(), VirtualAddress::new(0xffbff000));
    {
      let p1 = UnmappedPage::map(PhysicalAddress::new(0x5000));
      assert_eq!(SCRATCH_PAGES.load(Ordering::SeqCst), 3);
      assert_eq!(p1.virtual_address(), VirtualAddress::new(0xffbfe000));
      let p2 = UnmappedPage::map(PhysicalAddress::new(0x6000));
      assert_eq!(SCRATCH_PAGES.load(Ordering::SeqCst), 7);
      assert_eq!(p2.virtual_address(), VirtualAddress::new(0xffbfd000));
    }
    assert_eq!(SCRATCH_PAGES.load(Ordering::SeqCst), 1);
    {
      let p1 = UnmappedPage::map(PhysicalAddress::new(0x5000));
      assert_eq!(SCRATCH_PAGES.load(Ordering::SeqCst), 3);
      assert_eq!(p1.virtual_address(), VirtualAddress::new(0xffbfe000));
      let p2 = UnmappedPage::map(PhysicalAddress::new(0x6000));
      assert_eq!(SCRATCH_PAGES.load(Ordering::SeqCst), 7);
      assert_eq!(p2.virtual_address(), VirtualAddress::new(0xffbfd000));
    }
  }
}
