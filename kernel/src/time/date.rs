use super::timestamp::Timestamp;

#[derive(Eq, PartialEq)]
pub struct Date {
  pub day: u8,
  pub month: u8,
  pub year: u8,
}

impl core::fmt::Debug for Date {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_fmt(format_args!("Date({:02}-{:02}-{:04})", self.day, self.month, self.year as u32 + 1980))
  }
}

impl core::fmt::Display for Date {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_fmt(format_args!("{:02}-{:02}-{:04}", self.day, self.month, self.year as u32 + 1980))
  }
}

#[derive(Eq, PartialEq)]
pub struct Time {
  pub hours: u8,
  pub minutes: u8,
  pub seconds: u8,
}

impl core::fmt::Debug for Time {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_fmt(format_args!("Time({:02}:{:02}:{:02})", self.hours, self.minutes, self.seconds))
  }
}

impl core::fmt::Display for Time {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_fmt(format_args!("{:02}:{:02}:{:02}", self.hours, self.minutes, self.seconds))
  }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DateTime {
  pub date: Date,
  pub time: Time,
}

const SECONDS_IN_DAY: u32 = 60 * 60 * 24;

const MONTH_START_OFFSET: [u32; 12] = [
  0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334,
];

pub fn year_offset_from_days(days: u32) -> u32 {
  let hundredths = days * 100;
  hundredths / 36525
}

impl DateTime {
  pub fn from_unix_epoch(seconds: u32) -> DateTime {
    // from http://howardhinnant.github.io/date_algorithms.html
    // 719468 = shift from epoch start to 2000-03-01
    let days_shift = 719468;
    let days_in_era = 146097;
    let days = seconds / SECONDS_IN_DAY + days_shift;
    let era = days / days_in_era;
    let day_of_era = days - era * days_in_era;
    let year_of_era = (
      day_of_era -
      day_of_era / 1460 +
      day_of_era / 36524 -
      day_of_era / (days_in_era - 1)
    ) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (year_of_era * 365 + year_of_era / 4 - year_of_era / 100);
    let month_partial = (day_of_year * 5 + 2) / 153;
    let day_of_month = day_of_year - (153 * month_partial + 2) / 5 + 1;
    let month = if month_partial < 10 { month_partial + 3 } else { month_partial - 9 };
    
    if month <= 2 {
      year += 1;
    }

    if year < 1980 {
      year = 0; // idk, fix this?
    } else {
      year -= 1980;
    }

    let seconds_of_day = seconds - (seconds / SECONDS_IN_DAY) * SECONDS_IN_DAY;
    let hours = seconds_of_day / 60 / 60;
    let minutes_of_day = seconds_of_day / 60;
    let minutes = minutes_of_day - (hours * 60);
    let sec = seconds_of_day - (hours * 60 * 60) - (minutes * 60);

    DateTime {
      date: Date {
        day: day_of_month as u8,
        month: month as u8,
        year: year as u8,
      },

      time: Time {
        seconds: sec as u8,
        minutes: minutes as u8,
        hours: hours as u8,
      }
    }
  }
}

impl Timestamp {
  pub fn to_days_with_remainder(&self) -> (u32, u32) {
    let days = self.0 / SECONDS_IN_DAY;
    let remainder = self.0 % SECONDS_IN_DAY;
    (days, remainder)
  }

  pub fn to_datetime(&self) -> DateTime {
    let (days, raw_time) = self.to_days_with_remainder();
    let year_offset = year_offset_from_days(days);
    let quadrennial_days = days % (365 + 365 + 365 + 366);
    let year_days = if quadrennial_days > 365 { (quadrennial_days - 366) % 365 } else { quadrennial_days };
    let mut month = 0;
    let mut leap = 0;
    while month < 12 && MONTH_START_OFFSET[month] + leap <= year_days {
      month += 1;
      if month == 2 && year_offset % 4 == 0 {
        // 2000 is a leap year, don't need to check against 2100
        leap = 1;
      }
    }
    let mut day = year_days + 1 - MONTH_START_OFFSET[month - 1];
    if month > 2 {
      day -= leap;
    }

    let total_minutes = raw_time / 60;
    let seconds = raw_time % 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    DateTime {
      date: Date {
        day: day as u8,
        month: month as u8,
        year: year_offset as u8,
      },
      time: Time {
        hours: hours as u8,
        minutes: minutes as u8,
        seconds: seconds as u8,
      },
    }
  }

