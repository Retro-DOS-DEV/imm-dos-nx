pub mod bios;
pub mod frame_bitmap;
pub mod frame_range;
pub mod frame;

use frame_bitmap::{BitmapError, FrameBitmap};
use frame_range::FrameRange;
use spin::Mutex;

static mut ALLOCATOR: Option<Mutex<FrameBitmap>> = None;

pub fn init_allocator(location: usize, memory_map_addr: usize) {
  assert!(location & 0xfff == 0, "Allocator must start on a page boundary");
  let mut limit = 0;
  let memory_map = unsafe { bios::load_entries_at_address(memory_map_addr) };
  // memory map is not guaranteed to be in order
  for entry in memory_map.iter() {
    let end = (entry.base + entry.length) as usize;
    if end > limit {
      limit = end;
    }
  }

  let mut bitmap = FrameBitmap::at_location(location, limit >> 12);
  bitmap.initialize_from_memory_map(&memory_map).unwrap();

  let size_in_frames = bitmap.size_in_frames();
  let own_range = FrameRange::new(location, size_in_frames * 0x1000);
  bitmap.allocate_range(own_range).unwrap();

  unsafe {
    ALLOCATOR = Some(Mutex::new(bitmap));
  }
}

pub fn with_allocator<F, T>(f: F) -> T where
  F: Fn(&mut FrameBitmap) -> T {
  // Safe because the ALLOCATOR will only be set once, synchronously
  match unsafe { &ALLOCATOR } {
    Some(m) => {
      let mut alloc = m.lock();
      f(&mut alloc)
    },
    None => panic!("Physical frame allocator was not initialized"),
  }
}

pub fn allocate_frames(count: usize) -> Result<FrameRange, BitmapError> {
  with_allocator(|alloc| {
    alloc.allocate_frames(count)
  })
}

pub fn allocate_frame() -> Result<frame::Frame, BitmapError> {
  let frame = allocate_frames(1);
  match frame {
    Ok(f) => Ok(f.get_first_frame()),
    Err(e) => Err(e)
  }
}

pub fn allocate_range(range: FrameRange) -> Result<(), BitmapError> {
  with_allocator(|alloc| {
    alloc.allocate_range(range)
  })
}

pub fn free_range(range: FrameRange) -> Result<(), BitmapError> {
  with_allocator(|alloc| {
    alloc.free_range(range)
  })
}

pub fn get_frame_count() -> usize {
  with_allocator(|alloc| {
    alloc.get_frame_count()
  })
}

pub fn get_free_frame_count() -> usize {
  with_allocator(|alloc| {
    alloc.get_free_frame_count()
  })
}
