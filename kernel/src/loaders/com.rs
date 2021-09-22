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
  let psp_segment: u32 = 0x100;
  let ip = core::mem::size_of::<PSP>() as u32;

  // Same memory segmentation setup as BIN
  let segments = build_single_section_environment_with_psp(file_size, psp_segment as usize)?;

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

        cs: Some(psp_segment),
        ds: Some(psp_segment),
        es: Some(psp_segment),
        ss: Some(psp_segment),
      },
      require_vm: true,
    }
  )
}

pub fn build_single_section_environment_with_psp(
  file_size: usize,
  psp_segment: usize,
) -> Result<Vec<ExecutionSegment>, LoaderError> {
  let psp_start = psp_segment << 4;
  let psp_size = core::mem::size_of::<PSP>();
  let code_start = psp_start + psp_size;
  let mut page_start = VirtualAddress::new(psp_start)
    .prev_page_barrier();
  crate::kprintln!("SEGMENT START: {:?}", page_start);
  let psp_section = ExecutionSection {
    segment_offset: psp_start - page_start.as_usize(),
    executable_offset: None,
    size: psp_size,
  };
  let section = ExecutionSection {
    segment_offset: code_start - page_start.as_usize(),
    executable_offset: Some(0),
    size: file_size,
  };
  let final_byte = code_start + file_size;
  let total_length = final_byte - page_start.as_usize();
  let mut page_count = total_length / 0x1000;
  if total_length & 0xfff != 0 {
    page_count += 1;
  }

  crate::kprintln!("PSP: {:X}-{:X}-{:X} ({})", psp_start, code_start, final_byte, page_count);
  let mut segment = ExecutionSegment::at_address(
    page_start,
    page_count,
  ).map_err(|_| LoaderError::InternalError)?;
  segment.set_user_can_write(true);
  segment.add_section(psp_section).map_err(|_| LoaderError::InternalError)?;
  segment.add_section(section).map_err(|_| LoaderError::InternalError)?;
  let mut segments = Vec::with_capacity(1);
  segments.push(segment);
  Ok(segments)
}

