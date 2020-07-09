use crate::time::date::{Date, DateTime, Time};
use crate::x86::io::Port;

pub struct RTC {
  command: Port,
  data: Port,
}

#[derive(Debug)]
pub struct RTCTime {
  seconds: u8,
  minutes: u8,
  hours: u8,

  day: u8,
  month: u8,
  year: u8,
}

impl RTCTime {
  pub fn to_datetime(&self) -> DateTime {
    DateTime {
      date: Date {
        day: self.day,
        month: self.month,
        year: self.year + 20,
      },

      time: Time {
        seconds: self.seconds,
        minutes: self.minutes,
        hours: self.hours,
      }
    }
  }
}

fn convert_bcd(bcd: u8) -> u8 {
  let tens = bcd >> 4;
  let ones = bcd & 0xf;
  tens * 10 + ones
}

impl RTC {
  pub const fn new() -> RTC {
    RTC {
      command: Port::new(0x70),
      data: Port::new(0x71),
    }
  }

  pub unsafe fn read_register(&self, index: u8) -> u8 {
    self.command.write_u8(index);
    self.data.read_u8()
  }

  pub unsafe fn read_time(&self) -> RTCTime {
    let nmi = self.command.read_u8() & 0x80;
    let reg_b = self.read_register(nmi | 0x0b);

    let use_24_hour = reg_b & 2 == 2;
    let use_bcd = reg_b & 4 == 0;

    let mut time = RTCTime {
      seconds: 0,
      minutes: 0,
      hours: 0,

      day: 0,
      month: 0,
      year: 0,
    };

    time.seconds = self.read_register(nmi | 0);
    time.minutes = self.read_register(nmi | 0x02);
    time.hours = self.read_register(nmi | 0x04);
    time.day = self.read_register(nmi | 0x07);
    time.month = self.read_register(nmi | 0x08);
    time.year = self.read_register(nmi | 0x09);

    if use_bcd {
      // Convert all bcd times to binary
      time.seconds = convert_bcd(time.seconds);
      time.minutes = convert_bcd(time.minutes);
      time.day = convert_bcd(time.day);
      time.month = convert_bcd(time.month);
      time.year = convert_bcd(time.year);

      if !use_24_hour {
        let pm = time.hours & 0x80 != 0;
        time.hours = convert_bcd(time.hours & 0x7f);
        time.hours %= 12;
        if pm {
          time.hours += 12;
        }
      }
    } else {
      if !use_24_hour {
        let pm = time.hours & 0x80 != 0;
        time.hours = time.hours & 0x7f;
        time.hours %= 12;
        if pm {
          time.hours += 12;
        }
      }
    }

    time
  }
}