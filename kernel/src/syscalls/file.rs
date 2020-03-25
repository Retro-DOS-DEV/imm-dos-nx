use crate::devices;
use crate::files::filename::Path;
use crate::files::handle::{Handle, LocalHandle};
use crate::filesystems;
use crate::filesystems::filesystem::FileSystem;

pub fn open_path(path_str: &'static str) -> u32 {
  let path = Path::from_str(path_str);
  match path {
    Ok(p) => {
      if p.drive == "DEV     ".as_bytes() {
        match filesystems::DEV.open(&p) {
          Ok(handle) => handle.as_u32(),
          Err(_) => 0,
        }
      } else {
        0
      }
    },
    Err(_) => 0,
  }
}

pub fn close(handle: u32) {
  
}

pub unsafe fn read(handle: u32, dest: *mut u8, length: usize) -> usize {
  let buffer = core::slice::from_raw_parts_mut(dest, length);
  match filesystems::DEV.read(LocalHandle::new(handle), buffer) {
    Ok(len) => len,
    Err(_) => 0,
  }
}

pub unsafe fn write(handle: u32, src: *const u8, length: usize) -> usize {
  let buffer = core::slice::from_raw_parts(src, length);
  match filesystems::DEV.write(LocalHandle::new(handle), buffer) {
    Ok(len) => len,
    Err(_) => 0,
  }
}
