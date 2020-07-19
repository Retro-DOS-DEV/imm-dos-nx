use crate::drivers::driver::DeviceDriver;
use crate::files::handle::LocalHandle;

/// Device driver representing a TTY, so a shell program can open up DEV:/TTY1
/// and listen to console input / publish to the terminal.
pub struct TTYDevice {
  tty_id: usize,
}

impl TTYDevice {
  pub fn for_tty(tty_id: usize) -> TTYDevice {
    TTYDevice {
      tty_id,
    }
  }
}

impl DeviceDriver for TTYDevice {
  fn open(&self, handle: LocalHandle) -> Result<(), ()> {

    Ok(())
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    
    Ok(())
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    let router = super::get_router().read();
    let buffers = router.get_tty_buffers(self.tty_id);
    match buffers {
      Some(b) => {
        let bytes_read = b.read(buffer);
        Ok(bytes_read)
      },
      None => Err(()),
    }
  }

  fn write(&self, _handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    let router = super::get_router().read();
    let buffers = router.get_tty_buffers(self.tty_id);
    match buffers {
      Some(b) => {
        let bytes_written = b.write(buffer);
        Ok(bytes_written)
      },
      None => Err(()),
    }
  }
}