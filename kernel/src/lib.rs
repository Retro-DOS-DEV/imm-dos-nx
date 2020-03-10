#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]

#![no_std]

pub mod debug;
pub mod devices;
pub mod hardware;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod panic;
pub mod x86;

use memory::address::PhysicalAddress;

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
  #[link_name = "__stack_start"]
  static label_stack_start: u8;
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
  unsafe {
    // In the bootloader, we placed the memory map at 0x1000
    let mem_map = memory::map::load_entries_at_address(0x1000);

    let kernel_start = PhysicalAddress::new(&label_ro_physical_start as *const u8 as usize);
    let kernel_end = PhysicalAddress::new(&label_rw_physical_end as *const u8 as usize);
    memory::init(kernel_start, kernel_end);

    // Initialize paging
    memory::init_paging();
    let stack_start = PhysicalAddress::new(&label_stack_start as *const u8 as usize);
    memory::move_kernel_stack(memory::frame::Frame::containing_address(stack_start));

    kprintln!("\nEntering the Kernel...");

    // Initialize interrupts
    idt::init();

    kprintln!("Memory Map:");
    for entry in mem_map {
      kprintln!("{:?}", entry);
    }

    kprintln!("Kernel goes from {:?} to {:?}", kernel_start, kernel_end);

    kprint!("Frame Table: ----------------------------------");
    for i in 0..memory::FRAME_ALLOCATOR.length {
      if i & 15 == 0 {
        kprintln!();
      }
      let ptr = memory::FRAME_ALLOCATOR.start as *const u8;
      kprint!("{:02x} ", *(ptr.offset(i as isize)));
    }
    kprintln!("\nTotal Frames: {}\nFree Frames: {}", memory::count_frames(), memory::count_free_frames());

    // Update GDT

    // Initialize kernel heap

  }

  loop {
    unsafe {
      asm!("hlt" : : : : "volatile");
    }
  }
}