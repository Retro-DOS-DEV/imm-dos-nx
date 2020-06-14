use core::slice;
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
          let ending_address = (frame + 1) << 12;
          return Some(FrameRange::new(starting_address, ending_address));
        }
      }
      frame += 1;
    }
    None
  }

  pub fn allocate_range(&mut self, range: FrameRange) -> Result<(), ()> {
    if !self.contains_range(range) {
      return Err(());
    }
    let first = range.get_first_frame_index();
    let last = range.get_last_frame_index();
    for frame in first..=last {
      let byte_index = frame >> 3;
      self.map[byte_index] |= 1 << (frame & 7);
    }
    Ok(())
  }

  pub fn free_range(&mut self, range: FrameRange) -> Result<(), ()> {
    if !self.contains_range(range) {
      return Err(());
    }
    let first = range.get_first_frame_index();
    let last = range.get_last_frame_index();
    for frame in first..=last {
      let byte_index = frame >> 3;
      self.map[byte_index] &= !(1 << (frame & 7));
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::{FrameBitmap, FrameRange};

  #[test]
  fn bitmap_creation() {
    let memory: [u8; 4] = [0; 4];
    let bitmap = FrameBitmap::at_location(&memory[1] as *const u8 as usize, 10);
    assert!(bitmap.contains_frame_index(9));
    assert!(!bitmap.contains_frame_index(10));
    
    assert!(bitmap.is_range_free(FrameRange::new(0, 0xa000)));
    assert!(bitmap.is_range_free(FrameRange::new(0x5000, 0x8000)));
    assert!(!bitmap.is_range_free(FrameRange::new(0, 0xb000)));
    assert!(!bitmap.is_range_free(FrameRange::new(0xc000, 0xd000)));
  }

  #[test]
  fn bitmap_allocate() {
    let memory: [u8; 2] = [0; 2];
    let mut bitmap = FrameBitmap::at_location(&memory[0] as *const u8 as usize, 10);
    bitmap.allocate_range(FrameRange::new(0, 0x2000)).unwrap();
    assert_eq!(memory, [3, 0]);
    bitmap.allocate_range(FrameRange::new(0x6000, 0x9000)).unwrap();
    assert_eq!(memory, [0xc3, 1]);
    assert_eq!(bitmap.allocate_range(FrameRange::new(0x8000, 0xf000)), Err(()));
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
    bitmap.free_range(FrameRange::new(0x8000, 0xa000)).unwrap();
    assert_eq!(memory, [0xf8, 0]);
  }

  #[test]
  fn find_free_range() {
    let memory: [u8; 8] = [0; 8];
    let mut bitmap = FrameBitmap::at_location(&memory[0] as *const u8 as usize, 60);
    assert_eq!(bitmap.find_free_range(4), Some(FrameRange::new(0, 0x4000)));
    assert_eq!(bitmap.find_free_range(80), None);
    bitmap.allocate_range(FrameRange::new(0, 0x2000)).unwrap();
    bitmap.allocate_range(FrameRange::new(0x4000, 0x7000)).unwrap();
    assert_eq!(bitmap.find_free_range(3), Some(FrameRange::new(0x7000, 0xa000)));
    assert_eq!(bitmap.find_free_range(1), Some(FrameRange::new(0x2000, 0x3000)));
    bitmap.allocate_range(FrameRange::new(0x7000, 0x12000)).unwrap();
    assert_eq!(bitmap.find_free_range(4), Some(FrameRange::new(0x12000, 0x16000)));
  }
}
