use crate::devices;
use crate::files::filename::Path;
use crate::files::handle::{Handle, LocalHandle};

pub fn open_path(path_str: &'static str) -> u32 {
  let path = Path::from_str(path_str);
  match path {
    Ok(p) => {
      if p.drive == "DEV     ".as_bytes() {
        match devices::get_device_number_by_name(&p.filename) {
          Some(n) => n as u32,
          None => 0,
        }
        /*
        if p.filename == "ZERO    ".as_bytes() {
          1
        } else if p.filename == "NULL    ".as_bytes() {
          2
        } else if p.filename == "COM1    ".as_bytes() {
          3
        } else {
          0
        }
        */
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
  // pretend the handle is a device number for now
  let driver = {
    let drivers = devices::DEV.read();
    let device_driver = drivers.get_device(handle as usize).unwrap();
    device_driver.clone()
  };
  let buffer = core::slice::from_raw_parts_mut(dest, length);
  match driver.read(LocalHandle::new(0), buffer) {
    Ok(len) => len,
    Err(_) => 0
  }
}

pub unsafe fn write(handle: u32, src: *const u8, length: usize) -> usize {
  // pretend the handle is a device number for now
  let driver = {
    let drivers = devices::DEV.read();
    let device_driver = drivers.get_device(handle as usize).unwrap();
    device_driver.clone()
  };
  let buffer = core::slice::from_raw_parts(src, length);
  match driver.write(LocalHandle::new(0), buffer) {
    Ok(len) => len,
    Err(_) => 0
  }
}
