pub mod directory;
pub mod disk;
pub mod errors;
pub mod fat;
pub mod file;
#[cfg(not(test))]
pub mod fs;

#[cfg(not(test))]
use alloc::boxed::Box;
#[cfg(not(test))]
use super::FileSystemType;

#[cfg(not(test))]
pub fn create_fs(device: &str) -> Result<Box<FileSystemType>, ()> {
  use crate::devices;

  let mut name: [u8; 8] = [0x20; 8];
  // copied from DEV: FS, needs to be updated when we use strings for
  // registration instead
  {
    let mut i = 0;
    let bytes = device.as_bytes();
    while i < 8 && i < bytes.len() {
      name[i] = bytes[i];
      i += 1;
    }
  }
  let device_no = devices::get_device_number_by_name(&name).ok_or(())?;
  let mut fat = fs::Fat12FileSystem::new(device_no);
  fat.init()?;
  Ok(Box::new(fat))
}
