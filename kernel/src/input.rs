use crate::buffers::RingBuffer;
use crate::devices;
use crate::process::{self, id::ProcessID};

/**
 * The Input thread runs with kernel-level permissions, and sleeps until an
 * input interrupt occurs (keyboard, COM). In order to complete quickly, input
 * interrupts push data onto a queue and return without further processing.
 * The Input thread checks this queue whenever it is awake, and 
 */

pub static mut INPUT_THREAD_ID: ProcessID = ProcessID::new(0);

static mut INPUT_EVENTS_DATA: [u8; 32] = [0; 32];
pub static INPUT_EVENTS: RingBuffer = RingBuffer::new(unsafe { &INPUT_EVENTS_DATA });

#[inline(never)]
pub extern "C" fn run_input() {
  unsafe {
    INPUT_THREAD_ID = process::get_current_pid();
  }
  crate::kprintln!("INPUT THREAD REPORTING!");
  let mut read_buffer: [u8; 1] = [0; 1];
  loop {
    process::send_signal(unsafe { INPUT_THREAD_ID }, syscall::signals::STOP);
    process::yield_coop();
    let to_read = INPUT_EVENTS.available_bytes();
    for _ in 0..to_read {
      let read_len = INPUT_EVENTS.read(&mut read_buffer);
      if read_len < 1 {
        break;
      }
      unsafe {
        if let Some(kbd) = &devices::KEYBOARD {
          kbd.lock().handle_data(read_buffer[0]);
        }
      }
    }
  }
}

pub fn wake_thread() {
  process::send_signal(unsafe { INPUT_THREAD_ID }, syscall::signals::CONTINUE);
}