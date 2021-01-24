//! Parsing and loading for DOS COM binaries
//! Similar to BIN, there isn't any parsing or loading to do. The file is mapped
//! to 0x0000, and initial registers are set up according to DOS convention.

use crate::files::handle::LocalHandle;
use crate::fs::{drive::DriveID, DRIVES};
use super::LoaderError;
use super::environment::{ExecutionEnvironment, InitialRegisters};

pub fn build_environment(
  drive_id: DriveID,
  local_handle: LocalHandle,
) -> Result<ExecutionEnvironment, LoaderError> {
  let file_size = {
    let (_, instance) = DRIVES.get_drive_instance(&drive_id).ok_or(LoaderError::FileNotFound)?;
    let mut stat = syscall::files::FileStatus::empty();
    let _ = instance.stat(local_handle, &mut stat).map_err(|_| LoaderError::FileNotFound)?;
    stat.byte_size
  };
  // Same memory segmentation setup as BIN
  let segments = super::bin::build_single_section_environment(file_size, 0x100)?;

  // When running a COM file, the DOS shell is supposed to interpret the first
  // two command-line arguments and determine if they represent files.
  // If they start with references to valid drives, AL or AH (for FCB 1 and 2,
  // respectively) will be set to 0x00; if they are invalid, they will be set to
  // 0xff.
  Ok(
    ExecutionEnvironment {
      segments,
      registers: InitialRegisters {
        /// %eax should represent the validity of the pre-constructed FCBs
        /// Obviously this still needs to be implemented, or handled elsewhere
        eax: Some(0),

        eip: Some(0x100),
        esp: Some(0xfffe),

        cs: Some(0),
        ds: Some(0),
        es: Some(0),
        ss: Some(0),
      },
      require_vm: true,
    }
  )
}
