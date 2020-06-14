#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(llvm_asm)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]

#![no_std]

// Test-safe modules
pub mod memory;

#[cfg(not(test))]
pub mod debug;
#[cfg(not(test))]
pub mod devices;
#[cfg(not(test))]
pub mod drivers;
#[cfg(not(test))]
pub mod files;
#[cfg(not(test))]
pub mod filesystems;
#[cfg(not(test))]
pub mod gdt;
#[cfg(not(test))]
pub mod hardware;
#[cfg(not(test))]
pub mod idt;
#[cfg(not(test))]
pub mod init;
#[cfg(not(test))]
pub mod interrupts;
#[cfg(not(test))]
pub mod panic;
#[cfg(not(test))]
pub mod process;
#[cfg(not(test))]
pub mod syscalls;
#[cfg(not(test))]
pub mod time;
#[cfg(not(test))]
pub mod x86;

use memory::address::PhysicalAddress;

extern crate alloc;

#[cfg(not(test))]
extern {
  // Segment labels from the linker script
  // Makes it easy to mark pages as readable / writable
  #[link_name = "__ro_physical_start"]
  static label_ro_physical_start: u8;
  #[link_name = "__ro_physical_end"]
  static label_ro_physical_end: u8;
  #[link_name = "__rw_physical_start"]
  static label_rw_physical_start: u8;
  #[link_name = "__rw_physical_end"]
  static label_rw_physical_end: u8;
  #[link_name = "__bss_start"]
  static label_bss_start: u8;
  #[link_name = "__bss_end"]
  static label_bss_end: u8;
  #[link_name = "__stack_start"]
  static label_stack_start: u8;
}

/**
 * Clear the .bss section. Since we copied bytes from disk to memory, there's a
 * chance it contains the symbol table.
 */
#[cfg(not(test))]
unsafe fn zero_bss() {
  let mut bss_iter = &label_bss_start as *const u8 as usize;
  let bss_end = &label_bss_end as *const u8 as usize;
  while bss_iter < bss_end {
    let bss_ptr = bss_iter as *mut u8;
    *bss_ptr = 0;
    bss_iter += 1;
  }
}

#[cfg(not(test))]
unsafe fn init_memory() {
  let kernel_start = PhysicalAddress::new(&label_ro_physical_start as *const u8 as usize);
  let kernel_end = PhysicalAddress::new(&label_rw_physical_end as *const u8 as usize);
  memory::init(kernel_start, kernel_end);

  memory::init_paging();
  let stack_start = PhysicalAddress::new(&label_stack_start as *const u8 as usize);
  memory::move_kernel_stack(memory::frame::Frame::containing_address(stack_start));
}

#[cfg(not(test))]
unsafe fn init_tables() {
  idt::init();
  gdt::init();
}

/**
 * Entry point of the kernel
 */
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
  unsafe {
    zero_bss();
    init_memory();
    init_tables();
  }

  //assert_eq!(1, 2);

  unsafe {
    kprintln!("\nEntering the Kernel...");

    let mem_map = memory::map::load_entries_at_address(0x1000);
    kprintln!("Memory Map:");
    for entry in mem_map {
      kprintln!("{:?}", entry);
    }

    kprint!("Frame Table: ----------------------------------");
    for i in 0..memory::FRAME_ALLOCATOR.length {
      if i & 15 == 0 {
        kprintln!();
      }
      let ptr = memory::FRAME_ALLOCATOR.start as *const u8;
      kprint!("{:02x} ", *(ptr.offset(i as isize)));
    }
    kprintln!("\nTotal Memory: {} KiB\nFree Memory: {} KiB", memory::count_frames() * 4, memory::count_free_frames() * 4);

    // Initialize kernel heap
    {
      let heap_start = memory::address::VirtualAddress::new(0xc0400000);
      let heap_size_frames = 256;
      for i in 0..heap_size_frames {
        let heap_frame = memory::allocate_physical_frame().unwrap();
        let heap_page = memory::address::VirtualAddress::new(0xc0400000 + i * 4096);
        memory::paging::map_address_to_frame(heap_page, heap_frame);
      }
      let heap_size = heap_size_frames * 4096;
      kprintln!("Kernel heap at {:?}-{:?}", heap_start, memory::address::VirtualAddress::new(0xc0400000 + heap_size));
      memory::heap::init_allocator(heap_start, heap_size);
      kprintln!("Kernel initialized");
    }

    // Initialize hardware
    devices::init();

    filesystems::init_fs();

    process::init();
    let init_process = process::all_processes_mut().spawn_process();
    process::make_current(init_process);

    llvm_asm!("sti");

    let result = syscall::debug();
    kprintln!("returned from syscall, got {}", result);

    // pretend to read a file
    let handle = syscall::open("DEV:\\ZERO");
    assert_eq!(handle, 0);

    let mut buffer: [u8; 1] = [0xff];
    syscall::read(handle, buffer.as_mut_ptr(), 1);
    assert_eq!(buffer[0], 0);
  }

  let com1 = syscall::open("DEV:\\COM1");
  let msg = "HI SERIAL PORT";
  syscall::write(com1, msg.as_ptr(), msg.len());
  let mut buffer: [u8; 1] = [0];
  loop {
    let read = syscall::read(com1, buffer.as_mut_ptr(), 1);
    if read > 0 {
      kprint!("{}",
        unsafe { core::str::from_utf8_unchecked(&buffer) }
      );
    }
  }

  loop {
    unsafe {
      llvm_asm!("hlt" : : : : "volatile");
    }
  }
}