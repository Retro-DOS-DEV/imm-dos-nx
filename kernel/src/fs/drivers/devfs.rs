//! DevFS is a virtual filesystem that exposes hardware device drivers as files.
//! Each device has a unique name, 

use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::collections::SlotList;
use crate::devices::{get_driver_for_device, get_device_number_by_name, driver::DeviceDriverType};
use crate::files::{cursor::SeekMethod, handle::{Handle, LocalHandle}};
use crate::fs::KernelFileSystem;
use spin::RwLock;
use syscall::files::{DirEntryInfo, FileStatus};

#[derive(Copy, Clone)]
struct DeviceHandle {
  pub device_number: usize,
  pub local_index: usize,
}

pub struct DevFileSystem {
  handle_to_device: RwLock<SlotList<DeviceHandle>>,
}

impl DevFileSystem {
  pub const fn new() -> Self {
    Self {
      handle_to_device: RwLock::new(SlotList::new()),
    }
  }

  fn get_device_handle(&self, handle: LocalHandle) -> Option<DeviceHandle> {
    self.handle_to_device.read().get(handle.as_usize()).cloned()
  }

  fn run_device_operation<F, T>(&self, device_number: usize, op: F) -> Result<T, ()>
    where F: FnOnce(Arc<Box<DeviceDriverType>>) -> Result<T, ()> {
    
    let result = get_driver_for_device(device_number).map(|dev| op(dev));
    
    match result {
      Some(r) => r,
      None => Err(()),
    }
  }
}

impl KernelFileSystem for DevFileSystem {
  fn open(&self, path: &str) -> Result<LocalHandle, ()> {
    let local_path = if path.starts_with('\\') {
      &path[1..]
    } else {
      path
    };
    let mut path_segments = local_path.split('\\');
    let device_name = path_segments.next().ok_or(())?;
    let device_number = get_device_number_by_name(device_name).ok_or(())?;

    let local_index = self.run_device_operation(device_number, |driver| driver.open())?;
    
    let handle = self.handle_to_device.write().insert(
      DeviceHandle {
        device_number,
        local_index,
      }
    );
    
    Ok(LocalHandle::new(handle as u32))
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let device_handle = self.get_device_handle(handle).ok_or(())?;

    self.run_device_operation(
      device_handle.device_number,
      |driver| driver.read(device_handle.local_index, buffer),
    )
  }

  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Ok(0)
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    let device_handle = self.get_device_handle(handle).ok_or(())?;

    self.run_device_operation(
      device_handle.device_number,
      |driver| driver.close(device_handle.local_index),
    ).map(|_| ())
  }

  fn dup(&self, handle: LocalHandle) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()> {
    Err(())
  }
  
  fn open_dir(&self, path: &str) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn read_dir(&self, handle: LocalHandle, index: usize, info: &mut DirEntryInfo) -> Result<bool, ()> {
    Err(())
  }

  fn ioctl(&self, handle: LocalHandle, command: u32, arg: u32) -> Result<u32, ()> {
    Err(())
  }

  fn stat(&self, handle: LocalHandle, status: &mut FileStatus) -> Result<(), ()> {
    Err(())
  }
}
