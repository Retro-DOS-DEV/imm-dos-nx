//! Parsing and loading for static binaries with IMM-DOS syscalls
//! There isn't really any parsing or loading to do here. We just export a
//! mapping that copies the file to 0x0000, and reserves enough pages.

use alloc::vec::Vec;
use crate::files::handle::LocalHandle;
use crate::fs::{DRIVES, drive::{DriveID}};
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
  // A BIN file just has one segment and one section
  let segments = build_single_section_environment(file_size, 0)?;

  Ok(
    ExecutionEnvironment {
      segments,
      registers: InitialRegisters {
        eax: Some(0),

        eip: Some(0),
        esp: Some(0xbffffffc),

        cs: Some(0x1b),
        ds: None,
        es: None,
        ss: Some(0x23),
      },
      require_vm: false,
    }
  )
}

pub fn build_single_section_environment(
  file_size: usize,
  offset: usize,
) -> Result<Vec<ExecutionSegment>, LoaderError> {
  let section = ExecutionSection {
    segment_offset: offset,
    executable_offset: Some(0),
    size: file_size,
  };
  let mut page_count = file_size / 0x1000;
  if file_size & 0xfff != 0 {
    page_count += 1;
  }
  let mut segment = ExecutionSegment::at_address(VirtualAddress::new(0), page_count).map_err(|_| LoaderError::InternalError)?;
  segment.set_user_can_write(true);
  segment.add_section(section).map_err(|_| LoaderError::InternalError)?;
  let mut segments = Vec::with_capacity(1);
  segments.push(segment);
  Ok(segments)
}
