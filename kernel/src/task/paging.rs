use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::files::cursor::SeekMethod;
use crate::fs::DRIVES;
use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::memory::physical::frame::Frame;
use crate::memory::virt::page_directory::{self, AlternatePageDirectory, PageDirectory, PermissionFlags};
use crate::memory::virt::page_table::PageTable;
use spin::RwLock;
use super::memory::{USER_KERNEL_BARRIER, MMapBacking, MMapRegion};
use super::process::Process;

pub fn page_on_demand(lock: Arc<RwLock<Process>>, address: VirtualAddress) -> bool {
  let stack_size = 0x2000;
  let stack_range = VirtualAddress::new(USER_KERNEL_BARRIER - stack_size)..VirtualAddress::new(USER_KERNEL_BARRIER);

  let heap_range = lock.read().memory.get_heap_address_range();
  
  if heap_range.contains(&address) || stack_range.contains(&address) {
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
    let buffer = unsafe { core::slice::from_raw_parts_mut(address.prev_page_barrier().as_usize() as *mut u32, 0x400) };
    for i in 0..0x400 {
      buffer[i] = 0;
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
            segment.get_starting_address() + section.segment_offset,
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
          //crate::kprintln!("FILL FROM FILE: {:?} {:X}", section.0, section.1);
          
          // should really do something with these potential errors
          let _ = drive_instance.seek(exec_file_info.1, SeekMethod::Absolute(offset));
          let _ = drive_instance.read(exec_file_info.1, buffer);
        },
        None => {
          // Fill with zeroes
          //crate::kprintln!("FILL WITH ZEROES: {:?} {:X}", section.0, section.1);
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

pub fn get_or_allocate_physical_address(addr: VirtualAddress) -> Result<PhysicalAddress, ()> {
  if !addr.is_page_aligned() {
    return Err(());
  }
  if addr < VirtualAddress::new(0xc0000000) {
    // not supporting user space yet
    return Err(());
  }
  let current_pagedir = page_directory::CurrentPageDirectory::get();
  match current_pagedir.get_physical_address(addr) {
    Some(phys) => return Ok(phys),
    None => (),
  }
  // Not currently mapped
  {
    let kernel_mem = super::memory::KERNEL_MEMORY.read();
    let mapping = kernel_mem.get_mapping_containing_address(&addr);
    let new_frame = match mapping {
      Some(map) => get_frame_for_region(map).ok_or(()),
      None => Err(()),
    }?;
    let start = new_frame.get_address();
    current_pagedir.map(
      new_frame,
      addr.prev_page_barrier(),
      PermissionFlags::empty(),
    );
    Ok(start)
  }
}

pub fn get_frame_for_region(region: &MMapRegion) -> Option<Frame> {
  match region.backed_by {
    MMapBacking::Anonymous => {
      crate::memory::physical::allocate_frame().ok()
    },
    MMapBacking::DMA => {
      // TODO: needs to be in lower 16MB
      crate::memory::physical::allocate_frame().ok()
    },
    // need to be built
    _ => panic!("Unsupported physical backing"),
  }
}

pub fn share_kernel_page_directory(vaddr: VirtualAddress) {
  let dir_index = vaddr.get_page_directory_index();
  let top_page = PageTable::at_address(VirtualAddress::new(0xfffff000));
  let entry = top_page.get(dir_index);
  let frame_address = entry.get_address();

  super::switching::for_each_process(|p| {
    let dir_address = p.read().page_directory.get_address();
    let alt = AlternatePageDirectory::new(dir_address);
    alt.add_directory_frame(dir_index, frame_address);
  });
}
