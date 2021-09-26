use crate::interrupts::stack::StackFrame;
use super::registers::{DosApiRegisters, VM86Frame};

#[repr(u8)]
pub enum DosError {
  InvalidFunction = 1,
  FileNotFound = 2,
  PathNotFound = 3,
  TooManyOpenFiles = 4,
  AccessDenied = 5,
  InvalidHandle = 6,
  MCBDestroyed = 7,
  InsufficientMemory = 8,
  InvalidMemoryBlock = 9,
  InvalidEnvironment = 10,
  InvalidFormat = 11,
  InvalidAccess = 12,
  InvalidData = 13,
  Reserved14 = 14,
  InvalidDrive = 15,
  RemoveCurrentDir = 16,
  NotSameDevice = 17,
  NoMoreFiles = 18,
  WriteReadOnlyDisk = 19,
  UnknownUnit = 20,
  DriveNotReady = 21,
  UnknownCommand = 22,
  DataError = 23,
  BadRequestStructure = 24,
  SeekError = 25,
  UnknownMediaType = 26,
  SectorNotFound = 27,
  PrinterNoPaper = 28,
  WriteFault = 29,
  ReadFault = 30,
  GeneralFailure = 31,
  SharingViolation = 32,
  LockViolation = 33,
  InvalidDiskChange = 34,
  FCBUnavailable = 35,
  SharingBufferOverflow = 36,
}

pub fn with_error_code<F>(
  regs: &mut DosApiRegisters,
  segments: &mut VM86Frame,
  stack_frame: &StackFrame,
  f: F)
  where F: FnOnce(&mut DosApiRegisters, &mut VM86Frame) -> Result<(), DosError> {

  unsafe {
    match f(regs, segments) {
      Ok(_) => {
        stack_frame.clear_carry_flag();
      },
      Err(err) => {
        let code = err as u8;
        regs.ax = code as u32;
        stack_frame.set_carry_flag();
      },
    }
  }
}