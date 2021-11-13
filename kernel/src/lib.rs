#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(const_btree_new)]
#![feature(const_fn_trait_bound)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]

#![no_std]

// Test-safe modules
pub mod buffers;
pub mod collections;
pub mod dos;
pub mod files;
pub mod fs;
pub mod hardware;
pub mod input;
pub mod interrupts;
pub mod loaders;
pub mod memory;
//pub mod pipes;
pub mod promise;
pub mod task;
pub mod time;
pub mod vterm;
pub mod x86;

#[cfg(not(test))]
pub mod debug;
#[cfg(not(test))]
pub mod devices;
#[cfg(not(test))]
pub mod gdt;
#[cfg(not(test))]
pub mod init;
#[cfg(not(test))]
pub mod panic;
#[cfg(not(test))]
pub mod syscalls;
#[cfg(not(test))]
pub mod tty;

extern crate alloc;

use memory::address::VirtualAddress;

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

/// Used to pass data from the bootloader to the kernel
#[repr(C, packed)]
pub struct BootStruct {
  initfs_start: usize,
  initfs_size: usize,
}


/// Clear the .bss section. Since we copied bytes from disk to memory, there's a
/// chance it contains the symbol table.
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

/// Initialize the GDT and IDT, necessary for kernel execution
#[cfg(not(test))]
unsafe fn init_tables() {
  interrupts::idt::init();
  gdt::init();
}

/// Initialize system memory, enabling virtual memory and page tables.
/// Once virtual memory has been enabled, all references to kernel addresses
/// need to be or-ed with 0xc0000000 so that they can correctly point to the
/// kernel in all processes.
#[cfg(not(test))]
unsafe fn init_memory() {
  use memory::address::PhysicalAddress;

  let allocator_location = &label_rw_physical_end as *const u8 as usize;
  memory::physical::init_allocator(allocator_location, 0x1000);

  let stack_start_address = PhysicalAddress::new(&label_stack_start as *const u8 as usize);
  let kernel_data_bounds = memory::virt::KernelDataBounds {
    ro_start: PhysicalAddress::new(&label_ro_physical_start as *const u8 as usize),
    rw_end: PhysicalAddress::new(&label_rw_physical_end as *const u8 as usize),
    stack_start: stack_start_address,
  };

  let initial_pagedir = memory::virt::create_initial_pagedir();
  memory::virt::map_kernel(initial_pagedir, &kernel_data_bounds);
  initial_pagedir.make_active();
  memory::virt::enable_paging();

  memory::physical::move_allocator_reference_to_highmem();

  // move esp to the higher page, maintaining its relative location in the frame
  asm!(
    "mov {tmp}, esp
    sub {tmp}, {offset}
    add {tmp}, {stack_base}
    mov esp, {tmp}",
    offset = in(reg) stack_start_address.as_u32(),
    tmp = out(reg) _,
    stack_base = const task::stack::FIRST_STACK_TOP_PAGE,
  );

  memory::high_jump();
  // CLOWNTOWN: ebx points to the PLT, and is initialized based on a relative
  // position to the starting instruction pointer. Since we start in lowmem and
  // then jump to highmem, it gets set to a location in user memory space, which
  // breaks everything the first time a user process unmaps the low copy of the
  // kernel.
  // If we had a bootloader that initialized a page table and mapped us into
  // highmem before entering the kernel, this wouldn't be necessary...
  asm!("or ebx, 0xc0000000");

  kprintln!("\nKernel range: {:?}-{:?}", kernel_data_bounds.ro_start, kernel_data_bounds.rw_end);
}

