//! An interface to the low-level Floppy Disk Controller, allowing a driver to
//! communicate with the disk drive hardware.
//! 
//! The controller chip is accessible through a series of registers
//! 
//! Disk access involves sending commands to the controller, and then waiting
//! for an IRQ6 interrupt if the command returns a response. Sending commands
//! involves looping and waiting for some result, and is frequently problematic.
//! Drivers accessing the floppy controller should be aware of this.

#[derive(Copy, Clone, Debug)]
pub enum ControllerError {
  InvalidResponse,
  NotReadyForParam,
  ReadyTimeout,
  UnsupportedController
}

use alloc::collections::vec_deque::VecDeque;
use crate::task;
use spin::RwLock;

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

#[derive(Copy, Clone)]
pub enum DriveSelect {
  Primary,
  Secondary,
}

impl DriveSelect {
  pub fn get_number(&self) -> u8 {
    match self {
      DriveSelect::Primary => 0,
      DriveSelect::Secondary => 1,
    }
  }
}

#[derive(Copy, Clone)]
pub enum Operation {
  Read(DriveSelect, usize, usize, usize),
  Write(DriveSelect, usize, usize, usize),
}

#[derive(Copy, Clone)]
pub enum DriveType {
  None,
  Capacity360K,
  Capacity1200K,
  Capacity720K,
  Capacity1440K,
  Capacity2880K,
}

impl DriveType {
  pub fn from_cmos(value: u8) -> Self {
    match value {
      1 => Self::Capacity360K,
      2 => Self::Capacity1200K,
      3 => Self::Capacity720K,
      4 => Self::Capacity1440K,
      5 => Self::Capacity2880K,
      _ => Self::None,
    }
  }
}

impl core::fmt::Display for DriveType {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    match self {
      DriveType::None => f.write_str("Unavailable"),
      DriveType::Capacity360K => f.write_str("360KB 5.25 Disk"),
      DriveType::Capacity1200K => f.write_str("1.2MB 5.25 Disk"),
      DriveType::Capacity720K => f.write_str("720KB 3.5 Disk"),
      DriveType::Capacity1440K => f.write_str("1.44MB 3.5 Disk"),
      DriveType::Capacity2880K => f.write_str("2.88MB 3.5 Disk"),
    }
  }
}

const DOR_PORT_NUMBER: u16 = 0x3F2;
const MSR_PORT_NUMBER: u16  = 0x3f4;
const FIFO_PORT_NUMBER: u16 = 0x3f5;
const CCR_PORT_NUMBER: u16 = 0x3f7;

pub struct FloppyDiskController {
  operation_queue: RwLock<Option<VecDeque<task::id::ProcessID>>>,
  /// Cleared before each operation, and written every time an interrupt comes
  /// in on IRQ 6. This accommodates ultra-fast floppy controllers.
  interrupt_received: RwLock<bool>,
  /// Which process to resume when an interrupt occurs
  wake_on_interrupt: RwLock<Option<task::id::ProcessID>>,

  primary_drive_type: RwLock<DriveType>,
  secondary_drive_type: RwLock<DriveType>,
}

impl FloppyDiskController {
  pub const fn new() -> Self {
    Self {
      operation_queue: RwLock::new(None),
      interrupt_received: RwLock::new(false),
      wake_on_interrupt: RwLock::new(None),

      primary_drive_type: RwLock::new(DriveType::None),
      secondary_drive_type: RwLock::new(DriveType::None),
    }
  }

  /// Triggered by IRQ 6, indicating some disk drive has an update
  pub fn handle_interrupt(&self) {
    // Mark an interrupt as received
    match self.interrupt_received.try_write() {
      Some(mut guard) => *guard = true,
      None => (),
    }
    // Determine which process is executing
    let blocked = self.wake_on_interrupt.try_read().and_then(|r| *r);
    // Awaken the process
    if let Some(id) = blocked {
      resume_from_hardware(id);
    }
  }

