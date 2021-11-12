use alloc::vec::Vec;
use crate::memory::address::PhysicalAddress;
use super::memory::MemoryBackup;

/// A vterm virtualizes access to the keyboard input and video output.
/// This is how the operating system achieves multitasking from the user's
/// perspective. DOS is inherently a single-tasking environment, where each
/// program takes over the entire screen. By capturing keyboard hooks to switch
/// between environments, it allows the user to run multiple DOS applications in
/// parallel.
/// 
/// Switching requires that each vterm stores all state necessary to reconstruct
/// the video state at any time, and can track any changes that happen while
/// inactive.
pub struct VTerm {
  pub video_mode: u8,
  memory_backups: Vec<MemoryBackup>,
}

impl VTerm {
  pub fn with_video_mode(mode: u8) -> Self {
    let mut memory_backups = Vec::new();
    match mode {
      0x03 => {
        memory_backups.push(
          MemoryBackup::allocate(
            PhysicalAddress::new(0xb8000),
          ),
        );
      },
      _ => (),
    }
    Self {
      video_mode: mode,
      memory_backups,
    }
  }

  /// When a VTerm becomes active, all stashed video state needs to be restored.
  /// Each active video memory area is copied back to physical memory. Depending
  /// on video state, some other IO ports may be set as well.
  pub fn make_active(&self) {
    unsafe {
      for backup in &self.memory_backups {
        backup.copy_from_buffer();
      }
    }
  }

  /// When a VTerm becomes inactive, it needs to store its current state. This
  /// involves copying all active video memory areas to their back buffers.
  pub fn make_inactive(&self) {
    unsafe {
      for backup in &self.memory_backups {
        backup.copy_to_buffer();
      }
    }
  }
}