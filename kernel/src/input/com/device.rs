//! Device driver implementation for COM ports
//! DEV:/COM_ are single-reader devices, which can only hae one active reader at
//! a time. Any successive readers will be blocked in a queue, until all prior
//! readers have finished or aborted.
//! When data arrives on the serial port, an interrupt is triggered telling the
//! device driver to wake up the current reader.

use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::collections::SlotList;
use crate::devices::driver::DeviceDriver;
use crate::task::id::ProcessID;
use crate::task::switching::{get_current_id, get_current_process, get_process, yield_coop};
use super::serial::SerialPort;
use spin::RwLock;

pub static mut COM_DEVICES: [Option<ComDevice>; 2] = [None, None];

struct Handle {
  pub process: ProcessID,
  pub handle_id: usize,
}

pub struct ComDevice {
  com: SerialPort,
  next_handle: AtomicUsize,
  open_handles: RwLock<SlotList<Handle>>,
  readers: RwLock<VecDeque<usize>>,
}

impl ComDevice {
  pub fn new(first_port: u16) -> Self {
    Self {
      com: SerialPort::new(first_port),
      next_handle: AtomicUsize::new(0),
      open_handles: RwLock::new(SlotList::new()),
      readers: RwLock::new(VecDeque::new()),
    }
  }

  pub fn init(&self) {
    self.com.init();
  }

  pub fn get_id_for_handle(&self, handle: usize) -> Option<ProcessID> {
    self.open_handles
      .read()
      .iter()
      .find_map(|o| if o.handle_id == handle { Some(o.process) } else { None } )
  }

  pub fn get_interrupt_info(&self) -> u8 {
    self.com.get_interrupt_id()
  }

  pub fn wake_front(&self) {
    let next: Option<usize> = self.readers.read().front().copied();
    let next_lock = next
      .and_then(|handle| self.get_id_for_handle(handle))
      .and_then(|id| get_process(&id));
    if let Some(lock) = next_lock {
      lock.write().io_resume();
    }
  }

  pub fn read_available_data(&self, dest: &mut [u8]) -> usize {
    let mut read = 0;
    while read < dest.len() && self.com.has_data() {
      if let Some(data) = self.com.receive_byte() {
        dest[read] = data;
        read += 1;
      }
    }
    read
  }

  pub fn open(&self) -> usize {
    let id = self.next_handle.fetch_add(1, Ordering::SeqCst);
    let handle = Handle {
      process: get_current_id(),
      handle_id: id,
    };
    self.open_handles.write().insert(handle);

    id
  }

  pub fn read(&self, handle: usize, dest: &mut [u8]) -> usize {
    let queued = {
      let mut readers = self.readers.write();
      readers.push_back(handle);
      readers.len()
    };
    if queued > 1 {
      // if there are already readers in line, block until the process is first
      get_current_process().write().io_block(None);
      yield_coop();
    }
    // here, it's guaranteed that the process is first in line
    let mut written = 0;
    while written < dest.len() {
      let partial_read = self.read_available_data(&mut dest[written..]);
      written += partial_read;
      if written < dest.len() {
        get_current_process().write().io_block(None);
        yield_coop();
      }
    }
    // filled the destination buffer, wake the next reader
    self.readers.write().pop_front();
    self.wake_front();
    written
  }

  pub fn close(&self, handle: usize) {
    let mut handles = self.open_handles.write();
    let handle_index = handles
      .iter()
      .enumerate()
      .find_map(|(i, h)| if h.handle_id == handle { Some(i) } else { None });
    match handle_index {
      Some(index) => {
        handles.remove(index);
      },
      None => (),
    }
  }
}

pub struct ComDriver {
  com_number: usize,
}

impl ComDriver {
  pub fn new(com_number: usize) -> Self {
    Self {
      com_number,
    }
  }

  pub fn get_device(&self) -> Result<&ComDevice, ()> {
    unsafe {
      if self.com_number >= COM_DEVICES.len() {
        return Err(());
      }
      match &COM_DEVICES[self.com_number] {
        Some(dev) => Ok(dev),
        None => Err(()),
      }
    }
  }
}

impl DeviceDriver for ComDriver {
  fn open(&self) -> Result<usize, ()> {
    let device = self.get_device()?;
    Ok(device.open())
  }

  fn read(&self, index: usize, buffer: &mut [u8]) -> Result<usize, ()> {
    let device = self.get_device()?;
    Ok(device.read(index, buffer))
  }

  fn write(&self, index: usize, buffer: &[u8]) -> Result<usize, ()> {
    Err(())
  }

  fn close(&self, index: usize) -> Result<(), ()> {
    let device = self.get_device()?;
    Ok(device.close(index))
  }
}
