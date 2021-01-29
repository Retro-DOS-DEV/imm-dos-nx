//! The Input process runs with kernel-level permissions, and sleeps until an
//! input interrupt occurs (keyboard, COM). In order to complete quickly, input
//! interrupts push data onto a queue and return without further processing.
//! The Input thread checks this queue whenever it is awake, and forwards that
//! data onto the relevant device driver.

use crate::buffers::RingBuffer;

pub mod buffers;
#[cfg(not(test))]
pub mod com;
#[cfg(not(test))]
pub mod keyboard;

/// The raw buffer used to enqueue input events
static mut INPUT_EVENTS_DATA: [u8; 32] = [0; 32];
/// A ringbuffer that wraps the raw buffer above, making a contiguous range of
/// event data
pub static INPUT_EVENTS: RingBuffer = RingBuffer::new(unsafe { &INPUT_EVENTS_DATA });
/// Global instance of the keyboard state machine
#[cfg(not(test))]
static KEYBOARD: spin::RwLock<keyboard::Keyboard> = spin::RwLock::new(keyboard::Keyboard::new());

/// The main process thread for handling inputs.
#[cfg(not(test))]
#[inline(never)]
pub extern fn run_input() {
  crate::kprintln!("Input process ready");

  let mut read_buffer: [u8; 1] = [0; 1];
  loop {
    let input_to_read = INPUT_EVENTS.available_bytes();
    for _ in 0..input_to_read {
      let read_len = INPUT_EVENTS.read(&mut read_buffer);
      if read_len < 1 {
        break;
      }
      // Send the data to the keyboard state machine
      let result = KEYBOARD.write().handle_raw_data(read_buffer[0]);
      // If an action occurs, send it to all readers
      match result {
        Some(action) => {
          keyboard::device::write_all(action.to_raw());
        },
        None => (),
      }
    }
    crate::task::yield_coop();
  }

  panic!("Input process exited");
}
