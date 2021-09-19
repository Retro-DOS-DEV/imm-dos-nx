//! Parsing and loading for DOS COM binaries
//! Similar to BIN, there isn't any parsing or loading to do. The file is mapped
//! to a fixed location, and initial registers are set up according to DOS
//! convention.
//! 
//! DOS processes run at an elevated location beyond 0x0000 because they need to
//! preserve space at low memory to stores values that are supposed to be in the
//! internals of the DOS kernel.
//! Multiple DOS API methods are expected to return absolute pointers to structs
//! in the "kernel," so we keep a static map of these structs accessible.

use alloc::vec::Vec;
use crate::dos::execution::PSP;
use crate::files::handle::LocalHandle;
use crate::fs::{drive::DriveID, DRIVES};
use crate::memory::address::VirtualAddress;
use crate::task::memory::{ExecutionSection, ExecutionSegment};
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

  // Set the segment and IP based on the current environment and expected PSP
  // location
  let psp_size = core::mem::size_of::<PSP>() as u32;
  let segment: u32 = 0;
  let ip = psp_size;
  let offset = (segment << 4) + ip;

  // Same memory segmentation setup as BIN
  let segments = build_single_section_environment_with_psp(file_size, offset as usize)?;
  

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

        eip: Some(ip),
        esp: Some(0xfffe),

        cs: Some(segment),
        ds: Some(segment),
        es: Some(segment),
        ss: Some(segment),
      },
      require_vm: true,
    }
  )
}

pub fn build_single_section_environment_with_psp(
  file_size: usize,
  offset: usize,
) -> Result<Vec<ExecutionSegment>, LoaderError> {
  let psp_section = ExecutionSection {
    segment_offset: offset - 0x100,
    executable_offset: None,
    size: 0x100,
  };
  let data_size = file_size + offset;
  let section = ExecutionSection {
    segment_offset: offset,
    executable_offset: Some(0),
    size: data_size,
  };
  let mut page_count = data_size / 0x1000;
  if data_size & 0xfff != 0 {
    page_count += 1;
  }
  let mut segment = ExecutionSegment::at_address(VirtualAddress::new(0), page_count).map_err(|_| LoaderError::InternalError)?;
  segment.set_user_can_write(true);
  segment.add_section(psp_section).map_err(|_| LoaderError::InternalError)?;
  segment.add_section(section).map_err(|_| LoaderError::InternalError)?;
  let mut segments = Vec::with_capacity(1);
  segments.push(segment);
  Ok(segments)
}

