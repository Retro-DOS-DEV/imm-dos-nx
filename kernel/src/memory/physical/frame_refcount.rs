use alloc::collections::BTreeMap;
use super::frame::Frame;
use super::super::address::PhysicalAddress;

/// While the purpose of the Frame Bitmap is to determine which areas of memory
/// are available, the Refcount Table determines how many different pages refer
/// to allocated, "anonymous" memory.
/// By default, the map does not track an allocated frame -- doing so would
/// duplicate the effort of the Frame Bitmap. Only frames that have been
/// explicitly copied -- often to support copy-on-write behavior -- are inserted
/// into the map. If a frame does not exist in the map, it can be assumed that
/// it has at most one reference.
pub struct FrameRefcount {
  references: BTreeMap<PhysicalAddress, usize>,
}

impl FrameRefcount {
  pub fn new() -> FrameRefcount {
    FrameRefcount {
      references: BTreeMap::new(),
    }
  }

  /// Increment the number of references to the frame containing a given
  /// physical memory address, returning the new total.
  pub fn reference_frame_at_address(&mut self, addr: PhysicalAddress) -> usize {
    let prev: usize = match self.references.get_mut(&addr) {
      Some(entry) => {
        let prev = *entry;
        *entry = prev + 1;
        prev
      },
      None => 0,
    };
    if prev == 0 {
      self.references.insert(addr, 2);
      2
    } else {
      prev
    }
  }

  pub fn release_frame_at_address(&mut self, addr: PhysicalAddress) -> usize {
    let remainder: Option<usize> = match self.references.get_mut(&addr) {
      Some(entry) => {
        if *entry == 0 {
          Some(0)
        } else {
          *entry -= 1;
          Some(*entry)
        }
      },
      None => Some(0),
    };
    match remainder {
      Some(x) if x < 2 => {
        self.references.remove(&addr);
        x
      },
      Some(x) => {
        x
      },
      None => 0,
    }
  }

  pub fn get_count_for_address(&self, addr: PhysicalAddress) -> usize {
    self.references
      .get(&addr)
      .map(|count| *count)
      .unwrap_or(1)
  }
}