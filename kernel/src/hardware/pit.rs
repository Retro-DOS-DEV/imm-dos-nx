use crate::x86::io::Port;

pub struct PIT {
  channel_0_data: Port,
  channel_2_data: Port,
  command: Port,
}

impl PIT {
  pub const fn new() -> PIT {
    PIT {
      channel_0_data: Port::new(0x40),
      channel_2_data: Port::new(0x42),
      command: Port::new(0x43),
    }
  }

  pub unsafe fn set_divider(&mut self, div: u16) {
    self.command.write_u8(0x36); // BCD disabled + Mode 3 (Square Wave) + LSB/MSB IO
    self.channel_0_data.write_u8((div & 0xff) as u8); // LSB
    self.channel_0_data.write_u8((div >> 8) as u8); // MSB
  }
}