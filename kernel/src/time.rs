/**
 * Track system time
 */

use core::ops::Add;
use spin::Mutex;

use crate::interrupts;

// Represents time in number of 100ns increments since midnight on Jan 1, 1980
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct SystemTime(u64);

impl SystemTime {
  pub const fn new(value: u64) -> SystemTime {
    SystemTime(value)
  }

  pub fn set(&mut self, value: u64) {
    self.0 = value;
  }

  pub fn increment(&mut self, value: u64) {
    self.0 += value;
  }

  pub fn in_ms(&self) -> u64 {
    self.0 / 10000
  }

  pub fn in_seconds(&self) -> u64 {
    self.0 / 10000000
  }
}

impl Add for SystemTime {
  type Output = SystemTime;

  fn add(self, other: SystemTime) -> SystemTime {
    SystemTime(self.0 + other.0)
  }
}

// Store a known fixed point in time, sourced from CMOS RTC or (in the future)
// a NTP service. We use the programmable timer to update an offset relative to
// this.
static KNOWN_TIME: Mutex<SystemTime> = Mutex::new(SystemTime::new(0));

// Store an offset, regularly updated by the PIT
static TIME_OFFSET: Mutex<SystemTime> = Mutex::new(SystemTime::new(0));

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

pub fn get_system_time() -> SystemTime {
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