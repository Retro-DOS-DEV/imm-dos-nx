use crate::drivers::floppy;
use crate::filesystems;
use crate::devices;
use crate::process;

pub extern "C" fn floppy_driver() {
  let floppy = &devices::FLOPPY;
  match floppy.init() {
    Ok(_) => crate::tty::console_write(format_args!("Floppy Controller Ready\n")),
    Err(e) => crate::tty::console_write(format_args!("Floppy Init Failed: {:?}\n", e)),
  }

  floppy::init_dma();

  let fat_fs = filesystems::fat12::create_fs("FD0").unwrap();
  filesystems::VFS.register_fs("A", fat_fs).expect("Failed to register A:");

  process::send_signal(process::id::ProcessID::new(1), syscall::signals::CONTINUE);

  process::send_signal(process::get_current_pid(), syscall::signals::STOP);
  process::yield_coop();
}