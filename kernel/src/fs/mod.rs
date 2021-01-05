pub mod drive;
pub mod drivers;
pub mod filesystem;

pub use alloc::boxed::Box;
pub use crate::memory::address::VirtualAddress;
pub use filesystem::{FileSystemCategory, KernelFileSystem};

pub static DRIVES: drive::DriveMap = drive::DriveMap::new();

pub fn init_system_drives(initfs_location: VirtualAddress) {
  let initfs = drivers::initfs::InitFileSystem::new(initfs_location);
  DRIVES.mount_drive("INIT", FileSystemCategory::KernelSync, Box::new(initfs));
}