/// Entry point of the kernel.
/// The bootloader jumps here, passing some useful information from BIOS.
/// To initialize, the kernel sets up memory and key tables, a heap for
/// allocation, and the initial task hierarchy.
/// It starts core processes, including the init process, before jumping into
/// an infinite idle loop that will be used when no tasks are running.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start(boot_struct_ptr: *const BootStruct) -> ! {
  let initfs_start = unsafe {
    let boot_struct = &*boot_struct_ptr;
    boot_struct.initfs_start
  } | 0xc0000000;

  unsafe {
    zero_bss();
    init_memory();
    init_tables();
  }

  {
    kprintln!("\nEntering the Kernel...");

    kprintln!(
      "\nTotal Memory: {} KiB\nFree Memory: {} KiB",
      memory::physical::get_frame_count() * 4,
      memory::physical::get_free_frame_count() * 4,
    );

    let heap_start = VirtualAddress::new(0xc0400000);
    {
      let heap_size_frames = memory::heap::INITIAL_HEAP_SIZE;
      memory::heap::map_allocator(heap_start, heap_size_frames);
      let heap_size = heap_size_frames * 0x1000;
      kprintln!("Kernel heap at {:?}-{:?}", heap_start, VirtualAddress::new(0xc0400000 + heap_size));
      memory::heap::init_allocator(heap_start, heap_size);
    }
    memory::physical::init_refcount();

    // This context will become the idle task, and halt in a loop until other
    // processes are ready
    task::switching::initialize();
    // Next, we spawn all the processes necessary for running the system.
    // The init process loads the userspace init program, which in turn spawns
    // most of the system daemons.
    {
      let init_process = task::switching::kfork(run_init);
      let input_process = task::switching::kfork(input::run_input);
      task::switching::kfork(hardware::vga::driver::vga_driver_process);
      task::switching::kfork(tty::ttys_process);
    }

    fs::init_system_drives(VirtualAddress::new(initfs_start));
  }

  loop {
    unsafe {
      asm!("cli");
      task::yield_coop();
      asm!(
        "sti
        hlt"
      );
    }
  }
}

/// Load the init process, which will spawn all other system and user processes.
#[cfg(not(test))]
#[inline(never)]
pub extern fn run_init() {
  // Initialize hardware
  tty::init_ttys();
  vterm::init_vterm();
  devices::init();
  time::system::initialize_from_rtc();

  let current_time = time::system::get_system_time().to_timestamp().to_datetime();
  crate::klog!("System Time: \x1b[94m{:} {:}\x1b[m\n", current_time.date, current_time.time);

  //let r = task::exec::exec("INIT:\\driver.bin", loaders::InterpretationMode::Native);
  //let stdin = task::io::open_path("DEV:\\TTY1").unwrap();
  //let stdout = task::io::open_path("DEV:\\TTY1").unwrap();
  //let stderr = task::io::dup(stdout, None).unwrap();

  /*
  let r = task::exec::exec("INIT:\\dosio.com", loaders::InterpretationMode::DOS);
  if let Err(_) = r {
    kprintln!("Failed to run init process");
    loop {}
  }
  */

  let session = tty::begin_session(1, "INIT:\\command.elf");
  if let Err(_) = session {
    kprintln!("Failed to initialize shell");
    loop {
      task::yield_coop();
    }
  }
}

