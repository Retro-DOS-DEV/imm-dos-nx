use core::fmt;
use crate::process::{id::ProcessID, send_signal};
use crate::x86::io::Port;
use spin::RwLock;

const STATUS_ERROR_IMPENDING: u8 = 1 << 7;
const STATUS_TRANSMIT_IDLE: u8 = 1 << 6;
const STATUS_TRANSMIT_BUFFER_EMPTY: u8 = 1 << 5;
const STATUS_BREAK: u8 = 1 << 4;
const STATUS_FRAME_ERROR: u8 = 1 << 3;
const STATUS_PARITY_ERROR: u8 = 1 << 2;
const STATUS_OVERRUN_ERROR: u8 = 1 << 1;
const STATUS_DATA_READY: u8 = 1;

pub struct SerialPort {
  data: Port,
  interrupt_enable: Port,
  fifo_control: Port,
  line_control: Port,
  modem_control: Port,
  line_status: Port,
  modem_status: Port,

  wake_on_data_ready: RwLock<Option<ProcessID>>,
}

impl SerialPort {
  pub const fn new(initial_port: u16) -> SerialPort {
    SerialPort {
      data: Port::new(initial_port),
      interrupt_enable: Port::new(initial_port + 1),
      fifo_control: Port::new(initial_port + 2),
      line_control: Port::new(initial_port + 3),
      modem_control: Port::new(initial_port + 4),
      line_status: Port::new(initial_port + 5),
      modem_status: Port::new(initial_port + 6),

      wake_on_data_ready: RwLock::new(None),
    }
  }

  pub unsafe fn init(&self) {
    self.interrupt_enable.write_u8(0x01); // Enable data ready interrupt
    self.line_control.write_u8(0x80); // Enable DLAB bit
    self.data.write_u8(0x03); // Set divisor low to 3, aka 38400 baud
    self.interrupt_enable.write_u8(0x00); // Set divisor high
    self.line_control.write_u8(0x03); // 8 bits, no parity, 1 stop bit
    self.fifo_control.write_u8(0xc7); // Enable fifo
    self.modem_control.write_u8(0x0b); // Set RTS/DTR
  }

  pub unsafe fn is_transmitting(&self) -> bool {
    (self.line_status.read_u8() & STATUS_TRANSMIT_BUFFER_EMPTY) == 0
  }

  pub unsafe fn send_byte(&self, byte: u8) {
    while self.is_transmitting() {}
    self.data.write_u8(byte);
  }

  pub unsafe fn has_data(&self) -> bool {
    (self.line_status.read_u8() & STATUS_DATA_READY) != 0
  }

  pub unsafe fn receive_byte(&self) -> Option<u8> {
    if self.has_data() {
      Some(self.data.read_u8())
    } else {
      None
    }
  }

  pub unsafe fn handle_interrupt(&self) {
    let interrupt_info = self.fifo_control.read_u8();
    if interrupt_info & 4 != 0 {
      if let Some(pid) = *self.wake_on_data_ready.read() {
        // Wake the process
        send_signal(pid, syscall::signals::CONTINUE);
      }
    }
  }

  pub fn maybe_set_wake_on_data_ready(&self, pid: ProcessID) {
    let mut wake_on_ready = self.wake_on_data_ready.write();
    if let None = *wake_on_ready {
      *wake_on_ready = Some(pid);
    }
  }

  pub fn force_wake_on_data_ready(&self, pid: ProcessID) {
    let mut wake_on_ready = self.wake_on_data_ready.write();
    *wake_on_ready = Some(pid);
  }

  pub fn clear_wake_on_data_ready(&self) {
    let mut wake_on_ready = self.wake_on_data_ready.write();
    *wake_on_ready = None;
  }
}

impl fmt::Write for SerialPort {
  fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
    unsafe {
      for byte in s.bytes() {
        self.send_byte(byte);
      }
    }
    Ok(())
  }
}