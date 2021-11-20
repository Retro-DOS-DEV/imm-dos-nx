//! Parsing and loading for DOS MZ EXE files

use alloc::vec::Vec;
use crate::dos::execution::PSP;
use crate::files::{cursor::SeekMethod, handle::LocalHandle};
use crate::fs::{drive::DriveID, DRIVES};
use crate::memory::address::VirtualAddress;
use crate::task::memory::{ExecutionSection, ExecutionSegment};
use super::LoaderError;
use super::environment::{ExecutionEnvironment, InitialRegisters};

#[repr(C, packed)]
pub struct MZHeader {
  magic_number: [u8; 2],
  /// Number of bytes actually occupied in the final page
  last_page_size: u16,
  /// Number of 512B pages needed to contain this file
  page_count: u16,
  /// Number of entries in the relocation table
  relocation_entries: u16,
  /// Size of this header, in paragraphs (4 bytes)
  header_size_paragraphs: u16,
  /// Minimum number of paragraphs required for execution. This is used for
  /// uninitialized data that appears
  min_alloc_paragraphs: u16,
  /// Maximum number of paragraphs required for execution; this is the amount
  /// preferred by the program.
  max_alloc_paragraphs: u16,
  /// Initial value of the SS segment, added to the program's first segment
  initial_ss: u16,
  /// Initial value of the SP register
  initial_sp: u16,
  /// Data integrity checksum
  checksum: u16,
  /// Initial value of the IP register
  initial_ip: u16,
  /// Initial value of the CS segment, added to the program's first segment
  initial_cs: u16,
  /// Location of the relocation table, relative to the start of the file
  relocation_table_offset: u16,
  /// Overlay number (wut?)
  overlay_number: u16,
}

impl MZHeader {
  pub fn byte_length(&self) -> usize {
    if self.page_count == 0 {
      return 0;
    }
    (self.page_count as usize - 1) * 512 + (self.last_page_size as usize)
  }

  pub fn header_size_bytes(&self) -> usize {
    (self.header_size_paragraphs as usize) << 4
  }
}

pub fn build_environment(
  drive_id: DriveID,
  local_handle: LocalHandle,
) -> Result<ExecutionEnvironment, LoaderError> {
  let header = unsafe {
    let mut header: MZHeader = core::mem::zeroed::<MZHeader>();
    let header_slice = core::slice::from_raw_parts_mut(
      &mut header as *mut MZHeader as *mut u8,
      core::mem::size_of::<MZHeader>(),
    );

    let (_, instance) = DRIVES.get_drive_instance(&drive_id).ok_or(LoaderError::FileNotFound)?;
    let _ = instance.seek(local_handle, SeekMethod::Absolute(0)).map_err(|_| LoaderError::FileNotFound)?;
    let _ = instance.read(local_handle, header_slice).map_err(|_| LoaderError::FileNotFound)?;

    header
  };

  if header.page_count < 1 {
    return Err(LoaderError::InternalError);
  }
  let mut file_size = (header.page_count as usize - 1) * 512;
  file_size += if header.last_page_size == 0 {
    512
  } else {
    header.last_page_size as usize
  };

  let code_start = header.header_size_paragraphs as usize * 16;
  let exec_size = file_size - code_start;

  // segment location of the PSP
  let psp_segment: usize = 0x100;
  let psp_size = core::mem::size_of::<PSP>();
  let psp_size_paragraphs = psp_size / 16;
  // segment location of the "load module" aka the code copied from the EXE
  let load_module_segment = (psp_segment + psp_size_paragraphs) as u32;

  let segments = {
    let psp_address = VirtualAddress::new(psp_segment << 4);
    let load_module_address = psp_address + psp_size;
    let page_start = psp_address.prev_page_barrier();
    let psp_section = ExecutionSection {
      segment_offset: psp_address - page_start,
      executable_offset: None,
      size: psp_size,
    };
    let section = ExecutionSection {
      segment_offset: load_module_address - page_start,
      executable_offset: Some(code_start),
      size: exec_size,
    };

    let final_byte = load_module_address + exec_size;
    let total_length = final_byte - page_start;
    let mut page_count = total_length / 0x1000;
    if total_length & 0xfff != 0 {
      page_count += 1;
    }

    let mut segment = ExecutionSegment::at_address(
      page_start,
      page_count,
    ).map_err(|_| LoaderError::InternalError)?;
    segment.set_user_can_write(true);
    segment.add_section(psp_section).map_err(|_| LoaderError::InternalError)?;
    segment.add_section(section).map_err(|_| LoaderError::InternalError)?;
    let mut segments = Vec::with_capacity(1);
    segments.push(segment);
    segments
  };

  Ok(
    ExecutionEnvironment {
      segments,
      registers: InitialRegisters {
        /// Similar to COM, %eax should represent the validity of the
        /// pre-constructed FCBs.
        eax: Some(0),

        eip: Some(header.initial_ip as u32),
        esp: Some(header.initial_sp as u32),

        cs: Some(header.initial_cs as u32 + load_module_segment),
        ds: Some(psp_segment as u32),
        es: Some(psp_segment as u32),
        ss: Some(header.initial_ss as u32 + load_module_segment),
      },
      require_vm: true,
    }
  )
}