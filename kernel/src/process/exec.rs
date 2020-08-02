use crate::files::handle::LocalHandle;
use crate::memory::address::VirtualAddress;
use super::process_state::ProcessState;
use super::subsystem::{DosSubsystemMetadata, Subsystem};

/**
 * Used to force the kernel to interpret an executable as a specific format
 */
pub enum InterpretationMode {
  Detect, // Try to determine the executable type from magic numbers
  BIN, // Assume it's ExecType::BIN
  ELF, // Assume it's ExecType::ELF
  COM, // Assume it's ExecType::COM
  DOS, // Assume it's ExecType::DOS
}

impl InterpretationMode {
  pub fn from_u32(raw: u32) -> InterpretationMode {
    match raw {
      1 => InterpretationMode::BIN,
      2 => InterpretationMode::ELF,
      3 => InterpretationMode::COM,
      4 => InterpretationMode::DOS,
      _ => InterpretationMode::Detect,
    }
  }
}

/**
 * Represents the type of executable running in this process, used for
 * memmapping the file, initializing memory, and determining the subsystem.
 */
pub enum ExecFormat {
  BIN, // Native 32-bit binary
  ELF, // Native 32-bit ELF program
  COM, // Header-less 16-bit binary
  DOS, // MZ executable
}

impl ProcessState {
  pub fn detect_exec_format(&self) -> ExecFormat {
    // Read the header of the file to check for magic numbers
    let buffer: [u8; 4] = [0; 4];
    // read first 4 bytes of file

    if buffer[0..2] == [b'M', b'Z'] {
      ExecFormat::DOS
    } else if buffer == [0x7f, b'E', b'L', b'F'] {
      ExecFormat::ELF
    } else {
      ExecFormat::BIN
    }
  }

  pub fn get_exec_format(&self, interp_mode: InterpretationMode) -> ExecFormat {
    match interp_mode {
      InterpretationMode::Detect => self.detect_exec_format(),
      InterpretationMode::BIN => ExecFormat::BIN,
      InterpretationMode::ELF => ExecFormat::ELF,
      InterpretationMode::COM => ExecFormat::COM,
      InterpretationMode::DOS => ExecFormat::DOS,
    }
  }

  pub fn create_dos_psp(&self, prog_start: VirtualAddress) {
    // Create a DOS PSP struct in the 256 bytes before prog_start
  }

  pub fn prepare_for_exec(&self, drive_number: usize, handle: LocalHandle, interp_mode: InterpretationMode) -> usize {
    let format = self.get_exec_format(interp_mode);
    let new_subsystem = match format {
      ExecFormat::DOS | ExecFormat::COM => Subsystem::DOS(DosSubsystemMetadata::new()),
      _ => Subsystem::Native,
    };

    self.unmap_all();

    let entry = match format {
      ExecFormat::BIN => {
        // need to read the file length
        let length = 0xf0;
        self.mmap(VirtualAddress::new(0), length, drive_number, handle);
        // Start the brk heap space on the next page
        self.start_heap(VirtualAddress::new((length + 0x1000) & 0x1000));
        // Entry is always 0
        0
      },
      ExecFormat::ELF => {
        panic!("Can't interpret ELF files yet!");
      },
      ExecFormat::COM => {
        // need to read the file length
        let length = 0xf0;
        // To simplify memmapping the files, we start the executable on a page
        // boundary, and place the PSP in the last bytes of the previous page.
        let prog_start = 0x1000;
        self.anonymous_map(VirtualAddress::new(0), prog_start);
        self.mmap(VirtualAddress::new(prog_start), length, drive_number, handle);
        // need to do some direct and anonymous mapping for the remainder of the
        // first 640KiB

        // Create the PSP, in case the program uses it
        self.create_dos_psp(VirtualAddress::new(prog_start));
        prog_start
      },
      ExecFormat::DOS => {
        panic!("Can't interpret MZ executables yet!");
      },
    };
    *self.get_subsystem().write() = new_subsystem;

    entry
  }
}
