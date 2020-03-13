use crate::x86::io::Port;

pub struct PIC {
  primary_command: Port,
  primary_data: Port,
  secondary_command: Port,
  secondary_data: Port,
}

impl PIC {
  pub const fn new() -> PIC {
    PIC {
      primary_command: Port::new(0x20),
      primary_data: Port::new(0x21),
      secondary_command: Port::new(0xa0),
      secondary_data: Port::new(0xa1),
    }
  }

  pub unsafe fn init(&mut self) {
    self.primary_command.write_u8(0x10 | 0x01);
    self.secondary_command.write_u8(0x10 | 0x01);
    self.primary_data.write_u8(0x30);
    self.secondary_data.write_u8(0x38);
    self.primary_data.write_u8(0x04);
    self.secondary_data.write_u8(0x02);
    self.primary_data.write_u8(0x01);
    self.secondary_data.write_u8(0x01);
  }

  pub unsafe fn acknowledge_interrupt(&mut self, irq: u8) {
    if irq >= 8 {
      // send command to second chip too
      self.secondary_command.write_u8(0x20);
    }
    self.primary_command.write_u8(0x20);
  }
}