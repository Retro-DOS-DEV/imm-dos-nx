pub mod dma;
#[cfg(not(test))]
pub mod floppy;
pub mod pic;
pub mod pit;
pub mod qemu;
pub mod rtc;
pub mod vga;