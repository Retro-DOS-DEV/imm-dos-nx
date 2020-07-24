use crate::files::handle::LocalHandle;
use crate::process::id::ProcessID;
use super::driver::{DeviceDriver};
use super::queue::ReadQueue;

/// Device driver for interacting with data on a floppy disk. It exposes the
/// floppy disk as a byte stream, and can be used by a filesystem implementation
/// to actually read data on a disk.
/// The floppy driver allows artibrary reads and writes, but the floppy
/// controller only operates at a sector granularity. To accomodate this, the
/// driver maintains an internal LRU cache of sectors that have been read from
/// the disk. Byte-level data can be copied from this in-memory cache.
pub struct FloppyDevice {
  drive_number: usize,
}

impl FloppyDevice {
  pub fn new(drive_number: usize) -> FloppyDevice {
    FloppyDevice {
      drive_number,
    }
  }
}


impl DeviceDriver for FloppyDevice {
  fn open(&self, _handle: LocalHandle) -> Result<(), ()> {
    Ok(())
  }

  fn close(&self, _handle: LocalHandle) -> Result<(), ()> {
    Ok(())
  }

  fn read(&self, _handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let bytes_read = self.blocking_read(buffer);

    Ok(bytes_read)
  }

  fn write(&self, _handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    let mut index = 0;
    while index < buffer.len() {
      unsafe {
        self.serial.send_byte(buffer[index]);
      }
      index += 1;
    }
    Ok(index)
  }
}