use crate::devices::driver::DeviceDriver;

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
  fn open(&self) -> Result<usize, ()> {
    let router = super::get_router().read();
    let next = router.open_device(self.tty_id);
    match next {
      Some(id) => Ok(id),
      None => Err(()),
    }
  }

  fn close(&self, index: usize) -> Result<(), ()> {
    let router = super::get_router().read();
    router.close_device(self.tty_id);
    Ok(())
  }

  fn read(&self, index: usize, buffer: &mut [u8]) -> Result<usize, ()> {
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

  fn write(&self, index: usize, buffer: &[u8]) -> Result<usize, ()> {
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