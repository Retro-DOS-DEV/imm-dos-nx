//! Device driver implementation for COM ports
//! DEV:/COM_ are single-reader devices, which can only hae one active reader at
//! a time. Any successive readers will be blocked in a queue, until all prior
//! readers have finished or aborted.
//! When data arrives on the serial port, an interrupt is triggered telling the
//! device driver to wake up the current reader.

use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::collections::SlotList;
use crate::devices::driver::{DeviceDriver, IOHandle};
use crate::devices::queue::QueuedIO;
use crate::task::id::ProcessID;
use crate::task::switching::{get_current_id, get_current_process, yield_coop};
use super::serial::SerialPort;
use spin::RwLock;

pub static mut COM_DEVICES: [Option<ComDevice>; 2] = [None, None];

/// Associate a process with a file handle, so that the process can be woken
/// when new data is available.
struct Descriptor {
  pub process: ProcessID,
  pub handle: IOHandle,
}

pub struct ComDevice {
  com: SerialPort,
  next_handle: AtomicUsize,
  open_handles: RwLock<SlotList<Descriptor>>,
  readers: RwLock<VecDeque<IOHandle>>,
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

  pub fn get_interrupt_info(&self) -> u8 {
    self.com.get_interrupt_id()
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

  pub fn open(&self) -> IOHandle {
    let id = IOHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst));
    let desc = Descriptor {
      process: get_current_id(),
      handle: id,
    };
    self.open_handles.write().insert(desc);

    id
  }

  pub fn read(&self, handle: IOHandle, dest: &mut [u8]) -> usize {
    self.perform_io(handle, || {
      let mut bytes_read = 0;
      while bytes_read < dest.len() {
        let partial_read = self.read_available_data(&mut dest[bytes_read..]);
        bytes_read += partial_read;
        if bytes_read < dest.len() {
          get_current_process().write().io_block(None);
          yield_coop();
        }
      }
      bytes_read
    })
  }

  pub fn write(&self, _handle: IOHandle, src: &[u8]) -> usize {
    // TODO: make this not blocking
    let mut written = 0;
    for value in src.iter() {
      self.com.send_byte(*value);
      written += 1;
    }
    written
  }

  pub fn close(&self, handle: IOHandle) {
    let mut handles = self.open_handles.write();
    let handle_index = handles
      .iter()
      .enumerate()
      .find_map(|(i, h)| if h.handle == handle { Some(i) } else { None });
    match handle_index {
      Some(index) => {
        handles.remove(index);
      },
      None => (),
    }
  }
}

impl QueuedIO<(), usize> for ComDevice {
  fn get_process_id_for_handle(&self, handle: IOHandle) -> Option<ProcessID> {
    self.open_handles
      .read()
      .iter()
      .find_map(|o| if o.handle == handle { Some(o.process) } else { None } )
  }

  fn get_io_queue(&self) -> &RwLock<VecDeque<IOHandle>> {
    &self.readers
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
  fn open(&self) -> Result<IOHandle, ()> {
    let device = self.get_device()?;
    Ok(device.open())
  }

  fn read(&self, index: IOHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let device = self.get_device()?;
    Ok(device.read(index, buffer))
  }

  fn write(&self, index: IOHandle, buffer: &[u8]) -> Result<usize, ()> {
    let device = self.get_device()?;
    Ok(device.write(index, buffer))
  }

  fn close(&self, index: IOHandle) -> Result<(), ()> {
    let device = self.get_device()?;
    Ok(device.close(index))
  }
}
