use crate::drivers::floppy;
use crate::devices;
use crate::process;

pub extern "C" fn floppy_driver() {
  let floppy = &devices::FLOPPY;
  match floppy.init() {
    Ok(_) => crate::tty::console_write(format_args!("Floppy Controller Ready\n")),
    Err(e) => crate::tty::console_write(format_args!("Floppy Init Failed: {:?}\n", e)),
  }

  floppy::init_dma();

  process::send_signal(process::get_current_pid(), syscall::signals::STOP);
  process::yield_coop();
}