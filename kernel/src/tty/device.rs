use crate::devices::driver::{DeviceDriver, IOHandle};
use crate::task::id::ProcessID;

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
  fn open(&self) -> Result<IOHandle, ()> {
    let router = super::get_router().read();
    let next = router.open_device(self.tty_id);
    match next {
      Some(handle) => Ok(handle),
      None => Err(()),
    }
  }

  fn close(&self, handle: IOHandle) -> Result<(), ()> {
    let router = super::get_router().read();
    router.close_device(self.tty_id, handle);
    Ok(())
  }

  fn read(&self, handle: IOHandle, dest: &mut [u8]) -> Result<usize, ()> {
    let buffer = {
      let router = super::get_router().read();
      router.get_tty_reader_buffer(self.tty_id)
    };
    match buffer {
      Some(b) => {
        let bytes_read = b.read(handle, dest);
        Ok(bytes_read)
      },
      None => Err(()),
    }
  }

  fn write(&self, index: IOHandle, buffer: &[u8]) -> Result<usize, ()> {
    // this needs enqueuing
    let mut total_written = 0;
    loop {
      let remainder = &buffer[total_written..];
      let partial_write = {
        let router = super::get_router().read();
        let buffers = router.get_tty_buffers(self.tty_id);
        match buffers {
          Some(b) => {
            let bytes_written = b.write(remainder);
            Ok(bytes_written)
          },
          None => Err(()),
        }
      }?;
      total_written += partial_write;
      if total_written >= buffer.len() {
        break;
      }
      crate::task::yield_coop();
    }
    Ok(total_written)
  }

  fn reopen(&self, index: IOHandle, id: ProcessID) -> Result<IOHandle, ()> {
    let router = super::get_router().read();
    let new_handle = router.reopen_device(self.tty_id, id);
    match new_handle {
      Some(handle) => Ok(handle),
      None => Err(()),
    }
  }
}
