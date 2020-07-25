use crate::files::filename;
use crate::filesystems;
use crate::process;
use syscall::result::SystemError;

pub fn yield_coop() {
  process::yield_coop();
}

pub fn sleep(ms: u32) {
  process::sleep(ms as usize)
}

pub fn fork() -> u32 {
  process::fork()
}

pub fn exec_path(path_str: &'static str, arg_str: &'static str, raw_interp_mode: u32) -> Result<(), SystemError> {
  let (drive, path) = filename::string_to_drive_and_path(path_str);
  let number = filesystems::get_fs_number(drive).ok_or(SystemError::NoSuchDrive)?;
  let fs = filesystems::get_fs(number).ok_or(SystemError::NoSuchFileSystem)?;
  let local_handle = fs.open(path).map_err(|_| SystemError::NoSuchEntity)?;
  let interp_mode = process::exec::InterpretationMode::from_u32(raw_interp_mode);
  process::exec(number, local_handle, interp_mode);
  Ok(())
}

pub fn exit(code: u32) {
  process::exit(code);
}

pub fn get_pid() -> u32 {
  process::get_current_pid().as_u32()
}

pub fn raise_signal(sig: u32) {
  let id = process::get_current_pid();
  process::send_signal(id, sig);
}

pub fn send_signal(id: u32, sig: u32) {
  process::send_signal(process::id::ProcessID::new(id), sig);
}

pub fn wait_pid(id: u32) -> (u32, u32) {
  if id == 0 {
    // TODO: wait on any process
    (0, 0)
  } else {
    let code = process::wait(process::id::ProcessID::new(id));
    (id, code)
  }
}
