use crate::filesystems;
use crate::kprintln;
use crate::memory::{
  self,
  address::{VirtualAddress},
  physical,
  virt::{
    page_directory::{CurrentPageDirectory, PageDirectory, PermissionFlags},
    region::{MemoryRegionType, Permissions},
  },
};
use crate::process;
use crate::task;
use super::stack::StackFrame;
use super::syscall_legacy::{DosApiRegisters, dos_api, VM8086Frame};

#[no_mangle]
pub extern "x86-interrupt" fn divide_by_zero(stack_frame: &StackFrame) {
  kprintln!("\nERR: Divide By Zero\n{:?}", stack_frame);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn double_fault(stack_frame: &StackFrame) {
  //kprintln!("\nERR: Double Fault\n{:?}", stack_frame);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn gpf(stack_frame: &StackFrame, error: u32) {
  let reg_ptr: usize;
  unsafe {
    llvm_asm!("" : "={ebp}"(reg_ptr) ::: "volatile");
  }
  if stack_frame.eflags & 0x20000 != 0 {
    // VM 8086
    let stack_frame_ptr = stack_frame as *const StackFrame as usize;
    let vm_frame_ptr = (stack_frame_ptr + 12) as *mut VM8086Frame;
    unsafe {
      let regs = &mut *((reg_ptr - 6 * 4) as *mut DosApiRegisters);
      let vm_frame = &mut *vm_frame_ptr;
      let mut_stack_frame = &mut *(stack_frame_ptr as *mut StackFrame);
      let op_ptr = ((stack_frame.cs << 4) + stack_frame.eip) as *const u8;
      if *op_ptr == 0xcd {
        // INT
        match *op_ptr.offset(1) {
          0x21 => {
            // DOS API
            dos_api(regs, vm_frame);
          },
          _ => (),
        }
        mut_stack_frame.eip += 2;
        return;
      }
    }
  }

  kprintln!("\nERR: General Protection Fault, code {}", error);
  kprintln!("{:?}", stack_frame);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn page_fault(stack_frame: &StackFrame, error: u32) {
  let address: usize;
  unsafe {
    asm!(
      "mov {0}, cr2",
      out(reg) address,
    );
  }
  kprintln!("\nPage Fault at {:#010x} ({:x})", address, error);

  if address >= 0xc0000000 { // Kernel region
    if error & 4 == 4 {
      // Permission error - access attempt came from Ring 3
      // This should segfault the process
      kprintln!("Attempt to access kernel memory ({:#010x}) from userspace (IP {:#010x})", address, stack_frame.eip);
      loop {}
    }
    if error & 1 == 0 {
      // Page was not present
      // If it is in the heap region, map a new physical frame and extend the
      // region

    }
  } else { // User space
    if error & 1 == 0 {
      // Page was not present
      // Query the current task to determine
      let vaddr = VirtualAddress::new(address);
      let current_pagedir = CurrentPageDirectory::get();
      let current_process_lock = crate::task::switching::get_current_process();
      if crate::task::paging::page_on_demand(current_process_lock, vaddr) {
        unsafe { 
          llvm_asm!("1:
            jmp 1b");
        }
        return;
      }
    }

    // All other cases (accessing an unmapped section, writing a read-only
    // segment, etc) should cause a segfault.

    kprintln!("SEGFAULT");

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
