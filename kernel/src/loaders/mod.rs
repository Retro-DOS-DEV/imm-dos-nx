//! The loader module sets up a file for execution.
//! Based on the (possibly auto-detected) format of the file, it reads the
//! header of the file and determines how it should be laid out in memory.
//! Once processing is done, it returns an "Environment" containing information
//! on how to copy program data into memory, what to set the initial registers
//! to, and how to perform any relocations.

use crate::files::filename;
use crate::files::handle::LocalHandle;
use crate::fs::{drive::DriveID, DRIVES};
use syscall::result::SystemError;

pub mod bin;
pub mod com;
pub mod elf;
pub mod environment;
pub mod mz;

pub enum ExecutableFormat {
  /// Native 32-bit binary using IMM-DOS syscalls and linear memory
  BIN,
  /// Native 32-bit binary using IMM-DOS syscalls, with segments defined by an
  /// ELF header
  ELF,
  /// 16-bit DOS binary with no header or segmentation
  COM,
  /// 16-bit DOS MZ Executable,
  MZ,
}

/// Tells the kernel what type of executable it should expect
pub enum InterpretationMode {
  /// Attempt to determine the executable type from magic numbers.
  /// If none is detected, it will be interpreted as a native static binary.
  Detect,
  /// Interpret it as a native IMM-DOS program, either ELF or BIN
  Native,
  /// Interpret it as a DOS program, either MZ EXE or COM
  DOS,
}

impl InterpretationMode {
  pub fn from_u32(raw: u32) -> InterpretationMode {
    match raw {
      1 => InterpretationMode::Native,
      2 => InterpretationMode::DOS,
      _ => InterpretationMode::Detect,
    }
  }
}

pub enum LoaderError {
  FileNotFound,
  InternalError,
  InvalidHeader,
}

impl LoaderError {
  pub fn to_system_error(&self) -> SystemError {
    match self {
      LoaderError::FileNotFound => SystemError::NoSuchEntity,
      LoaderError::InternalError => SystemError::Unknown,
      LoaderError::InvalidHeader => SystemError::Unknown,
    }
  }
}

/// Read the header of an open file, and based on the current interpretation
/// mode, attempt to determine what kind of executable file it is.
pub fn determine_format(
  drive_id: DriveID,
  local_handle: LocalHandle,
  interp_mode: InterpretationMode,
  extension: Option<&str>,
) -> Result<ExecutableFormat, LoaderError> {
  let mut magic_number: [u8; 4] = [0; 4];
  let (_, instance) = DRIVES.get_drive_instance(&drive_id).ok_or(LoaderError::FileNotFound)?;
  let _ = instance.read(local_handle, &mut magic_number).map_err(|_| LoaderError::FileNotFound)?;
  let is_elf = magic_number == [0x7f, 0x45, 0x4c, 0x46];
  let is_mz = magic_number[0..2] == [b'M', b'Z'] || magic_number[0..2] == [b'Z', b'M'];
  let format = match interp_mode {
    InterpretationMode::Detect => {
      if is_elf {
        ExecutableFormat::ELF
      } else if is_mz {
        ExecutableFormat::MZ
      } else {
        match extension {
          Some("com") => ExecutableFormat::COM,
          Some("bin") => ExecutableFormat::BIN,
          _ => ExecutableFormat::BIN,
        }
      }
    },
    InterpretationMode::Native => {
      if is_elf {
        ExecutableFormat::ELF
      } else {
        ExecutableFormat::BIN
      }
    },
    InterpretationMode::DOS => {
      if is_mz {
        ExecutableFormat::MZ
      } else {
        ExecutableFormat::COM
      }
    }
  };
  Ok(format)
}

/// Open an executable file, read its headers to determine how it should be set
/// up in memory, and export the information necessary for a process to run this
/// binary file
pub fn load_executable(
  path_str: &str,
  interp_mode: InterpretationMode,
) -> Result<(DriveID, LocalHandle, environment::ExecutionEnvironment), LoaderError> {
  let (drive_id, full_path) = crate::task::io::get_drive_id_and_path(path_str).map_err(|_| LoaderError::FileNotFound)?;
  let (_, instance) = DRIVES.get_drive_instance(&drive_id).ok_or(LoaderError::FileNotFound)?;
  let local_handle = instance.open(full_path.as_str()).map_err(|_| LoaderError::FileNotFound)?;

  let ext = filename::get_extension(path_str);

  let format = determine_format(drive_id, local_handle, interp_mode, ext)?;
  let env = match format {
    ExecutableFormat::BIN => {
      bin::build_environment(drive_id, local_handle)
    },
    ExecutableFormat::COM => {
      com::build_environment(drive_id, local_handle)
    },
    ExecutableFormat::ELF => {
      elf::build_environment(drive_id, local_handle)
    },
    ExecutableFormat::MZ => {
      mz::build_environment(drive_id, local_handle)
    },
    _ => {
      // Not supported yet
      return Err(LoaderError::InternalError);
    },
  }?;
  Ok((drive_id, local_handle, env))
}
