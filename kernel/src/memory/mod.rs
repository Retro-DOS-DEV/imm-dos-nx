pub mod address;
pub mod bitmap_frame_allocator;
pub mod frame_allocator;
pub mod frame;
pub mod map;

use frame_allocator::FrameAllocator;
use bitmap_frame_allocator::BitmapFrameAllocator;

pub static mut FRAME_ALLOCATOR: BitmapFrameAllocator = BitmapFrameAllocator::new();

pub fn init(kernel_start: address::PhysicalAddress, kernel_end: address::PhysicalAddress) {
  unsafe {
    let mem_map = map::load_entries_at_address(0x1000);

    FRAME_ALLOCATOR.init(mem_map, kernel_start, kernel_end);
  }
}

pub fn count_frames() -> usize {
  unsafe {
    FRAME_ALLOCATOR.count_frames()
  }
}

pub fn count_free_frames() -> usize {
  unsafe {
    FRAME_ALLOCATOR.count_free_frames()
  }
}

pub fn allocate_physical_frame() -> frame_allocator::FrameAllocatorResult {
  unsafe {
    FRAME_ALLOCATOR.allocate()
  }
}
