/**
 * System time tracking
 */

use core::ops::Add;
use spin::Mutex;

use crate::devices;
use crate::interrupts;
use super::timestamp::{Timestamp, TimestampHires};

pub const HUNDRED_NS_PER_TICK: u64 = 100002;
pub const MS_PER_TICK: usize = (HUNDRED_NS_PER_TICK / 10000) as usize;

// Store a known fixed point in time, sourced from CMOS RTC or (in the future)
// a NTP service. We use the programmable timer to update an offset relative to
// this.
static KNOWN_TIME: Mutex<TimestampHires> = Mutex::new(TimestampHires(0));

// Store an offset, regularly updated by the PIT
static TIME_OFFSET: Mutex<TimestampHires> = Mutex::new(TimestampHires(0));

pub fn reset_known_time(time: u64) {
  let int_reenable = interrupts::is_interrupt_enabled();
  interrupts::cli();

  {
    KNOWN_TIME.lock().set(time);
    TIME_OFFSET.lock().set(0);
  }

  if int_reenable {
    interrupts::sti();
  }
}

pub fn get_system_time() -> TimestampHires {
  let int_reenable = interrupts::is_interrupt_enabled();
  interrupts::cli();

  let known = {
    *KNOWN_TIME.lock()
  };
  let offset = {
    *TIME_OFFSET.lock()
  };

  if int_reenable {
    interrupts::sti();
  }

  known + offset
}

pub fn get_offset_seconds() -> u64 {
  let int_reenable = interrupts::is_interrupt_enabled();
  interrupts::cli();

  let seconds = {
    TIME_OFFSET.lock().in_seconds()
  };

  if int_reenable {
    interrupts::sti();
  }
  seconds
}

pub fn increment_offset(delta: u64) {
  let int_reenable = interrupts::is_interrupt_enabled();
  interrupts::cli();

  {
    TIME_OFFSET.lock().increment(delta);
  }

  if int_reenable {
    interrupts::sti();
  }
}

pub fn initialize_from_rtc() {
  let cmos_time = unsafe {
    devices::RTC.read_time()
  };
  let timestamp = Timestamp::from_datetime(cmos_time.to_datetime());
  let system_time = TimestampHires::from_timestamp(timestamp);
  reset_known_time(system_time.0);
}