#[inline(never)]
pub extern fn user_init() {
  /*
  let tty0 = syscall::open("DEV:\\TTY0");
  syscall::write_str(tty0, "Initializing devices...\n");
  let pid = syscall::get_pid() as u8;
  let mut pidmsg: [u8; 7] = [b'P', b'I', b'D', b':', b' ', b' ', b'\n'];
  unsafe {
    pidmsg[5] = pid + 48;
  }
  syscall::write(tty0, pidmsg.as_ptr(), pidmsg.len());
  syscall::raise(syscall::signals::STOP);
  syscall::yield_coop();

  syscall::write_str(tty0, "System ready.\n");

  let mut entry = syscall::files::DirEntryInfo::empty();
  syscall::write_str(tty0, "Root Directory Contents:\n");
  let dir_handle = syscall::open_dir("A:\\");
  let mut dir_index = 0;
  loop {
    syscall::read_dir(dir_handle, dir_index, &mut entry as *mut syscall::files::DirEntryInfo);
    dir_index += 1;
    if entry.is_empty() {
      break;
    }
    syscall::write_str(tty0, "  ");
    syscall::write(tty0, entry.file_name.as_ptr(), entry.file_name.len());
    syscall::write_str(tty0, " ");
    syscall::write(tty0, entry.file_ext.as_ptr(), entry.file_ext.len());
    syscall::write_str(tty0, "\n");
  }
  syscall::write_str(tty0, "DONE");

  let file_handle = syscall::open("A:\\BOOT.BIN");
  */

  /*
  let read_write: [u32; 2] = [0; 2];
  let _ = syscall::pipe(&read_write);
  let read = read_write[0];
  let write = read_write[1];
  assert_eq!(read, 1);
  assert_eq!(write, 2);

  let pid = syscall::fork();
  if pid == 0 {
    let child_msg = "CHILD MSG";
    syscall::write(write, child_msg.as_ptr(), child_msg.len());
    syscall::exit(0);
  } else {
    syscall::sleep(1000);
    let mut bytes_available: u32 = 0;
    assert_eq!(syscall::ioctl(read, syscall::flags::FIONREAD, &bytes_available as *const u32 as u32), 0);
    assert_eq!(bytes_available, 9);
    let msg = "Got message: ";
    syscall::write(com1, msg.as_ptr(), msg.len());
    let mut buffer: [u8; 9] = [0; 9];
    syscall::read(read, buffer.as_mut_ptr(), buffer.len());
    syscall::write(com1, buffer.as_ptr(), buffer.len());

    loop {
      syscall::yield_coop();
    }
  }
  */

  /*
  let start = "Forking child. ";
  syscall::write(com1, start.as_ptr(), start.len());
  let pid = syscall::fork();
  if pid == 0 {
    let ch_start = "Child start. ";
    syscall::write(com1, ch_start.as_ptr(), ch_start.len());
    syscall::sleep(1000);
    let ch_end = "Child end. ";
    syscall::write(com1, ch_end.as_ptr(), ch_end.len());
    syscall::exit(0x10);
  } else {
    let (_, code) = syscall::wait_pid(pid);
    let back = "Back to parent. ";
    syscall::write(com1, back.as_ptr(), back.len());
    loop {
      syscall::yield_coop();
    }
  }
  */

  /*
  let pid = syscall::fork();
  let ticktock = if pid == 0 {
    "TOCK "
  } else {
    "TICK "
  };
  loop {
    syscall::write(com1, ticktock.as_ptr(), ticktock.len());
    syscall::sleep(1000);
  }
  */

  /*
  let pid = syscall::fork();
  if pid == 0 {
    let tty1 = syscall::open("DEV:\\TTY1");
    let prompt = "\nA:\\>";
    syscall::write(tty1, prompt.as_ptr(), prompt.len());

    loop {
      syscall::yield_coop();
    }
  } else if syscall::fork() == 0 {
    let console = syscall::open("DEV:\\TTY0");
    let msg = "TICK";
    loop {
      syscall::write(console, msg.as_ptr(), msg.len());
      syscall::sleep(1000);
    }
  } else {
    let fd = syscall::open("DEV:\\FD0");
    syscall::seek(fd, 0x2b);
    let mut buffer: [u8; 10] = [0; 10];
    syscall::read(fd, buffer.as_mut_ptr(), buffer.len());

    let console = syscall::open("DEV:\\TTY0");
    syscall::write(console, buffer.as_ptr(), buffer.len());

    syscall::seek_relative(fd, -10);
    syscall::read(fd, buffer.as_mut_ptr(), buffer.len());
    syscall::write(console, buffer.as_ptr(), buffer.len());

    loop {
      syscall::yield_coop();
    }
  }
  */

  /*
  let pid = syscall::fork();
  if pid == 0 {
    syscall::exec("INIT:\\test.bin");
  } else {
    loop {
      syscall::yield_coop();
    }
  }
  */

  loop {
    syscall::yield_coop();
  }
}