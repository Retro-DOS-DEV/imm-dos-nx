pub mod list_allocator;

extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

use super::address::VirtualAddress;

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
    allocator.alloc(layout)
  }

  unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
    let mut allocator = self.locked_allocator.lock();
    allocator.dealloc(ptr);
  }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

pub fn init_allocator(location: VirtualAddress, size: usize) {
  ALLOCATOR.update_implementation(location, size);
}

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
  panic!("Alloc error: {:?}", layout)
}
