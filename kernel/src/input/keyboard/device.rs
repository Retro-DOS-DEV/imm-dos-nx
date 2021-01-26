//! Device driver implementation for the keyboard
//! DEV:/KBD is a multi-reader device, which means that any incoming bytes will
//! be sent in parallel to all active readers.

use alloc::sync::Arc;
use crate::collections::SlotList;
use crate::task::switching::{get_current_id, get_current_process, yield_coop};
use spin::RwLock;
use super::super::buffers::InputBuffer;

/// Buffers for each of the processes reading the 
static KEYBOARD_READERS: RwLock<SlotList<Arc<InputBuffer>>> = RwLock::new(SlotList::new());

pub fn open() -> usize {
  let id = get_current_id();
  let buffer = InputBuffer::for_process(id);
  KEYBOARD_READERS.write().insert(Arc::new(buffer))
}

pub fn read(slot: usize, dest: &mut [u8]) -> Result<usize, ()> {
  let buffer = match KEYBOARD_READERS.read().get(slot) {
    Some(entry) => entry.clone(),
    None => return Err(()),
  };
  let mut written = 0;
  while written < dest.len() {
    get_current_process().write().io_block(None);
    buffer.start_read();
    yield_coop();
    let partial_read = buffer.read_to_buffer(&mut dest[written..]);
    written += partial_read;
  }
  Ok(written)
}

pub fn close(slot: usize) {
  KEYBOARD_READERS.write().remove(slot);
}

pub fn write_all(pair: [u8; 2]) {
  let readers = KEYBOARD_READERS.read();
  for r in readers.iter() {
    r.write_pair(pair);
  }
}