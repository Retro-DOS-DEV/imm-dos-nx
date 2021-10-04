use alloc::vec::Vec;
use crate::files::{cursor::SeekMethod, handle::LocalHandle};
use crate::fs::{DRIVES, drive::DriveID};
use super::super::LoaderError;
use super::tables::{Header, ProgramHeader, SectionHeader};

pub fn load_tables(
  drive_id: DriveID,
  local_handle: LocalHandle,
) -> Result<(Header, Vec<ProgramHeader>, Vec<SectionHeader>), LoaderError> {
  unsafe {
    let mut header: Header = core::mem::zeroed::<Header>();
    let header_slice = core::slice::from_raw_parts_mut(
      &mut header as *mut Header as *mut u8,
      core::mem::size_of::<Header>(),
    );

    let (_, instance) = DRIVES.get_drive_instance(&drive_id).ok_or(LoaderError::FileNotFound)?;
    let _ = instance.seek(local_handle, SeekMethod::Absolute(0)).map_err(|_| LoaderError::FileNotFound)?;
    let _ = instance.read(local_handle, header_slice).map_err(|_| LoaderError::FileNotFound)?;

    let mut program_table: Vec<ProgramHeader> = Vec::with_capacity(header.program_header_table_count as usize);
    let mut section_table: Vec<SectionHeader> = Vec::with_capacity(header.section_header_table_count as usize);

    {
      let _ = instance.seek(local_handle, SeekMethod::Absolute(header.program_header_table_offset as usize)).map_err(|_| LoaderError::FileNotFound)?;
      for _ in 0..header.program_header_table_count {
        let mut entry: ProgramHeader = core::mem::zeroed::<ProgramHeader>();
        let entry_slice = core::slice::from_raw_parts_mut(
          &mut entry as *mut ProgramHeader as *mut u8,
          core::mem::size_of::<ProgramHeader>(),
        );
        let _ = instance.read(local_handle, entry_slice).map_err(|_| LoaderError::FileNotFound)?;
        program_table.push(entry);
      }
    }

    {
      let _ = instance.seek(local_handle, SeekMethod::Absolute(header.section_header_table_offset as usize)).map_err(|_| LoaderError::FileNotFound)?;
      for _ in 0..header.section_header_table_count {
        let mut entry: SectionHeader = core::mem::zeroed::<SectionHeader>();
        let entry_slice = core::slice::from_raw_parts_mut(
          &mut entry as *mut SectionHeader as *mut u8,
          core::mem::size_of::<SectionHeader>(),
        );
        let _ = instance.read(local_handle, entry_slice).map_err(|_| LoaderError::FileNotFound)?;
        section_table.push(entry);
      }
    }

    Ok((header, program_table, section_table))
  }
}