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
  let dev_fs = unsafe {
    super::get_fs(super::DEV_FS).unwrap()
  };
  let access_handle = dev_fs.open(device).unwrap();
  let device_no = dev_fs.ioctl(access_handle, 0, 0)? as usize;

  let mut fat_fs = fs::Fat12FileSystem::new(device_no, access_handle);
  fat_fs.init()?;

  Ok(Box::new(fat_fs))
}
