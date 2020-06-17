use core::slice;
use super::bios;
use super::frame_range::FrameRange;

pub struct FrameBitmap {
  frame_count: usize,
  map: &'static mut [u8],
}

impl FrameBitmap {
  pub fn at_location(start: usize, frame_count: usize) -> FrameBitmap {
    let mut byte_size = frame_count >> 3;
    if frame_count & 0x7 != 0 {
      byte_size += 1;
    }
    let data = start as *mut u8;
    FrameBitmap {
      frame_count,
      map: unsafe { slice::from_raw_parts_mut(data, byte_size) },
    }
  }

  /**
   * Reset the table to being entirely allocated; should be used whenever a new
   * bitmap is initialized.
   * This simplifies any logic that deals with a number of frames not divisible
   * by 8 -- the last bits of the last marker byte will be seen as unavailable.
   * Additionally, it is necessary for initialization from the BIOS memory map.
   * The map provided by BIOS may have holes. Rather than compute the space
   * between rows in the map, it is easier to just de-allocate the known free
   * spaces.
   */
  pub fn reset(&mut self) {
    let mut frame = 0;
    while frame < self.frame_count {
      let byte_index = frame >> 3;
      self.map[byte_index] = 0xff;
      frame += 8;
    }
  }

  /**
   * Given a BIOS-generated memory map, iterate through that map and de-allocate
   * all known free ranges. If the process succeeds, the bitmap will accurately
   * reflect all memory areas available for allocation.
   */
  pub fn initialize_from_memory_map(&mut self, map: &[bios::MapEntry]) -> Result<(), BitmapError> {
    self.reset();
    for entry in map.iter() {
      if entry.region_type == bios::REGION_TYPE_FREE {
        let range = FrameRange::new(
          (entry.base & 0xffffffff) as usize,
          (entry.length & 0xffffffff) as usize,
        );
        match self.free_range(range) {
          Err(e) => return Err(e),
          _ => (),
        }
      }
    }
    Ok(())
  }

  /**
   * How big is this table, in 4096-byte frames? Useful for allocating itself.
   */
  pub fn size_in_frames(&self) -> usize {
    let byte_size = self.frame_count >> 3;
    let frame_count = byte_size >> 12;
    // Round up as necessary
    if byte_size & 0xfff == 0 {
      frame_count
    } else {
      frame_count + 1
    }
  }

  pub fn contains_frame_index(&self, index: usize) -> bool {
    index < self.frame_count
  }

  pub fn contains_range(&self, range: FrameRange) -> bool {
    self.contains_frame_index(range.get_last_frame_index())
  }

  pub fn is_range_free(&self, range: FrameRange) -> bool {
    if !self.contains_range(range) {
      return false;
    }
    let first = range.get_first_frame_index();
    let last = range.get_last_frame_index();
    for frame in first..=last {
      let byte_index = frame >> 3;
      let bitmap_byte = self.map[byte_index];
      let byte_offset = frame & 7;
      if bitmap_byte & (1 << byte_offset) != 0 {
        return false;
      }
    }
    true
  }

  pub fn find_free_range(&self, frame_count: usize) -> Option<FrameRange> {
    let mut frame = 0;
    let mut remaining = frame_count;
    let mut search_start = 0;
    while frame < self.frame_count {
      let byte_index = frame >> 3;
      let frame_mask = 1 << (frame & 7);
      if self.map[byte_index] & frame_mask != 0 {
        // occupied, start the search over
        remaining = frame_count;
        search_start = frame + 1;
      } else {
        remaining -= 1;
        if remaining == 0 {
          let starting_address = search_start << 12;
          let length = (frame + 1 - search_start) << 12;
          return Some(FrameRange::new(starting_address, length));
        }
      }
      frame += 1;
    }
    None
  }

  /**
   * Allocate a specific range -- useful when you need access to a known memory
   * address for memmapped IO, DMA, etc.
   */
  pub fn allocate_range(&mut self, range: FrameRange) -> Result<(), BitmapError> {
    if !self.contains_range(range) {
      return Err(BitmapError::OutOfBounds);
    }
    let first = range.get_first_frame_index();
    let last = range.get_last_frame_index();
    for frame in first..=last {
      let byte_index = frame >> 3;
      self.map[byte_index] |= 1 << (frame & 7);
    }
    Ok(())
  }

  /**
   * Allocate a *physically contiguous* set of frames, returning a reference to
   * the available memory area.
   * If you don't need a contiguous block of memory, it may be better to request
   * one frame at a time.
   */
  pub fn allocate_frames(&mut self, frame_count: usize) -> Result<FrameRange, BitmapError> {
    let range = match self.find_free_range(frame_count) {
      Some(r) => r,
      None => return Err(BitmapError::NoAvailableSpace),
    };
    match self.allocate_range(range) {
      Ok(()) => Ok(range),
      Err(e) => Err(e)
    }
  }

  /**
   * Mark a range as unused. Any subset of it may be used to fulfill a future
   * allocation request.
   */
  pub fn free_range(&mut self, range: FrameRange) -> Result<(), BitmapError> {
    if !self.contains_range(range) {
      return Err(BitmapError::OutOfBounds);
    }
    let first = range.get_first_frame_index();
    let last = range.get_last_frame_index();
    for frame in first..=last {
      let byte_index = frame >> 3;
      self.map[byte_index] &= !(1 << (frame & 7));
    }
    Ok(())
  }

