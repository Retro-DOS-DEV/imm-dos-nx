use crate::{klog, kprintln};
use crate::memory::{
  address::{VirtualAddress},
  virt::page_directory::{CurrentPageDirectory, invalidate_page},
};
use super::stack::StackFrame;

#[no_mangle]
pub extern "x86-interrupt" fn divide_by_zero(stack_frame: StackFrame) {
  kprintln!("\nERR: Divide By Zero\n{:?}", stack_frame);
  // Send a floating-point exception signal to the current process, and return
  // to execution.
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn breakpoint(_stack_frame: StackFrame) {
  // Send a Trap signal to the current process
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn invalid_opcode(stack_frame: StackFrame) {
  let eip = stack_frame.eip;
  let curid = crate::task::switching::get_current_id();
  kprintln!("Invalid opcode at {:#010x} ({:?})", eip, curid);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn double_fault(_stack_frame: StackFrame, _error: u32) {
  //kprintln!("\nERR: Double Fault\n{:?}", stack_frame);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn invalid_tss(_stack_frame: StackFrame, error: u32) {
  kprintln!("\nERR: Invalid TSS. Segment {:?}", error);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn segment_not_present(_stack_frame: StackFrame, error: u32) {
  kprintln!("\nERR: Segment not present: {:?}", error);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn stack_segment_fault(_stack_frame: StackFrame, error: u32) {
  kprintln!("\nERR: Stack segment fault: {:?}", error);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn gpf(stack_frame: StackFrame, error: u32) {
  if stack_frame.eflags & 0x20000 != 0 {
    // VM86 Mode
    if crate::dos::emulation::handle_gpf(&stack_frame) {
      return;
    }
    // else, fall through to the general GPF handler
  } else if stack_frame.eip >= 0xc0000000 {
    kprintln!("Kernel GPF: {}", error);
    loop {}
  }

  kprintln!("\nERR: General Protection Fault, code {}", error);
  kprintln!("{:?}", stack_frame);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn page_fault(stack_frame: StackFrame, error: u32) {
  let address: usize;
  unsafe {
    asm!(
      "mov {0}, cr2",
      out(reg) address,
    );
  }
  let curid = crate::task::switching::get_current_id();
  kprintln!("\nPage Fault ({:?}: {:#010X}) at {:#010x} ({:x})", curid, stack_frame.eip, address, error);

  if address >= 0xc0000000 { // Kernel region
    if error & 4 == 4 {
      // Permission error - access attempt came from Ring 3
      // This should segfault the process
      let eip = stack_frame.eip as usize;
      kprintln!("Attempt to access kernel memory ({:#010x}) from userspace (IP {:#010x})", address, eip);

      if eip == address && eip > 0xc0000000 && eip < 0xc0000010 {
        // Userspace attempted to return to an "IRQ marker"
        // This is our way of creating a simple developer experience for
        // userspace interrupt handlers -- all a program needs to do is return
        // to the fake calling address placed on its stack.
        let irq = eip - 0xc0000000;
        super::handlers::return_from_handler(irq);
      }

      loop {}
    }
    if error & 1 == 0 {
      // Page was not present
      // If it is in the heap region, map a new physical frame and extend the
      // region

      kprintln!("Attempted to reach unpaged kernel memory. Does heap need to be expanded?");
      loop {}
    }
  } else { // User space
    if stack_frame.eflags & 0x20000 != 0 {
      // VM86 mode, handle it separately
      if crate::dos::emulation::handle_page_fault(&stack_frame, address) {
        return;
      }
      kprintln!("Failed to handle page fault in DOS program");
      loop {}
    }

    if error & 1 == 0 {
      // Page was not present
      // Query the current task to determine how to fill the page
      let vaddr = VirtualAddress::new(address);
      let current_process_lock = crate::task::switching::get_current_process();
      if crate::task::paging::page_on_demand(current_process_lock, vaddr) {
        // Return back to the failed instruction
        return;
      }
    } else if error & 2 == 2 {
      // Write to a read-only page
      // Either this is a CoW modification, or a permissions violation
      // Load the page entry to determine which case should be handled
      let id = crate::task::switching::get_current_id();
      kprintln!("Write to page {:?}", id);

      let vaddr = VirtualAddress::new(address);
      let mut current_pagedir = CurrentPageDirectory::get();
      let page_table_entry = current_pagedir.get_table_entry_for(vaddr);
      if let Some(entry) = page_table_entry {
        //kprintln!("ENTRY: {:b}", entry.0);
        if entry.is_cow() {
          let new_count = crate::memory::physical::release_frame_at_address(entry.get_address());
          if new_count == 0 {
            // this was the only reference to the frame, simply mark it as readable
            //kprintln!("Entry is no longer marked COW");
            entry.clear_cow();
            entry.set_write_access();
            invalidate_page(vaddr);
            return;
          }
          kprintln!("Decrement COW, {} refs remaining", new_count);
          let page_start = vaddr.prev_page_barrier();
          let new_frame = crate::task::paging::duplicate_frame(page_start);

          kprintln!("COW: Replacing {:?} with {:?}", entry.get_address(), new_frame.get_address());

          entry.clear_cow();
          entry.set_address(new_frame.to_frame().get_address());
          entry.set_write_access();
          invalidate_page(page_start);

          return;
        }
      }
      kprintln!("No entry or cow");
    }

    // All other cases (accessing an unmapped section, writing a read-only
    // segment, etc) should cause a segfault.

    kprintln!("SEGFAULT AT IP: {:#010X} (Access {:#010X})", stack_frame.eip, address);

    loop {}
  }

  /*
  let current_proc = process::current_process().expect("Page fault outside a process");
  if address >= 0xc0000000 {
    // Kernel region
    if error & 4 == 4 {
      kprintln!("IP: {:#010x}", stack_frame.eip);
      // At ring 3
      kprintln!("Attempt to access kernel memory from userspace: {:#010x}", address);
      loop {}
    } else {
      if error & 1 == 0 {
        // Page was not present
        // If it is in the heap or stack regions, map a new physical frame and
        // extend the region

        let vaddr = VirtualAddress::new(address);
        let current_pagedir = CurrentPageDirectory::get();
        match current_proc.get_range_containing_address(vaddr) {
          Some(range) => {
            match range.backing_type() {
              MemoryRegionType::Anonymous(_) => {
                let kernel_frame = match memory::physical::allocate_frame() {
                  Ok(frame) => frame,
                  Err(_) => {
                    // Out of memory
                    // At some point we need to implement disk paging
                    panic!("Unable to allocate kernel memory");
                  },
                };
                current_pagedir.map(kernel_frame, VirtualAddress::new(address & 0xfffff000), PermissionFlags::empty());
                return;
              },
              MemoryRegionType::DMA(frame_range) => {
                let offset = (address & 0xfffff000) - range.get_starting_address_as_usize();
                let paddr = frame_range.get_starting_address().as_usize();
                let frame = physical::frame::Frame::new(paddr + offset);

                let page_start = VirtualAddress::new(address & 0xfffff000);
                let flags = PermissionFlags::empty();
                current_pagedir.map(frame, page_start, flags);
                return;
              },
              _ => (),
            }
          },
          None => (),
        }
        return;
      }
    }
  } else {
    // User region
    //kprintln!("Page fault in user region.");
    let vaddr = VirtualAddress::new(address);
    let current_pagedir = CurrentPageDirectory::get();
    match current_proc.get_range_containing_address(vaddr) {
      Some(range) => {
        // Three scenarios we need to support:
        //  - Attempted to read/write an unmapped code page
        //  - Attempted to write a Copy-on-Write page
        //  - Expanding the stack / heap
        if error & 1 == 0 {
          // Page not present
          match range.backing_type() {
            MemoryRegionType::Direct(frame_range) => {
              let offset = (address & 0xfffff000) - range.get_starting_address_as_usize();
              let paddr = frame_range.get_starting_address().as_usize();
              let frame = physical::frame::Frame::new(paddr + offset);
              
              let page_start = VirtualAddress::new(address & 0xfffff000);
              let flags = PermissionFlags::new(PermissionFlags::USER_ACCESS | PermissionFlags::WRITE_ACCESS);
              current_pagedir.map(frame, page_start, flags);
              return;
            },
            _ => (),
          }

          // Page not present
          let new_frame = match memory::physical::allocate_frame() {
            Ok(frame) => frame,
            Err(_) => {
              // Out of memory
              panic!("Unable to allocate userspace memory");
            },
          };
          let page_start = VirtualAddress::new(address & 0xfffff000);
          let flags = PermissionFlags::new(PermissionFlags::USER_ACCESS | PermissionFlags::WRITE_ACCESS);
          current_pagedir.map(new_frame, page_start, flags);
          if range.get_permissions() == Permissions::CopyOnWrite {
            physical::reference_frame_at_address(new_frame.get_address());
          }
          // should zero out the new_frame here, now that it's mapped

          if let MemoryRegionType::MemMapped(drive, handle, length) = range.backing_type() {
            let offset = page_start.as_usize() - range.get_starting_address_as_usize();
            let mut read_len = 0x1000;
            if length < offset {
              read_len = 0;
            } else if offset + length < 0x1000 {
              read_len = length;
            }
            let fs = filesystems::get_fs(drive).expect("Memmapped to invalid fileseystem");
            let buffer = unsafe {
              core::slice::from_raw_parts_mut(page_start.as_usize() as *mut u8, read_len)
            };
            fs.read(handle, buffer).expect("Error reading from memmapped file");
          }

          // If the range needs to be extended and has extension enabled, do so
          // ...

          return;
        } else if error & 2 == 2 {
          // Write attempted on a mapped page
          kprintln!("Copy on Write not implemented yet");
        }
      },
      None => (),
    }
  }

  /*  
  if error & 1 == 0 {
    kprintln!("  PAGE NOT PRESENT");
  }
  if error & 2 == 2 {
    kprintln!("  WRITE ATTEMPTED");
  } else {
    kprintln!("  READ ATTEMPTED");
  }
  if error & 4 == 4 {
    kprintln!("  AT RING 3");
  }
  if error & 16 == 16 {
    kprintln!("  INSTRUCTION FETCH");
  }
  */
  kprintln!("Failed to map address: {:#101x}", address);
  kprintln!("{:?}", stack_frame);
  loop {}
  */
}
