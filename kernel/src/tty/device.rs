use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::collections::SlotList;
use crate::devices::driver::{DeviceDriver, IOHandle};
use crate::task::{get_current_id, id::ProcessID};
use spin::RwLock;
use super::buffers::{TTYReaderBuffer, TTYWriterBuffer, Descriptor};

/// Device driver representing a TTY, so a shell program can open up DEV:/TTY1
/// and listen to console input / publish to the terminal.
pub struct TTYDevice {
  tty_id: usize,
}

impl TTYDevice {
  pub fn for_tty(tty_id: usize) -> TTYDevice {
    TTYDevice {
      tty_id,
    }
  }

  fn with_device_data<F, R>(&self, f: F) -> Result<R, ()>
    where F: FnOnce(&TTYDeviceData) -> Result<R, ()> {
    let data_collection = DEVICE_DATA.read();
    let data = data_collection.get(self.tty_id);
    match data {
      Some(d) => f(d),
      None => Err(())
    }
  }
}

impl DeviceDriver for TTYDevice {
  fn open(&self) -> Result<IOHandle, ()> {
    self.with_device_data(|d| d.open())
    /*
    let router = super::get_router().read();
    let next = router.open_device(self.tty_id);
    match next {
      Some(handle) => Ok(handle),
      None => Err(()),
    }
    */
  }

  fn close(&self, handle: IOHandle) -> Result<(), ()> {
    self.with_device_data(|d| d.close(handle))
    /*
    let router = super::get_router().read();
    router.close_device(self.tty_id, handle);
    Ok(())
    */
  }

  fn read(&self, handle: IOHandle, dest: &mut [u8]) -> Result<usize, ()> {
    self.with_device_data(|d| d.read(handle, dest))
    /*
    let buffer = {
      let router = super::get_router().read();
      router.get_tty_reader_buffer(self.tty_id)
    };
    match buffer {
      Some(b) => {
        let bytes_read = b.read(handle, dest);
        Ok(bytes_read)
      },
      None => Err(()),
    }
    */
  }

  fn write(&self, handle: IOHandle, buffer: &[u8]) -> Result<usize, ()> {
    self.with_device_data(|d| d.write(handle, buffer))
    /*
    // this needs enqueuing
    let mut total_written = 0;
    loop {
      let remainder = &buffer[total_written..];
      let partial_write = {
        let router = super::get_router().read();
        let buffers = router.get_tty_buffers(self.tty_id);
        match buffers {
          Some(b) => {
            let bytes_written = b.write(remainder);
            Ok(bytes_written)
          },
          None => Err(()),
        }
      }?;
      total_written += partial_write;
      if total_written >= buffer.len() {
        break;
      }
      crate::task::yield_coop();
    }
    Ok(total_written)
    */
  }

  fn reopen(&self, index: IOHandle, id: ProcessID) -> Result<IOHandle, ()> {
    /*
    let router = super::get_router().read();
    let new_handle = router.reopen_device(self.tty_id, id);
    match new_handle {
      Some(handle) => Ok(handle),
      None => Err(()),
    }
    */
    Err(())
  }
}

static DEVICE_DATA: RwLock<Vec<TTYDeviceData>> = RwLock::new(Vec::new());

pub struct TTYDeviceData {
  next_handle: AtomicUsize,
  read_buffer: Arc<TTYReaderBuffer>,
  write_buffer: Arc<TTYWriterBuffer>,
  open_io: Arc<RwLock<SlotList<Descriptor>>>,
}

unsafe impl Send for TTYDeviceData {}
unsafe impl Sync for TTYDeviceData {}

impl TTYDeviceData {
  pub fn new() -> Self {
    let open_io = Arc::new(RwLock::new(SlotList::new()));
    let read_buffer = TTYReaderBuffer::new(open_io.clone());
    Self {
      next_handle: AtomicUsize::new(1),
      read_buffer: Arc::new(read_buffer),
      write_buffer: Arc::new(TTYWriterBuffer::new()),
      open_io,
    }
  }

  pub fn get_read_buffer(&self) -> Arc<TTYReaderBuffer> {
    self.read_buffer.clone()
  }

  pub fn get_write_buffer(&self) -> Arc<TTYWriterBuffer> {
    self.write_buffer.clone()
  }

  pub fn open(&self) -> Result<IOHandle, ()> {
    let handle = IOHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst));
    let process = crate::task::get_current_id();
    self.open_io.write().insert(Descriptor { process, handle });
    Ok(handle)
  }

  pub fn close(&self, close_handle: IOHandle) -> Result<(), ()> {
    let mut open_io = self.open_io.write();
    let index = 0;
    let mut to_close: Option<usize> = None;
    while index < open_io.len() {
      let entry = open_io.get(index);
      if let Some(Descriptor { handle, .. }) = entry {
        if *handle == close_handle {
          to_close = Some(index);
          break;
        }
      }
    }
    if let Some(i) = to_close {
      open_io.remove(i);
      Ok(())
    } else {
      Err(())
    }
  }

  pub fn read(&self, handle: IOHandle, dest: &mut [u8]) -> Result<usize, ()> {
    let bytes_read = self.read_buffer.read(handle, dest);
    Ok(bytes_read)
  }

  pub fn write(&self, handle: IOHandle, buffer: &[u8]) -> Result<usize, ()> {
    let bytes_written = self.write_buffer.write(handle, buffer);
    Ok(bytes_written)
  }
}

pub fn get_read_buffer(index: usize) -> Arc<TTYReaderBuffer> {
  DEVICE_DATA.read().get(index).unwrap().get_read_buffer()
}

pub fn get_write_buffer(index: usize) -> Arc<TTYWriterBuffer> {
  DEVICE_DATA.read().get(index).unwrap().get_write_buffer()
}

pub fn create_tty() {
  let device_data = TTYDeviceData::new();
  let index = {
    let mut collection = DEVICE_DATA.write();
    let len = collection.len();
    collection.push(device_data);
    len
  };
  crate::devices::create_tty(index);
}
