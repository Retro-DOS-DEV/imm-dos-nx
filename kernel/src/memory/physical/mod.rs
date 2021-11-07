pub mod bios;
pub mod frame_bitmap;
pub mod frame_range;
pub mod frame_refcount;
pub mod frame;

use frame_bitmap::{BitmapError, FrameBitmap};
use frame_range::FrameRange;
use frame_refcount::FrameRefcount;
use spin::Mutex;
use super::address::PhysicalAddress;

static mut ALLOCATOR: Option<Mutex<FrameBitmap>> = None;
static mut REF_COUNT: Option<Mutex<FrameRefcount>> = None;

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

  // Mark the first frame as allocated, we may need the BIOS memory area
  bitmap.allocate_range(FrameRange::new(0, 0x1000)).unwrap();

  unsafe {
    ALLOCATOR = Some(Mutex::new(bitmap));
  }
}

pub fn move_allocator_reference_to_highmem() {
  with_allocator(|alloc| {
    alloc.move_to_highmem()
  });
}

pub fn init_refcount() {
  let frame_count = with_allocator(|alloc| {
    alloc.get_frame_count()
  });
  unsafe {
    REF_COUNT = Some(Mutex::new(FrameRefcount::new(frame_count)));
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

pub fn with_refcount<F, T>(f: F) -> T where
  F: Fn(&mut FrameRefcount) -> T {
  // Safe because the REF_COUNT will only be set once, synchronously
  match unsafe { &REF_COUNT } {
    Some(r) => {
      let mut refcount = r.lock();
      f(&mut refcount)
    },
    None => panic!("Reference counter was not initialized"),
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

pub fn get_frame_for_copy_on_write(prev: PhysicalAddress) -> Result<frame::Frame, BitmapError> {
  with_refcount(|refcount| {
    let current_count = refcount.current_count_at_address(prev);
    if current_count > 1 {
      // The frame is referenced multiple times. In order to write to it, we
      // need to copy to a new frame.
      let new_frame = allocate_frame();
      match new_frame {
        Ok(f) => {
          refcount.reference_frame(f);
          refcount.release_frame_at_address(prev);
          Ok(f)
        },
        Err(e) => Err(e),
      }
    } else {
      Ok(frame::Frame::new(prev.as_usize()))
    }
  })
}

pub fn reference_frame_at_address(addr: PhysicalAddress) -> u8 {
  with_refcount(|refcount| {
    refcount.reference_frame_at_address(addr)
  })
}

