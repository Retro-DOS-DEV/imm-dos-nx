use alloc::vec::Vec;
use crate::files::{cursor::SeekMethod, handle::LocalHandle};
use crate::fs::{DRIVES, drive::{DriveID}};
use crate::memory::address::VirtualAddress;
use crate::task::memory::{ExecutionSection, ExecutionSegment};
use super::LoaderError;
use super::environment::{ExecutionEnvironment, InitialRegisters};

pub mod tables;

pub fn build_environment(
  drive_id: DriveID,
  local_handle: LocalHandle,
) -> Result<ExecutionEnvironment, LoaderError> {
  unsafe {
    let mut header: tables::Header = core::mem::zeroed::<tables::Header>();
    let header_slice = core::slice::from_raw_parts_mut(
      &mut header as *mut tables::Header as *mut u8,
      core::mem::size_of::<tables::Header>(),
    );

    let (_, instance) = DRIVES.get_drive_instance(&drive_id).ok_or(LoaderError::FileNotFound)?;
    let _ = instance.seek(local_handle, SeekMethod::Absolute(0)).map_err(|_| LoaderError::FileNotFound)?;
    let _ = instance.read(local_handle, header_slice).map_err(|_| LoaderError::FileNotFound)?;

    crate::klog!("ELF File. Program Start: {:x}, program table: {:X}, section table: {:X}", header.entry_point, header.program_header_table_offset, header.section_header_table_offset);
  }

  return Err(LoaderError::InternalError);
}