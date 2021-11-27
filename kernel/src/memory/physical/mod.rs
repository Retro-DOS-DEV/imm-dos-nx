//! Physical memory represents the actual RAM installed on the system. When a
//! process needs memory, a block of RAM will be allocated and made available
//! for mapping into virtual address space.
//! Each allocation is split into "Frames," each of which is the same size as a
//! virtual memory page. Frames can be allocated individually, or as a
//! single contiguous block.
//! To facilitate allocation and tracking of memory, the kernel maintains two
//! global objects: an Allocator, and a Reference Count. The allocator keeps
//! track of all available memory, allowing it to provide new memory areas on
//! demand. When blocks of memory are used more than once, such as when a forked
//! process maps a range of memory as Copy-on-Write, the reference count is
//! incremented for those frames. The Reference Count store only tracks frames
//! with more than one reference; otherwise, it would be unnecessarily large.
//! 
//! To ensure that all memory is properly used or freed, it is provided using an
//! AllocatedFrame object. Only an AllocatedFrame can be mapped to a virtual
//! page, and each AllocatedFrame must be mapped or freed before it is dropped.
//! An AllocatedFrame cannot be constructed for an arbitrary address. The only
//! ways to get an AllocatedFrame are by requesting newly allocated memory,
//! unmapping a previously-mapped frame, or duplicating an already-mapped frame.
//! 
//! When a frame is requested, the Allocator will mark it as used and construct
//! an AllocatedFrame object that can be mapped to memory.
//! When a page is unmapped, it will return an AllocatedFrame object for the
//! now-unused memory area. That AllocatedFrame can then be freed or re-mapped
//! to a new area.
//! An existing mapping can be duplicated, which increases the reference count
//! for that frame before constructing an AllocatedFrame.
//! When an AllocatedFrame is freed, it first checks if it has a reference count
//! greater than 1. If so, the ref count is decreased and the AllocatedFrame is
//! forgotten. If not, the memory is no longer in use, and the Allocator frees
//! the frame.

pub mod allocated_frame;
pub mod bios;
pub mod frame_bitmap;
pub mod frame_range;
pub mod frame_refcount;
pub mod frame;

use allocated_frame::AllocatedFrame;
use frame_bitmap::{BitmapError, FrameBitmap};
use frame_range::FrameRange;
use frame_refcount::FrameRefcount;
use spin::Mutex;
use super::address::{PhysicalAddress, VirtualAddress};

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

  let mut bitmap = FrameBitmap::at_location(
    VirtualAddress::new(location),
    limit >> 12,
  );
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

pub fn allocate_frame() -> Result<AllocatedFrame, BitmapError> {
  let frame = with_allocator(|alloc| {
    alloc
      .allocate_frames(1)
      .map(|range| range.get_first_frame())
  });
  match frame {
    Ok(f) => Ok(AllocatedFrame::new(f.get_address())),
    Err(e) => Err(e)
  }
}

pub fn allocate_range(range: FrameRange) -> Result<(), BitmapError> {
  with_allocator(|alloc| {
    alloc.allocate_range(range)
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

/*
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
*/

pub fn reference_frame_at_address(addr: PhysicalAddress) -> u8 {
  with_refcount(|refcount| {
    refcount.reference_frame_at_address(addr)
  })
}

