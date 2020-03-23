pub fn open_path(path_str: &'static str) -> u32 {
  if path_str == "DEV:\\NULL" {
    1
  } else {
    0
  }
}

pub fn close(handle: u32) {
  
}

pub unsafe fn read(handle: u32, dest: *mut u8, length: usize) -> usize {
  // forever devnull
  *dest = 0;
  1
}
