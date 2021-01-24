//! A process can simulate a DOS 8086 environment to execute DOS programs.

use crate::memory::address::VirtualAddress;

pub fn address_from_segmented(segment: u32, offset: u32) -> VirtualAddress {
  let full_address = ((segment << 4) + offset) as usize;
  VirtualAddress::new(full_address)
}

/// Each process contains its own global DOS state, like current drive and PSP
/// address. Those values are stored in an optional DosState struct attached to
/// the process.
pub struct DosState {
  /// The "default" drive used in file IO operations
  /// The values are zero-indexed, where 0 == A: and 25 == Z:
  current_drive: usize,
  
}
