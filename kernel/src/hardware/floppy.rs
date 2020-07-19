//! An interface to the low-level Floppy Disk Controller, allowing a driver to
//! communicate with the disk drive hardware.
//! 
//! The controller chip is accessible through a series of registers
//! 
//! Disk access involves sending commands to the controller, and then waiting
//! for an IRQ6 interrupt if the command returns a response. Sending commands
//! involves looping and waiting for some result, and is frequently problematic.
//! Drivers accessing the floppy controller should be aware of this.

use crate::drivers::blocking::WakeReference;
use crate::process::{get_current_pid, send_signal, sleep, yield_coop};
use crate::x86::io::Port;
use spin::RwLock;

#[derive(Copy, Clone, Debug)]
pub enum ControllerError {
  InvalidResponse,
  NotReadyForParam,
  ReadyTimeout,
  UnsupportedController
}

pub struct FloppyController {
  initialized: RwLock<bool>,
  /// Reset before each interrupt-blocked request, set to true each time an INT6
  /// is fired. This helps cover cases where the hardware finishes work before
  /// the driver code starts looking for an interrupt.
  interrupt_received: RwLock<bool>,
  wake_on_int: WakeReference,

  motor_on: RwLock<bool>,

  dor_port: Port,
  msr_port: Port,
  fifo_port: Port,
  ccr_dir_port: Port,
}

impl FloppyController {
  pub const fn new() -> FloppyController {
    FloppyController {
      initialized: RwLock::new(false),
      interrupt_received: RwLock::new(false),
      wake_on_int: WakeReference::new(),
      motor_on: RwLock::new(false),
      dor_port: Port::new(0x3f2),
      msr_port: Port::new(0x3f4),
      fifo_port: Port::new(0x3f5),
      ccr_dir_port: Port::new(0x3f7),
    }
  }

  pub fn is_ready(&self) -> bool {
    *self.initialized.read()
  }

  /// The main status register contains flags that indicate the current stage
  /// of the controller chip. They determine when it is safe to issue commands,
  /// send parameters, and read results.
  pub fn get_status(&self) -> u8 {
    unsafe {
      self.msr_port.read_u8()
    }
  }

  /// When IRQ6 is triggered, this method should be called to alert any blocked
  /// process that work has completed.
  pub fn handle_int6(&self) {
    match self.interrupt_received.try_write() {
      Some(mut lock) => *lock = true,
      // if it's already being written, ignore the interrupt
      None => (),
    }
    self.wake_on_int.wake();
  }

  ///
  pub fn wait_for_interrupt(&self) {
    if let Some(val) = self.interrupt_received.try_read() {
      if *val {
        return;
      }
    }
    let pid = get_current_pid();
    self.wake_on_int.set_process(pid);
    send_signal(pid, syscall::signals::STOP);
    yield_coop();
    self.wake_on_int.clear_process();
  }

  /// The RQM bit indicates that a driver can now read or write data at the FIFO
  /// register. Many procedures involve looping over status register reads,
  /// waiting for the RQM bit to be set. This procedure will yield between reads
  /// so as to not block other processes, and will timeout after a number of
  /// attempts.
  pub fn wait_for_rqm(&self) -> Result<(), ControllerError> {
    let mut retry_count = 10;
    let mut ready = false;
    while !ready && retry_count > 0 {
      ready = self.get_status() & 0x80 == 0x80;
      retry_count -= 1;
      if !ready {
        yield_coop();
      }
    }
    if !ready {
      Err(ControllerError::ReadyTimeout)
    } else {
      Ok(())
    }
  }

  /// Issue a command to the floppy controller. If it succeeds, it will return
  /// an Ok Result. Because not all commands have a response phase, handling
  /// the response from a command is done in a different method.
  pub fn send_command(&self, command: Command, params: &[u8]) -> Result<(), ControllerError> {
    if self.get_status() & 0xc0 != 0x80 {
      self.reset();
    }

    *self.interrupt_received.write() = false;
    unsafe {
      self.fifo_port.write_u8(command as u8);
    }

    let mut param = 0;
    while param < params.len() {
      self.wait_for_rqm()?;
      if self.get_status() & 0x40 != 0 {
        return Err(ControllerError::NotReadyForParam);
      }
      unsafe {
        self.fifo_port.write_u8(params[param]);
      }
      param += 1;
    }
    self.wait_for_rqm()?;
    Ok(())
  }

