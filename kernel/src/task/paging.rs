use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::files::cursor::SeekMethod;
use crate::fs::DRIVES;
use crate::memory::address::VirtualAddress;
use crate::memory::virt::page_directory::{self, PageDirectory, PermissionFlags};
use spin::RwLock;
use super::process::Process;

pub fn page_on_demand(lock: Arc<RwLock<Process>>, address: VirtualAddress) -> bool {
  let heap_range = lock.read().memory.get_heap_address_range();
  if heap_range.contains(&address) {
    // allocate a new frame for the heap
    let new_frame = match crate::memory::physical::allocate_frame() {
      Ok(frame) => frame,
      Err(_) => return false,
    };
    let current_pagedir = page_directory::CurrentPageDirectory::get();
    current_pagedir.map(
      new_frame,
      address.prev_page_barrier(),
      PermissionFlags::new(PermissionFlags::USER_ACCESS | PermissionFlags::WRITE_ACCESS),
    );
    // zero the page
    for i in 0..0x400 {
      unsafe {
        let buffer = core::slice::from_raw_parts_mut(address.prev_page_barrier().as_usize() as *mut u32, 0x400);
        buffer[i] = 0;
      }
    }
    return true;
  }

  let mut subsections = Vec::new();
  let mut flags = PermissionFlags::new(PermissionFlags::USER_ACCESS);
  let exec_file_info = {
    let process = lock.read();
    match process.memory.get_execution_segment_containing_address(&address) {
      Some(segment) => {
        let start_offset = address.prev_page_barrier() - segment.get_starting_address();
        let end_offset = start_offset + 0x1000;
        let clipped = segment.sections_iter().map(|s| s.clip_to(start_offset..end_offset)).filter(|s| !s.is_empty());
        for section in clipped {
          subsections.push((
            address.prev_page_barrier() + section.segment_offset,
            section.size,
            section.executable_offset,
          ));
        }
        if segment.user_can_write() {
          flags = PermissionFlags::new(PermissionFlags::USER_ACCESS | PermissionFlags::WRITE_ACCESS);
        }
      },
      None => (),
    }
    match process.get_exec_file() {
      Some(pair) => pair,
      None => return false, // No open executable
    }
  };

  if subsections.len() > 0 {
    let new_frame = match crate::memory::physical::allocate_frame() {
      Ok(frame) => frame,
      Err(_) => return false,
    };
    let current_pagedir = page_directory::CurrentPageDirectory::get();
    current_pagedir.map(
      new_frame,
      address.prev_page_barrier(),
      flags,
    );

    // copy all sections from file to the page
    let drive_instance = match DRIVES.get_drive_instance(&exec_file_info.0) {
      Some((_, instance)) => instance,
      None => return false,
    };
    for section in subsections.iter() {
      let buffer = unsafe {
        let start = section.0;
        let size = section.1;
        core::slice::from_raw_parts_mut(start.as_usize() as *mut u8, size)
      };
      match section.2 {
        Some(offset) => {
          // should really do something with these potential errors
          let _ = drive_instance.seek(exec_file_info.1, SeekMethod::Absolute(0));
          let _ = drive_instance.read(exec_file_info.1, buffer);
        },
        None => {
          // Fill with zeroes
          for i in 0..buffer.len() {
            buffer[i] = 0;
          }
        },
      }
    }
    return true;
  }

  false
}