use crate::devices;
use crate::process;

pub extern "C" fn init() {
  let floppy = &devices::FLOPPY;
  match floppy.init() {
    Ok(_) => (),
    Err(e) => crate::kprintln!("Floppy Init Failed: {:?}", e),
  }

  process::send_signal(process::get_current_pid(), syscall::signals::STOP);
  process::yield_coop();
}