  /// Attempt to read response bytes and copy them to a mutable slice.
  /// If it succeeds, it will return an `Ok` Response containing the number of
  /// bytes copied to the `response` slice.
  /// If it fails, it will return an `Err` response, and the entire command will
  /// need to be retried.
  pub fn get_response(&self, response: &mut [u8]) -> Result<usize, ControllerError> {
    self.wait_for_rqm()?;
    let mut has_response = self.get_status() & 0x50 == 0x50;
    let mut response_index = 0;
    while has_response {
      if let Some(entry) = response.get_mut(response_index) {
        *entry = unsafe {
          self.fifo_port.read_u8()
        };
        response_index += 1;
      }
      self.wait_for_rqm()?;
      has_response = self.get_status() & 0x50 == 0x50;
    }

    if self.get_status() & 0xd0 == 0x80 {
      Ok(response_index)
    } else {
      Err(ControllerError::InvalidResponse)
    }
  }

  pub fn ensure_motor_on(&self) {
    let mut motor = self.motor_on.write();
    if *motor == true {
      return;
    }
    unsafe {
      let dor = self.dor_port.read_u8();
      self.dor_port.write_u8(0x10 | dor);
    }
    *motor = true;
    sleep(300);
  }

  pub fn reset(&self) -> Result<(), ControllerError> {
    unsafe {
      self.dor_port.write_u8(0);
      self.dor_port.write_u8(0x0c);
    }
    self.wait_for_interrupt();

    let mut sense = [0, 0];
    for _ in 0..4 {
      self.send_command(Command::SenseInterrupt, &[])?;
      self.get_response(&mut sense)?;
    }

    // Start drive select
    // Assume we're using a 1.44M disk
    unsafe {
      self.ccr_dir_port.write_u8(0);
    }
    // SPECIFY, with SRT=8, HUT=0, HLT=5, NDMA=0
    self.send_command(Command::Specify, &[8 << 4, 5 << 1]);
    Ok(())
  }

  pub fn init(&self) -> Result<(), ControllerError> {
    self.send_command(Command::Version, &[])?;
    let mut version_response = [0];
    self.get_response(&mut version_response)?;
    if version_response[0] != 0x90 {
      return Err(ControllerError::UnsupportedController);
    }
    self.send_command(Command::Configure, &[0, 0x78, 0])?;
    self.send_command(Command::Lock, &[])?;
    let mut lock_response = [0];
    self.get_response(&mut lock_response)?;
    // Check if lock bit is set?
    self.reset()?;
    self.ensure_motor_on();
    let mut st0 = [0, 0];
    self.send_command(Command::Recalibrate, &[0])?;
    self.wait_for_interrupt();
    self.send_command(Command::SenseInterrupt, &[])?;
    self.get_response(&mut st0)?;
    if st0[0] & 0x20 != 0x20 {
      // Retry command
      self.send_command(Command::Recalibrate, &[0])?;
      self.wait_for_interrupt();
      self.send_command(Command::SenseInterrupt, &[])?;
      self.get_response(&mut st0)?;
    }

    *self.initialized.write() = true;

    Ok(())
  }

  pub fn read(&self, cylinder: usize, head: usize, sector: usize) {
    self.dma(Command::ReadData, cylinder, head, sector);
  }

  pub fn dma(&self, command: Command, cylinder: usize, head: usize, sector: usize) {
    self.send_command(
      command,
      &[
        (head << 2) as u8,
        cylinder as u8,
        head as u8,
        sector as u8,
        2,
        18,
        0x1b,
        0xff,
      ],
    );
    self.wait_for_interrupt();
    let mut response = [0, 0, 0, 0, 0, 0, 0];
    self.get_response(&mut response);
  }
}

#[repr(u8)]
pub enum Command {
  ReadTrack = 0x02,
  Specify = 0x03,
  SenseDriveStatus = 0x04,
  WriteData = 0x05 | 0x40,
  ReadData = 0x06 | 0x40,
  Recalibrate = 0x07,
  SenseInterrupt = 0x08,
  WriteDeletedData = 0x09,
  ReadID = 0x0a,
  Seek = 0x0f,
  Version = 0x10,
  Configure = 0x13,
  Unlock = 0x14,
  Lock = 0x94,
}