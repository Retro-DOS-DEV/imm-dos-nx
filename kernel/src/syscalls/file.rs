use crate::devices;
use crate::files::filename::{self, Path};
use crate::files::handle::{Handle, LocalHandle};
use crate::filesystems;
use crate::filesystems::filesystem::FileSystem;

pub fn open_path(path_str: &'static str) -> u32 {
  let (drive, path) = filename::string_to_drive_and_path(path_str);
  match filesystems::get_fs(drive) {
    Some(fs) => {
      match fs.open(path) {
        Ok(handle) => handle.as_u32(),
        Err(_) => 0,
      }
    },
    None => 0,
  }

  /*
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
  */
}

pub fn close(handle: u32) {
  
}

pub unsafe fn read(handle: u32, dest: *mut u8, length: usize) -> usize {
  match filesystems::get_fs("DEV") {
    Some(dev) => {
      let buffer = core::slice::from_raw_parts_mut(dest, length);
      match dev.read(LocalHandle::new(handle), buffer) {
        Ok(len) => len,
        Err(_) => 0,
      }
    },
    None => 0,
  }
}

pub unsafe fn write(handle: u32, src: *const u8, length: usize) -> usize {
  /*
  let buffer = core::slice::from_raw_parts(src, length);
  match filesystems::DEV.write(LocalHandle::new(handle), buffer) {
    Ok(len) => len,
    Err(_) => 0,
  }
  */
  match filesystems::get_fs("DEV") {
    Some(dev) => {
      let buffer = core::slice::from_raw_parts(src, length);
      match dev.write(LocalHandle::new(handle), buffer) {
        Ok(len) => len,
        Err(_) => 0,
      }
    },
    None => 0,
  }
}
