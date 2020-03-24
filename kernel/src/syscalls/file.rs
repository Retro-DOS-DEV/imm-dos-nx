use alloc::sync::Arc;
use crate::devices;
use crate::drivers::driver::{DeviceDriver, LocalHandle};

pub fn open_path(path_str: &'static str) -> u32 {
  if path_str == "DEV:\\ZERO" {
    1
  } else if path_str == "DEV:\\NULL" {
    2
  } else if path_str == "DEV:\\COM1" {
    3
  } else {
    0
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
