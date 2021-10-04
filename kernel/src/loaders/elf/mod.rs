use alloc::vec::Vec;
use crate::files::{cursor::SeekMethod, handle::LocalHandle};
use crate::fs::{DRIVES, drive::{DriveID}};
use crate::memory::address::VirtualAddress;
use crate::task::memory::{ExecutionSection, ExecutionSegment};
use super::LoaderError;
use super::environment::{ExecutionEnvironment, InitialRegisters};

pub mod read;
pub mod tables;

pub fn build_environment(
  drive_id: DriveID,
  local_handle: LocalHandle,
) -> Result<ExecutionEnvironment, LoaderError> {

  let (header, program_headers, section_headers) = read::load_tables(drive_id, local_handle)?;

  let mut segments: Vec<ExecutionSegment> = program_headers.iter().map(|program_header| {
    if program_header.segment_type != tables::SEGMENT_TYPE_LOAD {
      return None;
    }
    let segment_start = VirtualAddress::new(program_header.segment_virtual_address as usize);
    let segment_end = segment_start + program_header.segment_size_in_memory as usize;
    let address = segment_start.prev_page_barrier();
    let page_count = (segment_end.next_page_barrier() - address) / 4096;
    let mut segment = ExecutionSegment::at_address(address, page_count).ok()?;
    segment.set_user_can_write(program_header.segment_flags & 2 == 2);

    Some(segment)
  })
  .filter_map(|e| e)
  .collect();

  for section_header in section_headers.iter() {
    let start = VirtualAddress::new(section_header.section_virtual_address as usize);
    for segment in segments.iter_mut() {
      // only allocate memory for sections marked ALLOC
      if section_header.section_flags & tables::SECTION_FLAG_ALLOC == 0 {
        continue;
      }

      let segment_start = segment.get_starting_address();
      let segment_end = segment_start + segment.get_size();
      if (segment_start..segment_end).contains(&start) {
        let offset = match section_header.section_type {
          tables::SECTION_TYPE_NOBITS => None,
          _ => Some(section_header.section_file_offset as usize),
        };
        let section = ExecutionSection {
          segment_offset: start - segment_start,
          executable_offset: offset,
          size: section_header.section_size_in_file as usize,
        };

        //crate::kprintln!("S: {:?} + {:X} ({:X}), from {:?}", segment_start, start - segment_start, section_header.section_size_in_file, offset);

        match segment.add_section(section) {
          Ok(_) => (),
          Err(_) => return Err(LoaderError::InternalError),
        }
        break;
      }
    }
  }

  let env = ExecutionEnvironment {
    segments,
    registers: InitialRegisters {
      eax: None,
      eip: Some(header.entry_point),
      esp: Some(0xbffffffc),
      cs: None,
      ds: None,
      es: None,
      ss: None,
    },
    require_vm: false,
  };

  return Ok(env);
}