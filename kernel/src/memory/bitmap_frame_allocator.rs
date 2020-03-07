use super::address::PhysicalAddress;
use super::frame_allocator::{FrameAllocator, FrameAllocatorError, FrameAllocatorResult};
use super::frame::Frame;
use super::map::{MapEntry, REGION_TYPE_FREE};

pub struct BitmapFrameAllocator {
  pub start: usize,
  pub length: usize,
}

#[inline]
fn get_index_and_offset_for_frame(frame: Frame) -> (usize, usize) {
  let addr = frame.get_address().as_usize() / 4096;
  let index = addr >> 3;
  let offset = addr & 7;
  (index, offset)
}

fn get_next_page_barrier(addr: usize) -> usize {
  if addr & 0xfffff000 == addr {
    addr
  } else {
    (addr & 0xfffff000) + 0x1000
  }
}

impl BitmapFrameAllocator {
  pub const fn new() -> BitmapFrameAllocator {
    BitmapFrameAllocator {
      start: 0,
      length: 0,
    }
  }

  pub unsafe fn init(&mut self, map: &[MapEntry], kernel_start: PhysicalAddress, kernel_end: PhysicalAddress) {
    let memory_start = map[0].base as usize;
    let last_entry = &map[map.len() - 1];
    let memory_end = (last_entry.base + last_entry.length) as usize;
    let frame_count = (memory_end - memory_start) >> 12;
    self.start = get_next_page_barrier(kernel_end.as_usize());
    self.length = frame_count >> 3;
    // reset all frames before we start making them as available
    self.reset_table();

    // Iterate through the memory map, marking only full-sized frames as free
    // If a memory range ends within a frame, and not on a 4KiB barrier, we
    // can assume that at least part of that frame is unavailable, and should
    // not be used as system memory.
    for entry in map.iter() {
      let entry_start = entry.base as usize;
      let entry_end = (entry.base + entry.length - 1) as usize;

      match entry.region_type {
        REGION_TYPE_FREE => {
          let mut addr = get_next_page_barrier(entry_start);
          while addr <= entry_end {
            self.mark_unallocated(Frame::new(addr));
            addr += 0x1000;
          }
        },
        _ => (),
      }
    }

    // Mark the kernel as occupied
    let first_kernel_frame = Frame::containing_address(kernel_start);
    let last_kernel_frame = Frame::containing_address(kernel_end);
    let mut frame = first_kernel_frame;
    while frame.get_address() <= last_kernel_frame.get_address() {
      self.mark_allocated(frame);
      frame = frame.next_frame();
    }
    // Mark this frame table as occupied
    let mut own_size_in_frames = self.length >> 12;
    if self.length & 0xfff != 0 {
      own_size_in_frames += 1;
    }
    let last_bitmap_frame = Frame::new(last_kernel_frame.get_address().as_usize() + 4096 * own_size_in_frames);
    frame = last_kernel_frame.next_frame();
    while frame.get_address() <= last_bitmap_frame.get_address() {
      self.mark_allocated(frame);
      frame = frame.next_frame();
    }
  }

  unsafe fn get_starting_ptr(&self) -> *mut u8 {
    self.start as *mut u8
  }

  /**
   * Mark each entry as unavailable. Since system memory can have "holes," it's
   * safer to explicitly mark available frames than to assume they are free and
   * mark the occupied ones.
   */
  unsafe fn reset_table(&mut self) {
    let mut index = 0;
    let mut ptr_32 = self.start as *mut u32;
    while index <= self.length - 4 {
      *ptr_32 = 0xffffffff;
      ptr_32 = ptr_32.offset(1);
      index += 4;
    }
    let mut ptr_8 = ptr_32 as *mut u8;
    while index < self.length {
      *ptr_8 = 0xff;
      ptr_8 = ptr_8.offset(1);
      index += 1;
    }
  }

  unsafe fn mark_allocated(&mut self, frame: Frame) {
    let (index, offset) = get_index_and_offset_for_frame(frame);
    let bitmap = self.get_starting_ptr().offset(index as isize);
    *bitmap |= 1 << offset;
  }

  unsafe fn mark_unallocated(&mut self, frame: Frame) {
    let (index, offset) = get_index_and_offset_for_frame(frame);
    let bitmap = self.get_starting_ptr().offset(index as isize);
    *bitmap &= !(1 << offset);
  }

  pub unsafe fn find_next_free(&self) -> FrameAllocatorResult {
    // Start at 32, so we can preserve lowmem for something else
    for i in 32..self.length {
      let bitmap = self.get_starting_ptr().offset(i as isize);
      if *bitmap & 0xff != 0xff {
        // has an unset bit
        let start = i << 3;
        for j in 0..8 {
          if *bitmap & (1 << j) == 0 {
            return Ok(Frame::new((start + j) * 4096));
          }
        }
      }
    }

    Err(FrameAllocatorError::OutOfMemory)
  }
}

impl FrameAllocator for BitmapFrameAllocator {
  fn allocate(&mut self) -> FrameAllocatorResult {
    let next_free = unsafe {
      self.find_next_free()
    };
    match next_free {
      Ok(frame) => {
        unsafe {
          self.mark_allocated(frame);
        }
        Ok(frame)
      },
      Err(e) => Err(e),
    }
  }

  fn is_free(&self, frame: Frame) -> bool {
    let (index, offset) = get_index_and_offset_for_frame(frame);
    unsafe {
      let bitmap = self.get_starting_ptr().offset(index as isize);
      *bitmap & (1 << offset) == 0
    }
  }

  fn release(&mut self, frame: Frame) {
    unsafe {
      self.mark_unallocated(frame);
    }
  }

  fn count_frames(&self) -> usize {
    self.length * 8
  }

  fn count_free_frames(&self) -> usize {
    let mut index = 0;
    let mut count = 0;
    unsafe {
      let mut ptr_8 = self.get_starting_ptr();
      while index < self.length {
        let mut bitmap = *ptr_8;
        if bitmap != 0xff {
          if bitmap == 0 {
            count += 8;
          } else {
            // can't use POPCOUNT without SSE
            count += 8;
            while bitmap != 0 {
              count -= (bitmap & 1) as usize;
              bitmap = bitmap >> 1;
            }
          }
        }
        index += 1;
        ptr_8 = ptr_8.offset(1);
      }
    }
    count
  }
}
