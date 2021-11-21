use crate::fs::DRIVES;
use syscall::result::SystemError;

/// Register the current process as a new filesystem driver
pub fn register() {

}

pub fn set_current_drive(name: &str) -> Result<u32, SystemError> {
  let drive_id = DRIVES.get_drive_number(name).ok_or(SystemError::NoSuchDrive)?;
  let current_lock = crate::task::get_current_process();
  let mut current = current_lock.write();
  current.current_drive = drive_id;
  Ok(drive_id.as_u32())
}

pub fn get_current_drive_name(buffer: &mut [u8]) -> Result<u32, SystemError> {
  let drive_id = {
    let current_lock = crate::task::get_current_process();
    let current = current_lock.read();
    current.current_drive
  };
  let name = match crate::fs::DRIVES.get_drive_name(&drive_id) {
    Some(name) => name,
    None => return Err(SystemError::NoSuchDrive),
  };
  let name_bytes = name.as_bytes();
  let mut len = name_bytes.len();
  if len > 8 {
    len = 8;
  }
  for i in 0..len {
    buffer[i] = name_bytes[i];
  }
  Ok(len as u32)
}

pub fn get_current_drive_number() -> Result<u32, SystemError> {
  let current_lock = crate::task::get_current_process();
  let current = current_lock.read();
  Ok(current.current_drive.as_u32())
}
