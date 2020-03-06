#![feature(asm)]

#![no_std]

pub mod debug;
pub mod devices;
pub mod hardware;
pub mod memory;
pub mod panic;

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
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
  unsafe {
    kprintln!("\nEntering the Kernel...");

    // In the bootloader, we placed the memory map at 0x1000
    let mem_map = memory::map::load_entries_at_address(0x1000);
    kprintln!("Memory Map:");
    for entry in mem_map {
      kprintln!("{:?}", entry);
    }
    
  }

  loop {
    unsafe {
      asm!("hlt" : : : : "volatile");
    }
  }
}