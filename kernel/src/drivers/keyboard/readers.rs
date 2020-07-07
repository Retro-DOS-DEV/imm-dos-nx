use alloc::{collections::BTreeMap, vec::Vec};
use crate::files::handle::LocalHandle;
use super::codes::KeyCode;

pub struct OpenReaders {
  map: BTreeMap<LocalHandle, Vec<KeyCode>>,
}

impl OpenReaders {
  pub fn new() -> OpenReaders {
    OpenReaders {
      map: BTreeMap::new(),
    }
  }

  pub fn open(&mut self, handle: LocalHandle) {
    self.map.insert(handle, Vec::with_capacity(16));
  }

  pub fn read(&mut self, handle: LocalHandle, buffer: &mut [u8]) -> usize {
    match self.map.get_mut(&handle) {
      Some(entry) => {
        let mut read_len = entry.len();
        if buffer.len() < read_len {
          read_len = buffer.len();
        }
        for i in 0..read_len {
          buffer[i] = match entry.pop() {
            Some(code) => code as u8,
            None => 0,
          };
        }
        read_len
      },
      None => 0
    }
  }

  pub fn close(&mut self, handle: LocalHandle) {
    self.map.remove(&handle);
  }

  pub fn get_map(&mut self) -> &mut BTreeMap<LocalHandle, Vec<KeyCode>> {
    &mut self.map
  }
}