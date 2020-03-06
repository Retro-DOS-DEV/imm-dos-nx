#![feature(asm)]

#![no_std]

pub mod debug;
pub mod devices;
pub mod hardware;
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
    // just write something so we know we hit the kernel
    asm!("mov eax, 0x99" : : : "eax" : "intel", "volatile");
  }

  kprintln!("\nEntering the Kernel...");

  loop {
    unsafe {
      asm!("hlt" : : : : "volatile");
    }
  }
}