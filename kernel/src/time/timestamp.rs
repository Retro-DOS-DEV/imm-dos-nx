use core::ops::Add;

/// Unsigned, 32-bit number representing the number of seconds passed since
/// midnight on 1 January 1980. It neglects leap seconds.
/// This is NOT the same as POSIX time.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Timestamp(pub u32);

impl Timestamp {

}

/// High-resolution 64-bit number representing the number of 100ns increments
/// since midnight on 1 January 1980.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct TimestampHires(pub u64);

impl TimestampHires {
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

  pub fn to_timestamp(&self) -> Timestamp {
    Timestamp(self.in_seconds() as u32)
  }

  pub fn from_timestamp(ts: Timestamp) -> TimestampHires {
    TimestampHires(ts.0 as u64 * 10000000)
  }
}

impl Add for TimestampHires {
  type Output = TimestampHires;

  fn add(self, other: TimestampHires) -> TimestampHires {
    TimestampHires(self.0 + other.0)
  }
}