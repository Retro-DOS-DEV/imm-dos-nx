use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::collections::SlotList;
use crate::devices::driver::IOHandle;
use crate::input::keyboard::{KeyAction, codes::KeyCode};
use crate::task::id::ProcessID;
use spin::RwLock;

use super::buffers::{TTYReadWriteBuffers, TTYReaderBuffer};
use super::keyboard::KeyState;
use super::tty::TTY;

/// Associates an open IOHandle with other relevant information
pub struct Descriptor {
  pub process: ProcessID,
  pub handle: IOHandle,
}

/// Associates a TTY driver, containing internal screen state and the ability to
/// write to the VGA buffer, with a device file that can be written and read by
/// other processes.
pub struct TTYData {
  next_handle: AtomicUsize,
  tty: Arc<RwLock<TTY>>,
  buffers: Arc<TTYReadWriteBuffers>,

  descriptors: Arc<RwLock<SlotList<Descriptor>>>,
  reader_buffer: Arc<TTYReaderBuffer>,
}

impl TTYData {
  pub fn new(tty: TTY) -> TTYData {
    let descriptors = Arc::new(RwLock::new(SlotList::new()));
    TTYData {
      next_handle: AtomicUsize::new(0),
      tty: Arc::new(RwLock::new(tty)),
      buffers: Arc::new(TTYReadWriteBuffers::new()),

      descriptors: descriptors.clone(),
      reader_buffer: Arc::new(TTYReaderBuffer::new(descriptors)),
    }
  }

  pub fn open(&self) -> IOHandle {
    let handle = IOHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst));
    let process = crate::task::switching::get_current_id();
    self.descriptors.write().insert(Descriptor { process, handle });
    handle
  }

  pub fn reopen(&self, process: ProcessID) -> IOHandle {
    let handle = IOHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst));
    self.descriptors.write().insert(Descriptor { process, handle });
    handle
  }

  pub fn close(&self, handle: IOHandle) {
    let mut descriptors = self.descriptors.write();
    let index = descriptors
      .iter()
      .enumerate()
      .find_map(|(i, h)| if h.handle == handle { Some(i) } else { None });
    match index {
      Some(i) => {
        descriptors.remove(i);
      },
      None => (),
    }
  }

  pub fn get_tty(&self) -> Arc<RwLock<TTY>> {
    Arc::clone(&self.tty)
  }

  pub fn get_buffers(&self) -> Arc<TTYReadWriteBuffers> {
    Arc::clone(&self.buffers)
  }

  pub fn get_reader_buffer(&self) -> Arc<TTYReaderBuffer> {
    Arc::clone(&self.reader_buffer)
  }
}

/// The TTY Router keeps a record of which TTY is currently "active," and routes
/// all input events there. The active TTY will output keyboard actions to any
/// processes listening to its TTY device file (ie, "DEV:\TTY1")
pub struct TTYRouter {
  tty_set: RwLock<Vec<TTYData>>,
  active_tty: usize,
  key_state: KeyState,
}

impl TTYRouter {
  pub fn new() -> TTYRouter {
    let mut set = Vec::with_capacity(1);
    let mut tty0 = TTY::new();

    set.push(TTYData::new(tty0));
    // Put all other TTYs into the background by default
    let mut tty1 = TTY::new();
    tty1.force_background();
    set.push(TTYData::new(tty1));
    TTYRouter {
      tty_set: RwLock::new(set),
      active_tty: 0,
      key_state: KeyState::new(),
    }
  }

  pub fn create_tty(&self) -> usize {
    let tty = TTY::new();
    
    let mut set = self.tty_set.write();
    let index = set.len();
    set.push(TTYData::new(tty));
    index
  }

  pub fn tty_count(&self) -> usize {
    self.tty_set.read().len()
  }

  pub fn get_tty_buffers(&self, index: usize) -> Option<Arc<TTYReadWriteBuffers>> {
    let set = self.tty_set.read();
    let data = set.get(index);
    match data {
      Some(tty) => Some(tty.get_buffers()),
      None => None
    }
  }

  pub fn get_tty_reader_buffer(&self, index: usize) -> Option<Arc<TTYReaderBuffer>> {
    let set = self.tty_set.read();
    let data = set.get(index);
    match data {
      Some(tty) => Some(tty.get_reader_buffer()),
      None => None
    }
  }

  pub fn open_device(&self, index: usize) -> Option<IOHandle> {
    let set = self.tty_set.read();
    let data = set.get(index);
    match data {
      Some(tty) => Some(tty.open()),
      None => None,
    }
  }

  pub fn reopen_device(&self, index: usize, id: ProcessID) -> Option<IOHandle> {
    let set = self.tty_set.read();
    let data = set.get(index);
    match data {
      Some(tty) => Some(tty.reopen(id)),
      None => None,
    }
  }

  pub fn close_device(&self, index: usize, handle: IOHandle) {
    let set = self.tty_set.read();
    let data = set.get(index);
    if let Some(tty) = data {
      tty.close(handle);
    }
  }

  pub fn get_active_tty(&self) -> Option<Arc<RwLock<TTY>>> {
    let set = self.tty_set.read();
    let active = set.get(self.active_tty);
    match active {
      Some(data) => Some(Arc::clone(&data.tty)),
      None => None
    }
  }

  pub fn set_active_tty(&mut self, index: usize) {
    if self.tty_set.read().len() <= index {
      return;
    }
    if self.active_tty == index {
      return;
    }
    if let Some(tty) = self.get_active_tty() {
      let mut prev = tty.write();
      unsafe { prev.swap_out(); }
    }
    self.active_tty = index;
    if let Some(tty) = self.get_active_tty() {
      let mut next = tty.write();
      unsafe { next.swap_in(); }
    }
  }

  pub fn send_key_action(&mut self, action: KeyAction) {
    let mut buffer: [u8; 4] = [0; 4];

    let output = self.key_state.process_key_action(action, &mut buffer);
    if let Some(len) = output {
      match action {
        KeyAction::Press(KeyCode::Num0) => {
          if self.key_state.alt {
            self.set_active_tty(0);
            return;
          }
        },
        KeyAction::Press(KeyCode::Num1) => {
          if self.key_state.alt {
            self.set_active_tty(1);
            return;
          }
        },
        _ => (),
      }

      let tty_set = self.tty_set.read();
      if let Some(active) = tty_set.get(self.active_tty) {
        let mut tty = active.tty.write();
        let data: &[u8] = &buffer[0..len];
        for i in 0..len {
          tty.handle_input(data[i]);
        }
        active.reader_buffer.add_data(&data);
        //active.buffers.output_buffer.write(&data);
      }
    }
  }

  /// Iterate through all ring buffers, and send all available data to the
  /// matching TTY device.
  pub fn process_buffers(&self) {
    let set = self.tty_set.read();
    for data in set.iter() {
      let buffers = data.get_buffers();
      match data.tty.try_write() {
        Some(mut tty) => {
          let mut data: [u8; 4] = [0; 4];
          let mut to_read = buffers.input_buffer.available_bytes();
          while to_read > 0 {
            let bytes_read = buffers.input_buffer.read(&mut data);
            to_read = if bytes_read == data.len() {
              to_read - bytes_read
            } else {
              0
            };
            for i in 0..bytes_read {
              tty.send_data(data[i]);
            }
          }
        },
        // If the tty is locked, we'll just get to it on the next call
        None => (),
      }
    }
  }
}