  pub fn get_frame_count(&self) -> usize {
    self.frame_count
  }

  /**
   * Compute the number of unallocated frames. Basically, tells you how much
   * memory is available.
   */
  pub fn get_free_frame_count(&self) -> usize {
    let mut frame = 0;
    let mut free = 0;
    while frame < self.frame_count {
      let index = frame >> 3;
      let map_value = self.map[index];
      if map_value != 0xff {
        let mut mask = 1;
        while mask != 0 {
          if map_value & mask == 0 {
            free += 1;
          }
          mask <<= 1;
          frame += 1;
        }
      } else {
        frame += 1;
      }
    }

    free
  }
}

#[derive(PartialEq)]
pub enum BitmapError {
  NoAvailableSpace,
  OutOfBounds,
}

impl core::fmt::Debug for BitmapError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      BitmapError::NoAvailableSpace => f.write_str("FrameBitmap: No available space"),
      BitmapError::OutOfBounds => f.write_str("FrameBitmap: Out of bounds"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{BitmapError, FrameBitmap, FrameRange};

  #[test]
  fn bitmap_creation() {
    let memory: [u8; 4] = [0; 4];
    let bitmap = FrameBitmap::at_location(&memory[1] as *const u8 as usize, 10);
    assert!(bitmap.contains_frame_index(9));
    assert!(!bitmap.contains_frame_index(10));
    
    assert!(bitmap.is_range_free(FrameRange::new(0, 0xa000)));
    assert!(bitmap.is_range_free(FrameRange::new(0x5000, 0x3000)));
    assert!(!bitmap.is_range_free(FrameRange::new(0, 0xb000)));
    assert!(!bitmap.is_range_free(FrameRange::new(0xc000, 0x1000)));
  }

  #[test]
  fn bitmap_allocate() {
    let memory: [u8; 2] = [0; 2];
    let mut bitmap = FrameBitmap::at_location(&memory[0] as *const u8 as usize, 10);
    bitmap.allocate_range(FrameRange::new(0, 0x2000)).unwrap();
    assert_eq!(memory, [3, 0]);
    bitmap.allocate_range(FrameRange::new(0x6000, 0x3000)).unwrap();
    assert_eq!(memory, [0xc3, 1]);
    assert_eq!(bitmap.allocate_range(FrameRange::new(0x8000, 0x7000)), Err(BitmapError::OutOfBounds));
    assert_eq!(memory, [0xc3, 1]);
  }

  #[test]
  fn bitmap_free() {
    let memory: [u8; 2] = [0; 2];
    let mut bitmap = FrameBitmap::at_location(&memory[0] as *const u8 as usize, 10);
    bitmap.allocate_range(FrameRange::new(0, 0xa000)).unwrap();
    assert_eq!(memory, [0xff, 0x03]);
    bitmap.free_range(FrameRange::new(0, 0x3000)).unwrap();
    assert_eq!(memory, [0xf8, 0x03]);
    bitmap.free_range(FrameRange::new(0x8000, 0x2000)).unwrap();
    assert_eq!(memory, [0xf8, 0]);
  }

  #[test]
  fn find_free_range() {
    let memory: [u8; 8] = [0; 8];
    let mut bitmap = FrameBitmap::at_location(&memory[0] as *const u8 as usize, 60);
    assert_eq!(bitmap.find_free_range(4), Some(FrameRange::new(0, 0x4000)));
    assert_eq!(bitmap.find_free_range(80), None);
    bitmap.allocate_range(FrameRange::new(0, 0x2000)).unwrap();
    bitmap.allocate_range(FrameRange::new(0x4000, 0x3000)).unwrap();
    assert_eq!(bitmap.find_free_range(3), Some(FrameRange::new(0x7000, 0x3000)));
    assert_eq!(bitmap.find_free_range(1), Some(FrameRange::new(0x2000, 0x1000)));
    bitmap.allocate_range(FrameRange::new(0x7000, 0xb000)).unwrap();
    assert_eq!(bitmap.find_free_range(4), Some(FrameRange::new(0x12000, 0x4000)));
  }

  #[test]
  fn free_frame_count() {
    let memory: [u8; 8] = [0; 8];
    let mut bitmap = FrameBitmap::at_location(&memory[0] as *const u8 as usize, 60);
    bitmap.reset();
    bitmap.free_range(FrameRange::new(0, 0x3c000)).unwrap();
    assert_eq!(bitmap.get_free_frame_count(), 60);
    bitmap.allocate_frames(2).unwrap();
    assert_eq!(bitmap.get_free_frame_count(), 58);
    let range = bitmap.allocate_frames(10).unwrap();
    assert_eq!(bitmap.get_free_frame_count(), 48);
    bitmap.allocate_frames(5).unwrap();
    assert_eq!(bitmap.get_free_frame_count(), 43);
    bitmap.free_range(range).unwrap();
    assert_eq!(bitmap.get_free_frame_count(), 53);
  }
}
