//! DevFS is a virtual filesystem that exposes hardware device drivers as files.
//! Each device has a unique name, 

use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::collections::SlotList;
use crate::devices::{get_driver_for_device, get_device_number_by_name, driver::{DeviceDriverType, IOHandle}};
use crate::files::{cursor::SeekMethod, handle::{Handle, LocalHandle}};
use crate::fs::KernelFileSystem;
use crate::task::id::ProcessID;
use spin::RwLock;
use syscall::files::{DirEntryInfo, FileStatus};

#[derive(Copy, Clone)]
struct DeviceHandle {
  pub device_number: usize,
  pub io_handle: IOHandle,
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

    let io_handle = self.run_device_operation(device_number, |driver| driver.open())?;
    
    let handle = self.handle_to_device.write().insert(
      DeviceHandle {
        device_number,
        io_handle,
      }
    );
    
    Ok(LocalHandle::new(handle as u32))
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let device_handle = self.get_device_handle(handle).ok_or(())?;

    self.run_device_operation(
      device_handle.device_number,
      |driver| driver.read(device_handle.io_handle, buffer),
    )
  }

  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    let device_handle = self.get_device_handle(handle).ok_or(())?;

    self.run_device_operation(
      device_handle.device_number,
      |driver| driver.write(device_handle.io_handle, buffer),
    )
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    let device_handle = self.get_device_handle(handle).ok_or(())?;

    self.run_device_operation(
      device_handle.device_number,
      |driver| driver.close(device_handle.io_handle),
    ).map(|_| ())
  }

  fn reopen(&self, handle: LocalHandle, id: ProcessID) -> Result<LocalHandle, ()> {
    let device_handle = self.get_device_handle(handle).ok_or(())?;

    let io_handle = self.run_device_operation(
      device_handle.device_number,
      |driver| driver.reopen(device_handle.io_handle, id),
    )?;

    let new_handle = self.handle_to_device.write().insert(
      DeviceHandle {
        device_number: device_handle.device_number,
        io_handle,
      }
    );
    
    Ok(LocalHandle::new(new_handle as u32))
  }

  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()> {
    let device_handle = self.get_device_handle(handle).ok_or(())?;

    self.run_device_operation(
      device_handle.device_number,
      |driver| driver.seek(device_handle.io_handle, offset),
    )
  }
  
  fn open_dir(&self, _path: &str) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn read_dir(&self, _handle: LocalHandle, _index: usize, _info: &mut DirEntryInfo) -> Result<bool, ()> {
    Err(())
  }

  fn ioctl(&self, _handle: LocalHandle, _command: u32, _arg: u32) -> Result<u32, ()> {
    Err(())
  }

  fn stat(&self, _handle: LocalHandle, _status: &mut FileStatus) -> Result<(), ()> {
    Err(())
  }
}