  pub fn from_datetime(dt: DateTime) -> Timestamp {
    let quadrennials = dt.date.year as u32 / 4;
    let year_remainder = dt.date.year as u32 % 4;
    let mut days = quadrennials * (366 + 365 + 365 + 365) + year_remainder * 365;
    if year_remainder > 0 {
      days += 1;
    }
    days += MONTH_START_OFFSET[dt.date.month as usize - 1];
    days += dt.date.day as u32;

    let timestamp = days * SECONDS_IN_DAY
      + dt.time.hours as u32 * 60 * 60
      + dt.time.minutes as u32 * 60
      + dt.time.seconds as u32;

    Timestamp(timestamp)
  }
}

#[cfg(test)]
mod tests {
  use super::{Date, Time, DateTime, Timestamp, year_offset_from_days};

  #[test]
  fn year_offset() {
    assert_eq!(year_offset_from_days(1), 0);
    assert_eq!(year_offset_from_days(365), 0); // 1980 is a leap year
    assert_eq!(year_offset_from_days(366), 1);
    assert_eq!(year_offset_from_days(366 + 365 + 365 + 365), 4);
    assert_eq!(year_offset_from_days(366 + 365 + 365 + 365 + 365), 4);
    assert_eq!(year_offset_from_days(366 + 365 + 365 + 365 + 366), 5);
  }

  #[test]
  fn extract_time() {
    let mut time = Timestamp(1).to_datetime().time;
    assert_eq!(time, Time{ hours: 0, minutes: 0, seconds: 1 });
    time = Timestamp(16332).to_datetime().time;
    assert_eq!(time, Time{ hours: 4, minutes: 32, seconds: 12 });
    time = Timestamp(93595).to_datetime().time;
    assert_eq!(time, Time{ hours: 1, minutes: 59, seconds: 55 });
  }

  #[test]
  fn extract_date() {
    let mut date = Timestamp(10).to_datetime().date;
    assert_eq!(date, Date{ day: 1, month: 1, year: 0 });
    date = Timestamp(2592000).to_datetime().date;
    assert_eq!(date, Date{ day: 31, month: 1, year: 0 });
    date = Timestamp(2678400).to_datetime().date;
    assert_eq!(date, Date{ day: 1, month: 2, year: 0 });
    date = Timestamp(5097600).to_datetime().date;
    assert_eq!(date, Date{ day: 29, month: 2, year: 0 });
    date = Timestamp(5184000).to_datetime().date;
    assert_eq!(date, Date{ day: 1, month: 3, year: 0 });
    date = Timestamp(7862400).to_datetime().date;
    assert_eq!(date, Date{ day: 1, month: 4, year: 0 });
    date = Timestamp(31622400).to_datetime().date;
    assert_eq!(date, Date{ day: 1, month: 1, year: 1 });
    date = Timestamp(126230400).to_datetime().date;
    assert_eq!(date, Date{ day: 1, month: 1, year: 4 });
    date = Timestamp(131328000).to_datetime().date;
    assert_eq!(date, Date{ day: 29, month: 2, year: 4 });

    date = Timestamp(1278713001).to_datetime().date;
    assert_eq!(date, Date{ day: 8, month: 7, year: 40 });
  }

  #[test]
  fn to_timestamp() {
    let dt = Timestamp(1278713001).to_datetime();
    assert_eq!(Timestamp::from_datetime(dt), Timestamp(1278713001));
  }

  #[test]
  fn from_unix() {
    assert_eq!(
      DateTime::from_unix_epoch(951868800),
      DateTime {
        date: Date {
          day: 1,
          month: 3,
          year: 20,
        },
        time: Time {
          hours: 0,
          minutes: 0,
          seconds: 0,
        },
      },
    );

    assert_eq!(
      DateTime::from_unix_epoch(1635291103),
      DateTime {
        date: Date {
          day: 26,
          month: 10,
          year: 41,
        },
        time: Time {
          hours: 23,
          minutes: 31,
          seconds: 43,
        },
      },
    );
  }
}
