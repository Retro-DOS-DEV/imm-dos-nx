pub mod list_allocator;

extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

use super::address::VirtualAddress;
use super::physical;
use super::virt::page_directory::{CurrentPageDirectory, PermissionFlags};

struct Allocator {
  locked_allocator: Mutex<list_allocator::ListAllocator>,
}

impl Allocator {
  pub const fn new() -> Allocator {
    Allocator {
      locked_allocator: Mutex::new(list_allocator::ListAllocator::empty()),
    }
  }

  pub fn update_implementation(&self, start: VirtualAddress, size: usize) {
    let mut allocator = self.locked_allocator.lock();
    *allocator = list_allocator::ListAllocator::new(start, size);
  }
}

unsafe impl GlobalAlloc for Allocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let mut allocator = self.locked_allocator.lock();
    let mut ptr = allocator.alloc(layout);
    if ptr.is_null() {
      panic!("Heap expansion needs to be implemented");
      /*
      // Attempt to extend the heap
      let space_needed = layout.size();
      let new_size = expand_kernel_heap(space_needed);
      allocator.expand_size(new_size);
      // Try again with new free space
      ptr = allocator.alloc(layout);
      */
    }
    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
    let mut allocator = self.locked_allocator.lock();
    allocator.dealloc(ptr);
  }
}

pub const INITIAL_HEAP_SIZE: usize = 64;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

pub fn init_allocator(location: VirtualAddress, size: usize) {
  ALLOCATOR.update_implementation(location, size);
}

pub fn map_allocator(location: VirtualAddress, initial_frame_count: usize) {
  for i in 0..initial_frame_count {
    let heap_frame = physical::allocate_frame().unwrap();
    let heap_vaddr = location + (i * 0x1000);
    let current_mapping = CurrentPageDirectory::get();
    current_mapping.map(heap_frame, heap_vaddr, PermissionFlags::empty());
  }
}

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
  panic!("Alloc error: {:?}", layout)
}