  /// Set up the controller for the first time
  pub fn init(&self) -> Result<(), ControllerError> {
    // first, detect how many drives
    {
      let cmos_value = unsafe {
        // read from CMOS register 0x10
        crate::x86::io::outb(0x70, 0x10);
        crate::x86::io::inb(0x71)
      };
      let primary_drive = DriveType::from_cmos(cmos_value >> 4);
      let secondary_drive = DriveType::from_cmos(cmos_value & 0x0f);
      *(self.primary_drive_type.write()) = primary_drive;
      *(self.secondary_drive_type.write()) = secondary_drive;
      crate::klog!("Drives Detected:\n");
      crate::klog!("  Primary Drive:   \x1b[97m{:}\x1b[m\n", primary_drive);
      crate::klog!("  Secondary Drive: \x1b[97m{:}\x1b[m\n", secondary_drive);
    }

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
    // Ensure motor is on
    let mut motor_flag = 0;
    if self.has_primary_drive() {
      motor_flag |= 0x10;
    }
    if self.has_secondary_drive() {
      motor_flag |= 0x20;
    }
    let dor = self.dor_read();
    self.dor_write(dor | motor_flag);
    task::sleep(300);

    if self.has_primary_drive() {
      self.select_drive(DriveSelect::Primary);
      self.recalibrate()?;
    }

    if self.has_secondary_drive() {
      self.select_drive(DriveSelect::Secondary);
      self.recalibrate()?;
    }

    Ok(())
  }

  /// Enqueue a read/write operation from a process
  pub fn add_operation(&self, op: Operation) {
    let current_id = task::switching::get_current_id();
    // Push the process onto the end of the queue, returning the total number of
    // waiting processes
    let len: usize = loop {
      match self.operation_queue.try_write() {
        Some(mut ops) => {
          if let None = *ops {
            *ops = Some(VecDeque::with_capacity(2));
          }
          let q: &mut VecDeque<task::id::ProcessID> = ops.as_mut().unwrap();
          q.push_back(current_id);
          break q.len();
        },
        None => {
          task::yield_coop();
        },
      }
    };
    if len > 1 {
      // block until this process is front of the queue
      block_on_hardware();
    }
    // The operation is now first in the queue
    let result = match op {
      Operation::Read(drive, c, h, s) => {
        self.read(drive, c, h, s)
      },
      Operation::Write(drive, c, h, s) => {
        self.write(drive, c, h, s)
      },
    };

    // This operation is now complete, remove the operation from the queue.
    // If there is another process waiting to read or write, wake it up.
    let next: Option<task::id::ProcessID> = loop {
      match self.operation_queue.try_write() {
        Some(mut q) => {
          let front = q.as_mut().unwrap().pop_front();
          break front;
        },
        None => {
          task::yield_coop();
        },
      }
    };

    let to_wake = match next {
      Some(id) => id,
      None => return,
    };
    resume_from_hardware(to_wake);
  }

  pub fn has_primary_drive(&self) -> bool {
    match *self.primary_drive_type.read() {
      DriveType::None => false,
      _ => true,
    }
  }

  pub fn has_secondary_drive(&self) -> bool {
    match *self.secondary_drive_type.read() {
      DriveType::None => false,
      _ => true,
    }
  }

  fn clear_interrupt_received(&self) {
    *(self.interrupt_received.write()) = false;
  }

  fn ensure_motor_on(&self, drive: DriveSelect) {
    let dor = self.dor_read();
    let flag = match drive {
      DriveSelect::Primary => 0x10,
      DriveSelect::Secondary => 0x20,
    };
    self.dor_write(dor | flag);
    task::sleep(300);
  }

  fn select_drive(&self, drive: DriveSelect) {
    let dor = self.dor_read();
    let select = match drive {
      DriveSelect::Primary => 0,
      DriveSelect::Secondary => 1,
    };
    self.dor_write((dor & 0xfc) | select);
  }

  fn recalibrate(&self) -> Result<(), ControllerError> {
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
    Ok(())
  }

  /// Wait until an IRQ 6 interrupt occurs
  /// When the handler is triggered, it will resume this process
  fn wait_for_interrupt(&self) {
    // Set this first
    let pid = task::switching::get_current_id();
    *self.wake_on_interrupt.write() = Some(pid);

    match self.interrupt_received.try_read() {
      Some(val) => {
        if *val {
          return;
        }
      },
      None => {
        // The only way this is locked is if an interrupt is writing to it,
        // since we queue operations to be one process at a time.
        return;
      },
    }
    block_on_hardware();
    *self.wake_on_interrupt.write() = None;
  }

  fn get_status(&self) -> u8 {
    unsafe {
      crate::x86::io::inb(MSR_PORT_NUMBER)
    }
  }

  fn fifo_write(&self, value: u8) {
    unsafe {
      crate::x86::io::outb(FIFO_PORT_NUMBER, value);
    }
  }

