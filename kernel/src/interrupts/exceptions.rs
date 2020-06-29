use crate::kprintln;
use crate::memory::{
  self,
  address::{VirtualAddress},
  virt::{page_directory::{CurrentPageDirectory, PageDirectory, PermissionFlags}},
};
use crate::process;
use super::stack::StackFrame;

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
  kprintln!("\nERR: General Protection Fault, code {}", error);
  kprintln!("{:?}", stack_frame);
  loop {}
}

#[no_mangle]
pub extern "x86-interrupt" fn page_fault(stack_frame: &StackFrame, error: u32) {
  let address: usize;
  unsafe {
    llvm_asm!("mov $0, cr2" : "=r"(address) : : : "intel", "volatile");
  }
  kprintln!("\nPage Fault at {:#010x} {:x}:", address, error);
  let current_proc = process::current_process().expect("Page fault outside a process");
  kprintln!("Current proc: {:?}", current_proc.get_id());
  if address >= 0xc0000000 {
    // Kernel region
    if error & 4 == 4 {
      // At ring 3
      kprintln!("Attempt to access kernel memory from userspace: {:#010x}", address);
      loop {}
    } else {
      if error & 1 == 0 {
        // Page was not present
        // If it is in the heap or stack regions, map a new physical frame and
        // extend the region

        /*
        let vaddr = VirtualAddress::new(address);
        let current_pagedir = CurrentPageDirectory::get();
        if current_proc.get_kernel_heap_region().read().contains_address(vaddr) {
          let heap_frame = match memory::physical::allocate_frame() {
            Ok(frame) => frame,
            Err(_) => {
              // Out of memory
              // At some point we need to implement disk paging
              panic!("Unable to allocate kernel heap memory");
            },
          };
          current_pagedir.map(heap_frame, VirtualAddress::new(address & 0xfffff000), PermissionFlags::empty());
          return;
        }

        if current_proc.get_kernel_stack_region().read().contains_address(vaddr) {
          let stack_frame = match memory::physical::allocate_frame() {
            Ok(frame) => frame,
            Err(_) => {
              // Out of memory
              panic!("Unable to allocate kernel stack memory");
            },
          };
          let stack_page_start = VirtualAddress::new(address & 0xfffff000);
          current_pagedir.map(stack_frame, stack_page_start, PermissionFlags::empty());
          let stack_start = current_proc.get_kernel_stack_region().read().get_starting_address_as_usize();
          if stack_start == stack_page_start.as_usize() {
            // Extend the stack by a frame
            kprintln!("Extending stack by one frame");
            current_proc.get_kernel_stack_region().write().extend_before(1);
          }
          return;
        }
        */

        let vaddr = VirtualAddress::new(address);
        let current_pagedir = CurrentPageDirectory::get();
        match current_proc.get_range_containing_address(vaddr) {
          Some(range) => {
            let kernel_frame = match memory::physical::allocate_frame() {
              Ok(frame) => frame,
              Err(_) => {
                // Out of memory
                // At some point we need to implement disk paging
                panic!("Unable to allocate kernel memory");
              },
            };
            current_pagedir.map(kernel_frame, VirtualAddress::new(address & 0xfffff000), PermissionFlags::empty());
          },
          None => (),
        }
        return;
      }
    }
  } else {
    // User region
    kprintln!("Page fault in user region.");
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
          // If the range needs to be extended and has extension enabled, do so
          // ...
          kprintln!("Mapped new page");
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
  kprintln!("{:?}", stack_frame);
  loop {}
}
