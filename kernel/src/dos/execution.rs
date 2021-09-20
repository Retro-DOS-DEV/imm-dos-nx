use super::memory::SegmentedAddress;
use super::registers::{DosApiRegisters, VM86Frame};

/// The Program Segment Prefix (PSP) is an in-memory header that stores program
/// state. It is always paragraph-aligned. Many of the fields are unused legacy
/// values, but useful fields have been made public.
#[repr(C, packed)]
pub struct PSP {
  /// Shortcut to terminate the program via int 20h
  int_20: [u8; 2], // 0x00
  /// First segment beyond the memory allocated for this program
  pub memory_top_paragraphs: u16, // 0x02
  dos_reserved: u8, // 0x04
  /// Long jump to the DOS API dispatcher
  dispatcher_long: [u8; 5], // 0x05
  /// Used to restore the value of int 22, if changed by the program
  pub termination_vector: SegmentedAddress, // 0x0a
  /// Used to restore the value of int 23, if changed by the program
  pub control_break_vector: SegmentedAddress, // 0x0e
  /// Used to restore the value of int 24, if changed by the program
  pub critical_error_vector: SegmentedAddress, // 0x12
  /// Segment of the parent's PSP. If there is no parent, it is this PSP's segment
  pub parent_segment: u16, // 0x16
  /// Contains aliases from local handles to the corresponding entries in the
  /// System File Table. The first five entries are STDIN, STDOUT, STDERR,
  /// STDAUX, STDPRN
  pub file_handles: [u8; 20], // 0x18
  /// Segment of the current ENV string
  pub env_segment: u16, // 0x2c
  /// Stores the stack address when calling into DOS API. Not needed in our VM
  stack_save: u32, // 0x2e
  /// Length of the file handle table
  handle_array_length: u16, // 0x32
  /// Pointer to the file handle table, in case it has been relocated to extend
  /// beyond 20 open files
  handle_array_pointer: u32, // 0x34
  /// Pointer to the previous PSP, buy typically unused
  previous_psp: u32, // 0x38
  dos_reserved_2: [u8; 4], // 0x3c
  /// Contains the DOS version to return from API calls, in case it has been
  /// overridden with the SETVER command
  pub dos_version: u16, // 0x40
  dos_reserved_3: [u8; 14], // 0x42
  /// Another dispatcher, int 21h + RETF
  dispatcher: [u8; 3], // 0x50
  unused: [u8; 9], // 0x53
  /// Reserved space for the first FCB
  fcb_first: [u8; 16], // 0x5c
  /// Reserved space for the second FCB
  fcb_second: [u8; 20], // 0x6c
  /// Number of bytes in the command tail
  pub command_tail_length: u8, // 0x80
  /// Actual contents of the command tail (arguments passed after the executable
  /// name)
  pub command_tail: [u8; 127], // 0x81
}

impl PSP {
  pub fn reset(&mut self) {
    self.int_20 = [0xcd, 0x20];
    self.dispatcher = [0xcd, 0x21, 0xcb];
    self.parent_segment = self.get_segment();

    self.file_handles = [
      0, 1, 2, 3, 4,
      0xff, 0xff, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xff, 0xff,
    ];
  }

  pub fn get_segment(&self) -> u16 {
    (self.as_address() >> 4) as u16
  }

  pub fn as_ptr(&self) -> *const PSP {
    self as *const PSP
  }

  pub fn as_address(&self) -> usize {
    self.as_ptr() as usize
  }

  pub fn as_segmented_address(&self) -> SegmentedAddress {
    SegmentedAddress {
      segment: (self.as_address() >> 4) as u16,
      offset: 0,
    }
  }

  pub unsafe fn at_segment(segment: u16) -> &'static mut PSP {
    let address = (segment as usize) << 4;
    let ptr = address as *mut PSP;
    &mut *ptr
  }

  pub fn get_parent_segment(&self) -> Option<u16> {
    if self.parent_segment == self.get_segment() {
      None
    } else {
      Some(self.parent_segment)
    }
  }
}

/// int 21, 0 - Program Terminate
/// Restores interrupt vectors 0x22, 0x23, 0x24. Frees the memory allocated to
/// this program. Does not close any open files.
pub fn terminate(cs: u16) -> SegmentedAddress {
  // assumes CS is the PSP segment
  let psp = unsafe { PSP::at_segment(cs) };
  // clean up resources

  match psp.get_parent_segment() {
    Some(_seg) => {
      // jump to parent psp via the terminate address
      psp.termination_vector
    },
    None => {
      // Top-level DOS program, exit the process
      SegmentedAddress {
        segment: 0,
        offset: 0,
      }
    },
  }
}