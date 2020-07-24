use crate::devices;
use crate::process;

pub extern "C" fn init() {
  let floppy = &devices::FLOPPY;
  match floppy.init() {
    Ok(_) => crate::tty::console_write(format_args!("Floppy Ready\n")),
    Err(e) => crate::tty::console_write(format_args!("Floppy Init Failed: {:?}\n", e)),
  }

  process::send_signal(process::get_current_pid(), syscall::signals::STOP);
  process::yield_coop();
}