  fn fifo_read(&self) -> u8 {
    unsafe {
      crate::x86::io::inb(FIFO_PORT_NUMBER)
    }
  }

  fn dor_write(&self, value: u8) {
    unsafe {
      crate::x86::io::outb(DOR_PORT_NUMBER, value);
    }
  }

  fn dor_read(&self) -> u8 {
    unsafe {
      crate::x86::io::inb(DOR_PORT_NUMBER)
    }
  }

  /// The RQM bit indicates that a driver can now read or write data at the FIFO
  /// register. Many procedures involve looping over status register reads,
  /// waiting for the RQM bit to be set. This procedure will yield between reads
  /// so as to not block other processes, and will timeout after a number of
  /// attempts.
  fn wait_for_rqm(&self) -> Result<(), ControllerError> {
    let mut retry_count = 10;

    let mut ready = false;
    while !ready && retry_count > 0 {
      ready = self.get_status() & 0x80 == 0x80;
      retry_count -= 1;
      if !ready {
        task::yield_coop();
      }
    }
    if !ready {
      Err(ControllerError::ReadyTimeout)
    } else {
      Ok(())
    }
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
        *entry = self.fifo_read();
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

  /// Reset an uninitialized or locked up controller
  fn reset(&self) -> Result<(), ControllerError> {
    self.dor_write(0);
    // needs to sleep for 4 microseconds, a yield should cover that
    task::yield_coop();
    // Motors off, reset + IRQ enabled, select disk 0
    self.dor_write(0x0c);
    self.wait_for_interrupt();

    let mut sense = [0, 0];
    for _ in 0..4 {
      self.send_command(Command::SenseInterrupt, &[])?;
      self.get_response(&mut sense)?;
    }

    // Start drive select
    // Assume we're using a 1.44M disk
    unsafe {
      crate::x86::io::outb(CCR_PORT_NUMBER, 0);
    }
    // SPECIFY, with "safe values" SRT=8, HUT=0, HLT=5, NDMA=0
    self.send_command(Command::Specify, &[8 << 4, 5 << 1])?;
    Ok(())
  }

  /// Issue a command to the floppy controller. If it succeeds, it will return
  /// an Ok Result. Because not all commands have a response phase, handling
  /// the response from a command is done in a different method.
  fn send_command(&self, command: Command, params: &[u8]) -> Result<(), ControllerError> {
    if self.get_status() & 0xc0 != 0x80 {
      self.reset()?;
    }

    self.clear_interrupt_received();
    self.fifo_write(command as u8);

    // Commands have a variable set of parameters that need to be issued one by
    // one. Loop through the set of parameters, waiting until the controller is
    // ready to receive data, and sending it out byte-by-byte.
    let mut param = 0;
    while param < params.len() {
      self.wait_for_rqm()?;
      if self.get_status() & 0x40 != 0 {
        return Err(ControllerError::NotReadyForParam);
      }
      self.fifo_write(params[param]);
      param += 1;
    }
    self.wait_for_rqm()?;

    Ok(())
  }

  fn read(&self, drive: DriveSelect, c: usize, h: usize, s: usize) -> Result<(), ControllerError> {
    self.select_drive(drive);
    self.dma(Command::ReadData, drive.get_number(), c, h, s)
  }

  fn write(&self, drive: DriveSelect, c: usize, h: usize, s: usize) -> Result<(), ControllerError> {
    self.select_drive(drive);
    self.dma(Command::WriteData, drive.get_number(), c, h, s)
  }

  fn dma(&self, command: Command, drive_number: u8, cylinder: usize, head: usize, sector: usize) -> Result<(), ControllerError> {
    self.send_command(
      command,
      &[
        (head << 2) as u8 | drive_number,
        cylinder as u8,
        head as u8,
        sector as u8,
        2,
        18, // last sector on track
        0x1b, // GAP1 default size
        0xff,
      ],
    )?;
    self.wait_for_interrupt();
    let mut response = [0, 0, 0, 0, 0, 0, 0];
    self.get_response(&mut response)?;
    // Process response

    Ok(())
  }
}

fn block_on_hardware() {
  let current_process = task::switching::get_current_process();
  current_process.write().hardware_block(None);
  task::yield_coop();
}

fn resume_from_hardware(id: task::id::ProcessID) {
  match task::switching::get_process(&id) {
    Some(proc) => proc.write().hardware_resume(),
    None => (),
  }
}
