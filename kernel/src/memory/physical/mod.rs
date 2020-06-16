pub mod bios;
pub mod frame_bitmap;
pub mod frame_range;
pub mod frame;

use frame_bitmap::FrameBitmap;
use frame_range::FrameRange;

pub fn build_allocator_at_address(addr: usize, frame_count: usize) -> FrameBitmap {
  assert!(addr & 0xfff == 0, "Allocator must start on a page boundary");
  let mut bitmap = FrameBitmap::at_location(addr, frame_count);
  let size_in_frames = bitmap.size_in_frames();
  let own_range = FrameRange::new(addr, addr + size_in_frames * 0x1000);
  bitmap.allocate_range(own_range).unwrap();
  bitmap
}
