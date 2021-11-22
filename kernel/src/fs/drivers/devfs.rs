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
struct OpenDevice {
  pub device_number: usize,
  pub io_handle: IOHandle,
}

#[derive(Copy, Clone)]
struct OpenDirectory {
  cursor: usize,
}

#[derive(Copy, Clone)]
enum OpenHandle {
  Device(OpenDevice),
  Directory(OpenDirectory),
}

pub struct DevFileSystem {
  open_handles: RwLock<SlotList<OpenHandle>>,
}

impl DevFileSystem {
  pub const fn new() -> Self {
    Self {
      open_handles: RwLock::new(SlotList::new()),
    }
  }

  fn get_handle(&self, handle: LocalHandle) -> Option<OpenHandle> {
    self.open_handles.read().get(handle.as_usize()).cloned()
  }

  fn get_device_handle(&self, handle: LocalHandle) -> Option<OpenDevice> {
    let open_handle = self.get_handle(handle)?;
    match open_handle {
      OpenHandle::Device(dev) => Some(dev),
      _ => None,
    }
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
    
    let handle = self.open_handles.write().insert(
      OpenHandle::Device(
        OpenDevice {
          device_number,
          io_handle,
        },
      ),
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

    let new_handle = self.open_handles.write().insert(
      OpenHandle::Device(
        OpenDevice {
          device_number: device_handle.device_number,
          io_handle,
        },
      ),
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
  
  fn open_dir(&self, path: &str) -> Result<LocalHandle, ()> {
    if path != "" {
      return Err(());
    }
    let open_dir = OpenDirectory {
      cursor: 0,
    };
    let index = self.open_handles.write().insert(OpenHandle::Directory(open_dir));
    return Ok(LocalHandle::new(index as u32));
  }

  fn read_dir(&self, handle: LocalHandle, info: &mut DirEntryInfo) -> Result<bool, ()> {
    match self.open_handles.write().get_mut(handle.as_usize()) {
      Some(OpenHandle::Directory(open_dir)) => {
        let devices = crate::devices::DEVICES.read();
        let name = match devices.get_device_name(open_dir.cursor) {
          Some(name) => name,
          None => return Err(()),
        };

        let mut name_index = 0;
        for b in name.as_bytes() {
          info.file_name[name_index] = *b;
          name_index += 1;
        }
        for i in name_index..8 {
          info.file_name[i] = 0x20;
        }
        for i in 0..3 {
          info.file_ext[i] = 0x20;
        }
        open_dir.cursor += 1;
        if devices.get_device_name(open_dir.cursor).is_none() {
          Ok(false)
        } else {
          Ok(true)
        }
      },
      Some(OpenHandle::Device(_)) => Err(()),
      None => Err(()),
    }
  }

  fn ioctl(&self, _handle: LocalHandle, _command: u32, _arg: u32) -> Result<u32, ()> {
    Err(())
  }

  fn stat(&self, _handle: LocalHandle, _status: &mut FileStatus) -> Result<(), ()> {
    Err(())
  }
}
