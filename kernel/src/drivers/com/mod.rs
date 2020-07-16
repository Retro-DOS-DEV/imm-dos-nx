use alloc::collections::VecDeque;
use crate::files::handle::LocalHandle;
use crate::process::id::ProcessID;
use super::driver::{DeviceDriver};
use super::queue::ReadQueue;
use spin::Mutex;

pub mod serial;

use serial::SerialPort;

pub struct ComDevice {
  serial: &'static SerialPort,
  queue: Mutex<VecDeque<ProcessID>>,
}

impl ComDevice {
  pub fn new(serial: &'static SerialPort) -> ComDevice {
    ComDevice {
      serial,
      queue: Mutex::new(VecDeque::with_capacity(2)),
    }
  }
}

impl DeviceDriver for ComDevice {
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

impl ReadQueue for ComDevice {
  fn add_process_to_queue(&self, pid: ProcessID) -> usize {
    let len = {
      let mut queue = self.queue.lock();
      queue.push_back(pid);
      queue.len()
    };
    if len == 1 {
      self.serial.maybe_set_wake_on_data_ready(pid);
    }
    len
  }

  fn remove_first_in_queue(&self) -> Option<ProcessID> {
    self.serial.clear_wake_on_data_ready();
    let (first, next) = {
      let mut queue = self.queue.lock();
      let first = queue.pop_front();
      let next = match queue.get(0) {
        Some(pid) => Some(*pid),
        None => None,
      };
      (first, next)
    };
    if let Some(pid) = next {
      self.serial.force_wake_on_data_ready(pid);
    }
    first
  }

  fn get_queue_length(&self) -> usize {
    self.queue.lock().len()
  }

  fn get_first_process_in_queue(&self) -> Option<ProcessID> {
    let queue = self.queue.lock();
    let first = queue.get(0)?;
    Some(*first)
  }

  fn is_data_available(&self) -> bool {
    unsafe {
      self.serial.has_data()
    }
  }

  fn read_available_data(&self, buffer: &mut [u8]) -> usize {
    let mut read = 0;
    unsafe {
      while read < buffer.len() && self.serial.has_data() {
        if let Some(data) = self.serial.receive_byte() {
          buffer[read] = data;
          read += 1;
        }
      }
    }
    read
  }
}
